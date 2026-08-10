//! Headwater Q1 risk-retirement spike.
//!
//! Not an engine. Four claims from `docs/evaluations/language-choice.md`, built
//! far enough to pass or fail:
//!
//! 1. A front-matter parse that reports line and column for every key, and a
//!    finding that carries the position to output. (`frontmatter`, `span`)
//! 2. A scope leak that fails to compile. (`view`, `check`, `tests/ui`)
//! 3. One crate built as a native Node addon and as `wasm32`. (`crates/node`,
//!    `crates/wasm`)
//! 4. A warm change-scoped run over 1,000 documents under 200 ms. (`runner`)

pub mod check;
pub mod checks;
pub mod corpus;
pub mod finding;
pub mod frontmatter;
pub mod hash;
pub mod model;
pub mod runner;
pub mod span;
pub mod view;

#[cfg(test)]
mod spans_test;

use check::Registry;

/// The check set the spike runs, standing in for a resolved taxonomy.
pub fn default_registry() -> Registry {
    Registry::new()
        .with_document(checks::FacetRequired { facet: "owner" })
        .with_document(checks::FacetRequired {
            facet: "last_verified",
        })
        .with_document(checks::StatusEnum {
            allowed: &["draft", "current", "superseded", "retired"],
        })
        .with_document(checks::HeadingRequired {
            heading: "Consequences",
        })
        .with_edge(checks::Reciprocity)
        .with_corpus(checks::IdUnique)
}

/// Check one document in isolation — the call an editor makes for the buffer
/// in front of the user.
///
/// This entry point exists because of item 2, and the connection is worth
/// recording. The Node and wasm wrappers are separate crates, so they are
/// *external* consumers, and `DocumentView::new` is `pub(crate)`. The first
/// draft of the Node wrapper tried to build a view and did not compile.
///
/// That is the enforcement working on its author rather than on a test. The
/// repair is the correct layering: an embedder asks for a task to be performed
/// and receives findings. The view type never crosses the crate boundary, so
/// no embedder can widen a check's scope by constructing a view for it.
pub fn check_document_source(path: &str, source: &str) -> Vec<finding::Finding> {
    let Ok(doc) = corpus::parse_document(path, source) else {
        return Vec::new();
    };
    let registry = default_registry();
    let view = view::DocumentView::new(&doc);
    let mut out = Vec::new();
    for c in &registry.document {
        out.extend(c.evaluate(&view));
    }
    finding::sort(&mut out);
    out
}

/// Parse, build, check. The one call the wrappers in `crates/node` and
/// `crates/wasm` need, so that both embeddings exercise the same code.
pub fn check_sources(files: &[(String, String)], taxonomy_hash: u128) -> runner::RunReport {
    let loaded = corpus::load(files);
    let registry = default_registry();
    let runner = runner::Runner::new(&registry, taxonomy_hash);
    let mut cache = runner::Cache::new();
    let mut report = runner.run_full(&loaded.graph, &mut cache);
    report.documents_in_census = loaded.census.len();
    report
}
