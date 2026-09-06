//! Positive control: an ordinary Document-scoped check compiles and registers.

use headwater_core::check::{CheckMeta, DocumentCheck, Registry};
use headwater_core::finding::{Finding, Severity};
use headwater_core::view::DocumentView;

struct OwnerPresent;

impl CheckMeta for OwnerPresent {
    fn id(&self) -> &'static str {
        "owner_present"
    }
    fn obligation(&self) -> &'static str {
        "OB-SHAPE-1"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl DocumentCheck for OwnerPresent {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding> {
        if view.facet("owner").is_some() {
            return Vec::new();
        }
        vec![Finding {
            check: "owner_present",
            obligation: "OB-SHAPE-1",
            severity: Severity::Error,
            path: view.path().to_string(),
            span: view.front_span(),
            message: "no owner".into(),
            remediation: "add an owner".into(),
            fix: None,
        }]
    }
}

fn main() {
    let r = Registry::new().with_document(OwnerPresent);
    assert_eq!(r.count(), 1);
}
