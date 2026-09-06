//! Positive control: an Edge-scoped check sees both endpoints and may ask the
//! one scoped question about the wider graph, `reciprocal_exists`.

use headwater_core::check::{CheckMeta, EdgeCheck, Registry};
use headwater_core::finding::{Finding, Severity};
use headwater_core::view::EdgeView;

struct MustReciprocate;

impl CheckMeta for MustReciprocate {
    fn id(&self) -> &'static str {
        "must_reciprocate"
    }
    fn obligation(&self) -> &'static str {
        "OB-GRAPH-1"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl EdgeCheck for MustReciprocate {
    fn evaluate(&self, view: &EdgeView<'_>) -> Vec<Finding> {
        if view.reciprocal_exists() {
            return Vec::new();
        }
        let Some(to) = view.to() else {
            return Vec::new();
        };
        vec![Finding {
            check: "must_reciprocate",
            obligation: "OB-GRAPH-1",
            severity: Severity::Error,
            path: view.from().path().to_string(),
            span: view.span(),
            message: format!("no reciprocal from {}", to.id()),
            remediation: "add the reciprocal".into(),
            fix: None,
        }]
    }
}

fn main() {
    let r = Registry::new().with_edge(MustReciprocate);
    assert_eq!(r.count(), 1);
}
