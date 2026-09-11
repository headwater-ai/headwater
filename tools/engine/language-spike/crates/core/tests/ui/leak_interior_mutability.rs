//! Leak 6: route around `&self` with interior mutability.
//!
//! This is the one that would race. The runner evaluates document checks on
//! several threads over one shared graph, so a `RefCell` in a check is a data
//! race that surfaces as findings that differ between runs — and determinism is
//! what `generate --check` and the whole cache rest on.
//!
//! `DocumentCheck: Send + Sync` rejects it where it is written, rather than
//! under load.

use headwater_core::check::{CheckMeta, DocumentCheck};
use headwater_core::finding::{Finding, Severity};
use headwater_core::view::DocumentView;
use std::cell::RefCell;

struct Accumulating {
    seen: RefCell<Vec<String>>,
}

impl CheckMeta for Accumulating {
    fn id(&self) -> &'static str {
        "accumulating"
    }
    fn obligation(&self) -> &'static str {
        "OB-X"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
}

impl DocumentCheck for Accumulating {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding> {
        self.seen.borrow_mut().push(view.path().to_string());
        Vec::new()
    }
}

fn main() {}
