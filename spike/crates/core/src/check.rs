//! The check traits.
//!
//! Spec 12 models scope as a value that a check returns from `scope()`. The
//! spike found that a *type* is the better carrier, and the difference is the
//! point of item 2.
//!
//! With one trait and a returned `Scope`, a check declares one scope and can
//! still receive a view that reads more, because nothing ties the declaration
//! to the argument. With one trait per scope, the declaration *is* the argument
//! type. A check that wants to read siblings has to implement `CorpusCheck`,
//! and it then registers in the corpus bucket, where the runner already knows
//! to key its cache on the whole corpus. The declaration cannot disagree with
//! what the check reads, because there is only one fact and not two.
//!
//! `Scope` survives as a reporting and cache-keying value, derived from the
//! trait rather than supplied by the implementer.

use crate::finding::{Finding, Severity};
use crate::view::{CorpusView, DocumentView, EdgeView};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Document,
    Edge,
    Corpus,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Document => "document",
            Scope::Edge => "edge",
            Scope::Corpus => "corpus",
        }
    }
}

/// Shared identity, so that findings and cache entries can attach to any check
/// whatever its scope.
pub trait CheckMeta {
    fn id(&self) -> &'static str;
    /// Required. Spec 4 admits no orphan checks, and spec 12 keeps this on the
    /// plugin surface for the same reason.
    fn obligation(&self) -> &'static str;
    fn severity(&self) -> Severity;
    /// Version participates in the cache key, so a changed check invalidates
    /// its own cached results.
    fn version(&self) -> u32 {
        1
    }
}

/// `Send + Sync` is not decoration. The runner evaluates document and edge
/// checks on several threads over one shared graph, so a check holding interior
/// mutability fails to compile at the point of registration rather than racing
/// at run time. `&self` on `evaluate` is the other half: a check cannot
/// accumulate state between its instances.
pub trait DocumentCheck: CheckMeta + Send + Sync {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding>;
    fn scope(&self) -> Scope {
        Scope::Document
    }
}

pub trait EdgeCheck: CheckMeta + Send + Sync {
    fn evaluate(&self, view: &EdgeView<'_>) -> Vec<Finding>;
    fn scope(&self) -> Scope {
        Scope::Edge
    }
}

pub trait CorpusCheck: CheckMeta + Send + Sync {
    fn evaluate(&self, view: &CorpusView<'_>) -> Vec<Finding>;
    fn scope(&self) -> Scope {
        Scope::Corpus
    }
}

/// The registry is three buckets, not one, for the reason above.
#[derive(Default)]
pub struct Registry {
    pub document: Vec<Box<dyn DocumentCheck>>,
    pub edge: Vec<Box<dyn EdgeCheck>>,
    pub corpus: Vec<Box<dyn CorpusCheck>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_document(mut self, c: impl DocumentCheck + 'static) -> Self {
        self.document.push(Box::new(c));
        self
    }

    pub fn with_edge(mut self, c: impl EdgeCheck + 'static) -> Self {
        self.edge.push(Box::new(c));
        self
    }

    pub fn with_corpus(mut self, c: impl CorpusCheck + 'static) -> Self {
        self.corpus.push(Box::new(c));
        self
    }

    pub fn count(&self) -> usize {
        self.document.len() + self.edge.len() + self.corpus.len()
    }
}
