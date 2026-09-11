//! Leak 5: accumulate state across instances.
//!
//! Spec 12 requires a check to be pure. `evaluate(&self, ..)` is what makes
//! "no mutation of the check" mechanical: a counter across instances would make
//! results depend on evaluation order, and order is not in any cache key.

use headwater_core::check::{CheckMeta, DocumentCheck};
use headwater_core::finding::{Finding, Severity};
use headwater_core::view::DocumentView;

struct Counting {
    seen: usize,
}

impl CheckMeta for Counting {
    fn id(&self) -> &'static str {
        "counting"
    }
    fn obligation(&self) -> &'static str {
        "OB-X"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
}

impl DocumentCheck for Counting {
    fn evaluate(&self, _view: &DocumentView<'_>) -> Vec<Finding> {
        self.seen += 1;
        Vec::new()
    }
}

fn main() {}
