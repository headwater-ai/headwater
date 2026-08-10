//! Leak 1, the headline case: a Document-scoped check tries to read a sibling.
//!
//! There is no accessor to reach one, and — the part that matters — no accessor
//! could be added by accident, because `Document` itself holds no reference to
//! any other document or to the graph (see `model.rs`). The check has nothing
//! to follow.

use headwater_core::check::{CheckMeta, DocumentCheck};
use headwater_core::finding::{Finding, Severity};
use headwater_core::view::DocumentView;

struct PeekAtSiblings;

impl CheckMeta for PeekAtSiblings {
    fn id(&self) -> &'static str {
        "peek"
    }
    fn obligation(&self) -> &'static str {
        "OB-X"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl DocumentCheck for PeekAtSiblings {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding> {
        // Each of these is what an author would reach for first.
        let _ = view.graph();
        let _ = view.siblings();
        let _ = view.corpus();
        Vec::new()
    }
}

fn main() {}
