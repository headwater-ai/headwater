// SPDX-License-Identifier: Apache-2.0
//! The briefing: what an agent is to read, and what it already may not report.
//!
//! # A plan carries pointers and declarations, and never content
//!
//! The members below are paths, identifiers, kinds, titles and summaries. The
//! agent opens the files itself. Two reasons, and the second is the one that
//! matters. A plan that inlined the corpus would be a second copy of it that
//! goes stale between the plan and the reading. And the reading is the sweep's
//! own work: an agent that never opened a document cannot cite a passage from
//! it, and a citation is the only thing [`crate::intake`] can verify.
//!
//! # The declared half is in the plan so that it cannot be reported back
//!
//! Spec 4's fourth constraint: "It never re-derives what the graph declares.
//! The sweep runs against the undeclared half by construction." The plan
//! therefore lists every edge already declared among the members. An agent that
//! reads the plan knows what is already known, and [`crate::intake`] refuses a
//! finding that restates one whether or not the agent read it.
//!
//! # The slice is the caller's, and the plan reports its own extent
//!
//! There is no sampling rule here. A rule that picked the slice would be an
//! unreproducible sample dressed as a reproducible one, and the caller who
//! knows which corner of the corpus is worth an hour of a model is the person
//! or the schedule that asked. What the plan owes instead is its **extent**:
//! how many classified documents are in the slice against how many are in the
//! corpus. A sweep report over 12 of 169 documents is not a statement about
//! 169, and the number rides on the plan so that no report can lose it.

use headwater_census::census::{Census, Outcome};
use headwater_graph::edges::Target;
use headwater_graph::{Config, Graph};

/// One document of a slice, as the agent meets it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub path: String,
    pub id: Option<String>,
    pub kind: String,
    pub title: Option<String>,
    pub summary: Option<String>,
}

/// One edge the graph already carries between two members of the slice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declared {
    pub from: String,
    pub relation: String,
    pub to: String,
}

/// A bounded slice of the corpus, with what is already known about it.
#[derive(Clone, Debug)]
pub struct Plan {
    /// The digest of the lock this plan was taken against. A return file names
    /// it, and [`crate::intake`] refuses a file that names another: a sweep of
    /// one taxonomy read back against a second is a report about neither.
    pub lock: String,
    /// The path prefix the caller named.
    pub under: String,
    pub members: Vec<Member>,
    pub declared: Vec<Declared>,
    /// Classified documents in the whole corpus, which is the denominator the
    /// slice is a part of.
    pub corpus: usize,
}

impl Plan {
    /// Take a plan over the documents under one path.
    pub fn over(census: &Census, graph: &Graph, config: &Config, lock: &str, under: &str) -> Plan {
        let under = under.trim_end_matches('/').to_string();
        let mut members = Vec::new();
        let mut corpus = 0;
        for row in &census.rows {
            let Outcome::Typed { kind, .. } = &row.outcome else {
                continue;
            };
            corpus += 1;
            if !within(&row.path, &under) {
                continue;
            }
            let facets = row.document.as_ref().map(|document| &document.facets);
            members.push(Member {
                path: row.path.clone(),
                id: facets.and_then(|facets| scalar(facets, &config.identifier_facet)),
                kind: kind.clone(),
                title: facets.and_then(|facets| scalar(facets, "title")),
                summary: facets.and_then(|facets| scalar(facets, "summary")),
            });
        }
        members.sort_by(|a, b| a.path.cmp(&b.path));

        let paths: Vec<&str> = members.iter().map(|member| member.path.as_str()).collect();
        let mut declared = Vec::new();
        for edge in &graph.edges {
            let Target::Document { path, .. } = &edge.target else {
                continue;
            };
            if !paths.contains(&edge.source.path.as_str()) || !paths.contains(&path.as_str()) {
                continue;
            }
            declared.push(Declared {
                from: edge.source.path.clone(),
                relation: edge.declared.clone(),
                to: path.clone(),
            });
        }
        declared.sort_by(|a, b| (&a.from, &a.relation, &a.to).cmp(&(&b.from, &b.relation, &b.to)));
        declared.dedup();

        Plan {
            lock: lock.to_string(),
            under,
            members,
            declared,
            corpus,
        }
    }

    /// The briefing, in the engine's own words.
    ///
    /// One format, and it is prose. The reader is an agent, and every other
    /// thing an agent reads in this repository — a skill, the report of
    /// `headwater new`, the pointers of `headwater route` — is prose. A second
    /// machine format would be a second contract to keep in step, and the file
    /// that comes *back* is the one that has to parse.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "A coherence sweep over {}, {} of the {} classified documents of this corpus.",
            match self.under.is_empty() {
                true => "the whole corpus",
                false => &self.under,
            },
            self.members.len(),
            self.corpus
        );
        let _ = writeln!(out, "taxonomy: {}", self.lock);
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "Read every document below. Report only what no linter can see, in one of the {} \
             classes named at the end. Your output is a proposal a person reads and accepts, and \
             it is never a verdict: nothing in this repository gates on it.",
            crate::Class::ALL.len()
        );
        let _ = writeln!(out);

        let _ = writeln!(out, "## The slice");
        let _ = writeln!(out);
        for member in &self.members {
            let _ = writeln!(out, "- {} ({})", member.path, member.kind);
            if let Some(id) = &member.id {
                let _ = writeln!(out, "    id: {id}");
            }
            if let Some(title) = &member.title {
                let _ = writeln!(out, "    title: {title}");
            }
            if let Some(summary) = &member.summary {
                let _ = writeln!(out, "    summary: {summary}");
            }
        }
        let _ = writeln!(out);

        let _ = writeln!(out, "## What the graph already declares");
        let _ = writeln!(out);
        match self.declared.is_empty() {
            true => {
                let _ = writeln!(
                    out,
                    "Nothing, between two members of this slice. Every relation you propose is new."
                );
            }
            false => {
                let _ = writeln!(
                    out,
                    "These {} edges are already known. A finding that restates one is a defect in \
                     the sweep rather than a fact about the corpus, and the intake refuses it.",
                    self.declared.len()
                );
                let _ = writeln!(out);
                for edge in &self.declared {
                    let _ = writeln!(out, "- {} {} {}", edge.from, edge.relation, edge.to);
                }
            }
        }
        let _ = writeln!(out);

        let _ = writeln!(out, "## What to write back");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "One YAML file, then `headwater sweep report <path>`. The intake verifies every \
             quotation against the file it names and refuses a finding it cannot find, so quote \
             and never paraphrase."
        );
        let _ = writeln!(out);
        let _ = writeln!(out, "```yaml");
        let _ = writeln!(out, "taxonomy: {}", self.lock);
        let _ = writeln!(
            out,
            "slice: {}",
            match self.under.is_empty() {
                true => ".",
                false => &self.under,
            }
        );
        let _ = writeln!(out, "findings:");
        let _ = writeln!(out, "  - class: undeclared_conflict");
        let _ = writeln!(out, "    documents:");
        let _ = writeln!(out, "      - <a path from the slice above>");
        let _ = writeln!(out, "      - <a second path, where the class compares two>");
        let _ = writeln!(out, "    evidence:");
        let _ = writeln!(out, "      - path: <one of the paths above>");
        let _ = writeln!(out, "        quote: <the passage, copied>");
        let _ = writeln!(out, "    message: <what you believe, in one sentence>");
        let _ = writeln!(
            out,
            "    proposal:            # optional, and the best outcome"
        );
        let _ = writeln!(out, "      relation: conflicts_with");
        let _ = writeln!(out, "      from: <identifier>");
        let _ = writeln!(out, "      to: <identifier>");
        let _ = writeln!(out, "```");
        let _ = writeln!(out);

        let _ = writeln!(out, "## The classes");
        let _ = writeln!(out);
        for class in crate::Class::ALL {
            let _ = writeln!(out, "- `{}`: {}", class.name(), describe(class));
        }
        out
    }
}

fn describe(class: crate::Class) -> &'static str {
    match class {
        crate::Class::UndeclaredConflict => {
            "two documents contradict each other, both are current, and neither says so"
        }
        crate::Class::QuietSupersession => "a newer document has quietly overtaken an older claim",
        crate::Class::UndefinedConcept => {
            "a term is used across the slice and defined in none of it"
        }
        crate::Class::AudienceMismatch => {
            "the audience the document declares could not act on what it says"
        }
        crate::Class::UnwrittenSection => {
            "a heading the kind requires, over prose that says nothing about it"
        }
    }
}

/// Whether a path is in the slice. An empty prefix takes the whole corpus.
fn within(path: &str, under: &str) -> bool {
    if under.is_empty() || under == "." {
        return true;
    }
    path == under || path.starts_with(&format!("{under}/"))
}

fn scalar(facets: &headwater_yaml::value::Mapping, key: &str) -> Option<String> {
    facets
        .get(key)?
        .value
        .as_scalar()
        .map(|scalar| scalar.text.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_prefix_takes_a_directory_and_never_a_sibling_that_shares_its_name() {
        assert!(within("docs/spec/04.md", "docs/spec"));
        assert!(within("docs/spec", "docs/spec"));
        assert!(!within("docs/specimens/04.md", "docs/spec"));
        assert!(within("docs/anything.md", ""));
    }
}
