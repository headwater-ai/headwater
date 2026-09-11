// SPDX-License-Identifier: Apache-2.0
//! The runner: census, cache, and change-scoped evaluation.
//!
//! **Item 4 of the spike.** The claim under test is that a change-scoped run
//! over a 1,000-document corpus fits inside 200 ms warm.
//!
//! The runner is also where two of spec 12's rules become mechanical rather
//! than aspirational. Cache keys carry every in-scope input, and findings sort
//! before they leave.

use crate::check::{Registry, Scope};
use crate::finding::{sort, Finding};
use crate::hash;
use crate::model::{DocIdx, Graph, Target};
use crate::view::{CorpusView, DocumentView, EdgeView};
use std::collections::HashMap;

/// A cache key covers exactly the inputs in scope, plus the taxonomy lock hash
/// and the check's version. Spec 12 calls a key that omits an input a
/// correctness bug, so the construction is centralized here.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CacheKey {
    pub check: &'static str,
    pub version: u32,
    pub target: u128,
    pub taxonomy: u128,
}

#[derive(Default)]
pub struct Cache {
    entries: HashMap<CacheKey, Vec<Finding>>,
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// What a run accounted for. Spec 4's no-silent-passes rule reduces to this
/// being reported rather than inferred.
#[derive(Debug, Default, Clone)]
pub struct RunReport {
    pub findings: Vec<Finding>,
    pub instances_created: usize,
    pub instances_evaluated: usize,
    pub instances_from_cache: usize,
    pub documents_in_census: usize,
    pub documents_with_zero_instances: Vec<String>,
}

/// Instance identity, used for the coverage accounting above.
fn doc_target_hash(graph: &Graph, idx: DocIdx) -> u128 {
    graph.doc(idx).content_hash
}

/// An edge instance depends on *both* endpoints, so both hashes go into the
/// key. This is what makes the "invalidate in both directions" rule sound
/// rather than merely stated.
fn edge_target_hash(graph: &Graph, edge_idx: usize) -> u128 {
    let e = &graph.edges()[edge_idx];
    let from = graph.doc(e.from).content_hash;
    let to = match e.target {
        Target::Resolved(j) => graph.doc(j).content_hash,
        Target::Dangling(_) => 0,
    };
    hash::combine(&[from, to, e.relation as u128, e.span.start.line as u128])
}

fn corpus_target_hash(graph: &Graph) -> u128 {
    let mut parts: Vec<u128> = graph.documents().iter().map(|d| d.content_hash).collect();
    parts.sort_unstable();
    hash::combine(&parts)
}

pub struct Runner<'r> {
    registry: &'r Registry,
    taxonomy: u128,
}

impl<'r> Runner<'r> {
    pub fn new(registry: &'r Registry, taxonomy_hash: u128) -> Self {
        Self {
            registry,
            taxonomy: taxonomy_hash,
        }
    }

    /// Full-corpus run. Every instance is created before any check runs, so the
    /// denominator is fixed first (spec 12, phase A then phase B).
    pub fn run_full(&self, graph: &Graph, cache: &mut Cache) -> RunReport {
        let docs: Vec<DocIdx> = (0..graph.documents().len()).collect();
        let edges: Vec<usize> = (0..graph.edges().len()).collect();
        self.run(graph, cache, &docs, &edges, true)
    }

    /// Change-scoped run. Invalidation follows spec 12 exactly: document checks
    /// for changed files, edge checks for every edge *incident to* a changed
    /// file in both directions, and corpus checks unconditionally, because a
    /// corpus check has the whole corpus in scope and cannot be narrowed.
    pub fn run_changed(&self, graph: &Graph, cache: &mut Cache, changed: &[DocIdx]) -> RunReport {
        let mut edges: Vec<usize> = Vec::new();
        for &d in changed {
            edges.extend_from_slice(graph.incident_edges(d));
        }
        edges.sort_unstable();
        edges.dedup();
        self.run(graph, cache, changed, &edges, true)
    }

    fn run(
        &self,
        graph: &Graph,
        cache: &mut Cache,
        docs: &[DocIdx],
        edges: &[usize],
        run_corpus: bool,
    ) -> RunReport {
        let mut report = RunReport {
            documents_in_census: graph.documents().len(),
            ..Default::default()
        };

        // ---- instance creation (the denominator) ----
        let mut doc_jobs: Vec<(usize, DocIdx, CacheKey)> = Vec::new();
        for &d in docs {
            for (ci, c) in self.registry.document.iter().enumerate() {
                doc_jobs.push((
                    ci,
                    d,
                    CacheKey {
                        check: c.id(),
                        version: c.version(),
                        target: doc_target_hash(graph, d),
                        taxonomy: self.taxonomy,
                    },
                ));
            }
        }
        let mut edge_jobs: Vec<(usize, usize, CacheKey)> = Vec::new();
        for &e in edges {
            for (ci, c) in self.registry.edge.iter().enumerate() {
                edge_jobs.push((
                    ci,
                    e,
                    CacheKey {
                        check: c.id(),
                        version: c.version(),
                        target: edge_target_hash(graph, e),
                        taxonomy: self.taxonomy,
                    },
                ));
            }
        }
        let corpus_jobs: Vec<(usize, CacheKey)> = if run_corpus {
            let th = corpus_target_hash(graph);
            self.registry
                .corpus
                .iter()
                .enumerate()
                .map(|(ci, c)| {
                    (
                        ci,
                        CacheKey {
                            check: c.id(),
                            version: c.version(),
                            target: th,
                            taxonomy: self.taxonomy,
                        },
                    )
                })
                .collect()
        } else {
            Vec::new()
        };

        report.instances_created = doc_jobs.len() + edge_jobs.len() + corpus_jobs.len();

        // ---- cache partition ----
        let mut misses_doc = Vec::new();
        for job in doc_jobs {
            match cache.entries.get(&job.2) {
                Some(hit) => {
                    report.instances_from_cache += 1;
                    report.findings.extend(hit.iter().cloned());
                }
                None => misses_doc.push(job),
            }
        }
        let mut misses_edge = Vec::new();
        for job in edge_jobs {
            match cache.entries.get(&job.2) {
                Some(hit) => {
                    report.instances_from_cache += 1;
                    report.findings.extend(hit.iter().cloned());
                }
                None => misses_edge.push(job),
            }
        }
        let mut misses_corpus = Vec::new();
        for job in corpus_jobs {
            match cache.entries.get(&job.1) {
                Some(hit) => {
                    report.instances_from_cache += 1;
                    report.findings.extend(hit.iter().cloned());
                }
                None => misses_corpus.push(job),
            }
        }

        // ---- evaluation ----
        let doc_results = self.eval_documents(graph, &misses_doc);
        let edge_results = self.eval_edges(graph, &misses_edge);

        for ((_, _, key), found) in misses_doc.iter().zip(doc_results) {
            report.instances_evaluated += 1;
            report.findings.extend(found.iter().cloned());
            cache.entries.insert(*key, found);
        }
        for ((_, _, key), found) in misses_edge.iter().zip(edge_results) {
            report.instances_evaluated += 1;
            report.findings.extend(found.iter().cloned());
            cache.entries.insert(*key, found);
        }
        for (ci, key) in misses_corpus {
            let view = CorpusView::new(graph);
            let found = self.registry.corpus[ci].evaluate(&view);
            report.instances_evaluated += 1;
            report.findings.extend(found.iter().cloned());
            cache.entries.insert(key, found);
        }

        // A document with zero instances is a finding (spec 12).
        if self.registry.document.is_empty() {
            for d in graph.documents() {
                report.documents_with_zero_instances.push(d.path.clone());
            }
        }

        sort(&mut report.findings);
        report
    }

    #[cfg(feature = "parallel")]
    fn eval_documents(
        &self,
        graph: &Graph,
        jobs: &[(usize, DocIdx, CacheKey)],
    ) -> Vec<Vec<Finding>> {
        self.parallel_map(jobs, |(ci, d, _)| {
            let view = DocumentView::new(graph.doc(*d));
            self.registry.document[*ci].evaluate(&view)
        })
    }

    #[cfg(not(feature = "parallel"))]
    fn eval_documents(
        &self,
        graph: &Graph,
        jobs: &[(usize, DocIdx, CacheKey)],
    ) -> Vec<Vec<Finding>> {
        jobs.iter()
            .map(|(ci, d, _)| {
                let view = DocumentView::new(graph.doc(*d));
                self.registry.document[*ci].evaluate(&view)
            })
            .collect()
    }

    #[cfg(feature = "parallel")]
    fn eval_edges(&self, graph: &Graph, jobs: &[(usize, usize, CacheKey)]) -> Vec<Vec<Finding>> {
        self.parallel_map(jobs, |(ci, e, _)| {
            let edge = &graph.edges()[*e];
            let to = match edge.target {
                Target::Resolved(j) => Some(graph.doc(j)),
                Target::Dangling(_) => None,
            };
            let view = EdgeView::new(edge, graph.doc(edge.from), to, graph);
            self.registry.edge[*ci].evaluate(&view)
        })
    }

    #[cfg(not(feature = "parallel"))]
    fn eval_edges(&self, graph: &Graph, jobs: &[(usize, usize, CacheKey)]) -> Vec<Vec<Finding>> {
        jobs.iter()
            .map(|(ci, e, _)| {
                let edge = &graph.edges()[*e];
                let to = match edge.target {
                    Target::Resolved(j) => Some(graph.doc(j)),
                    Target::Dangling(_) => None,
                };
                let view = EdgeView::new(edge, graph.doc(edge.from), to, graph);
                self.registry.edge[*ci].evaluate(&view)
            })
            .collect()
    }

    /// Chunked fan-out over scoped threads. The `Send + Sync` bound on the
    /// check traits is what makes this compile, and it is what would reject a
    /// check carrying interior mutability.
    #[cfg(feature = "parallel")]
    fn parallel_map<J, F>(&self, jobs: &[J], f: F) -> Vec<Vec<Finding>>
    where
        J: Sync,
        F: Fn(&J) -> Vec<Finding> + Send + Sync,
    {
        let n = jobs.len();
        if n < 256 {
            return jobs.iter().map(&f).collect();
        }
        let threads = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1)
            .min(8);
        if threads <= 1 {
            return jobs.iter().map(&f).collect();
        }
        let chunk = n.div_ceil(threads);
        let f = &f;
        std::thread::scope(|s| {
            let handles: Vec<_> = jobs
                .chunks(chunk)
                .map(|c| s.spawn(move || c.iter().map(f).collect::<Vec<_>>()))
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect()
        })
    }
}

/// Convenience for reporting.
pub fn scope_counts(registry: &Registry) -> Vec<(Scope, usize)> {
    vec![
        (Scope::Document, registry.document.len()),
        (Scope::Edge, registry.edge.len()),
        (Scope::Corpus, registry.corpus.len()),
    ]
}
