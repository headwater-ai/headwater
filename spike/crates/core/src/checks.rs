//! Concrete checks, one per origin the spike exercises.
//!
//! These stand in for the generated shape and graph checks of spec 12. Each one
//! is written to put a *span* on its finding, because item 1 is only retired if
//! positions reach the output of a real check rather than a parser test.

use crate::check::{CheckMeta, CorpusCheck, DocumentCheck, EdgeCheck};
use crate::finding::{Finding, Fix, Severity};
use crate::frontmatter::Value;
use crate::view::{CorpusView, DocumentView, EdgeView};
use std::collections::HashMap;

/// Shape origin: a facet the taxonomy declares required.
pub struct FacetRequired {
    pub facet: &'static str,
}

impl CheckMeta for FacetRequired {
    fn id(&self) -> &'static str {
        "facet_required"
    }
    fn obligation(&self) -> &'static str {
        "OB-SHAPE-1"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl DocumentCheck for FacetRequired {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding> {
        match view.facet(self.facet) {
            None => vec![Finding {
                check: "facet_required",
                obligation: "OB-SHAPE-1",
                severity: Severity::Error,
                path: view.path().to_string(),
                // A missing key has no span, so the finding anchors to the
                // block. This is the case that a naive implementation gets
                // wrong by reporting line 0.
                span: view.front_span(),
                message: format!("required facet `{}` is absent", self.facet),
                remediation: format!("add `{}:` to the front matter", self.facet),
                fix: None,
            }],
            Some(Value::Null) => vec![Finding {
                check: "facet_required",
                obligation: "OB-SHAPE-1",
                severity: Severity::Error,
                path: view.path().to_string(),
                // Present but empty: the finding points at the key the author
                // actually wrote.
                span: view.facet_span(self.facet).unwrap_or(view.front_span()),
                message: format!("required facet `{}` is empty", self.facet),
                remediation: format!("give `{}` a value", self.facet),
                fix: None,
            }],
            Some(_) => Vec::new(),
        }
    }
}

/// Shape origin: enum membership.
pub struct StatusEnum {
    pub allowed: &'static [&'static str],
}

impl CheckMeta for StatusEnum {
    fn id(&self) -> &'static str {
        "status_enum"
    }
    fn obligation(&self) -> &'static str {
        "OB-SHAPE-2"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl DocumentCheck for StatusEnum {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding> {
        let Some(Value::Scalar(s)) = view.facet("status") else {
            return Vec::new();
        };
        if self.allowed.contains(&s.as_str()) {
            return Vec::new();
        }
        // A single-candidate near miss is mechanical and total, so it may carry
        // a fix. Anything ambiguous carries prose instead.
        let near: Vec<&&str> = self
            .allowed
            .iter()
            .filter(|a| a.starts_with(s.chars().next().unwrap_or('\0')))
            .collect();
        let fix = if near.len() == 1 {
            Some(Fix {
                description: format!("replace with `{}`", near[0]),
                replacement: near[0].to_string(),
            })
        } else {
            None
        };
        vec![Finding {
            check: "status_enum",
            obligation: "OB-SHAPE-2",
            severity: Severity::Error,
            path: view.path().to_string(),
            span: view.facet_span("status").unwrap_or(view.front_span()),
            message: format!("`{}` is not a declared status", s),
            remediation: format!("use one of: {}", self.allowed.join(", ")),
            fix,
        }]
    }
}

/// Document origin: a regime applied to the body, which is not in the graph.
pub struct HeadingRequired {
    pub heading: &'static str,
}

impl CheckMeta for HeadingRequired {
    fn id(&self) -> &'static str {
        "section_contract"
    }
    fn obligation(&self) -> &'static str {
        "OB-DOC-1"
    }
    fn severity(&self) -> Severity {
        Severity::Warning
    }
}

impl DocumentCheck for HeadingRequired {
    fn evaluate(&self, view: &DocumentView<'_>) -> Vec<Finding> {
        if view.headings().iter().any(|h| h.value == self.heading) {
            return Vec::new();
        }
        vec![Finding {
            check: "section_contract",
            obligation: "OB-DOC-1",
            severity: Severity::Warning,
            path: view.path().to_string(),
            span: view
                .headings()
                .first()
                .map(|h| h.span)
                .unwrap_or(view.front_span()),
            message: format!("no `{}` section", self.heading),
            remediation: format!("add a `## {}` section", self.heading),
            fix: None,
        }]
    }
}

/// Graph origin: the constraint that defeated LinkML.
pub struct Reciprocity;

impl CheckMeta for Reciprocity {
    fn id(&self) -> &'static str {
        "reciprocity"
    }
    fn obligation(&self) -> &'static str {
        "OB-GRAPH-1"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl EdgeCheck for Reciprocity {
    fn evaluate(&self, view: &EdgeView<'_>) -> Vec<Finding> {
        if let Some(missing) = view.dangling_target() {
            return vec![Finding {
                check: "reciprocity",
                obligation: "OB-GRAPH-1",
                severity: Severity::Error,
                path: view.from().path().to_string(),
                span: view.span(),
                message: format!("`{}` points at unknown `{}`", view.relation().facet(), missing),
                remediation: "correct the identifier or add the target document".into(),
                fix: None,
            }];
        }
        if view.reciprocal_exists() {
            return Vec::new();
        }
        let Some(to) = view.to() else {
            return Vec::new();
        };
        let inverse = match view.relation().inverse() {
            Some(i) => i,
            None => return Vec::new(),
        };
        vec![Finding {
            check: "reciprocity",
            obligation: "OB-GRAPH-1",
            severity: Severity::Error,
            path: view.from().path().to_string(),
            span: view.span(),
            message: format!(
                "`{}` to `{}` has no reciprocal `{}`",
                view.relation().facet(),
                to.id(),
                inverse.facet()
            ),
            remediation: format!(
                "add `{}: [{}]` to {}",
                inverse.facet(),
                view.from().id(),
                to.path()
            ),
            // Mechanical and total: one correct outcome, derivable without
            // judgment. Spec 12's rule for when a fix may be offered.
            fix: Some(Fix {
                description: format!("add reciprocal `{}` to {}", inverse.facet(), to.path()),
                replacement: format!("{}: [{}]", inverse.facet(), view.from().id()),
            }),
        }]
    }
}

/// Corpus origin: needs every document at once, and is therefore the barrier.
pub struct IdUnique;

impl CheckMeta for IdUnique {
    fn id(&self) -> &'static str {
        "id_unique"
    }
    fn obligation(&self) -> &'static str {
        "OB-CORPUS-1"
    }
    fn severity(&self) -> Severity {
        Severity::Error
    }
}

impl CorpusCheck for IdUnique {
    fn evaluate(&self, view: &CorpusView<'_>) -> Vec<Finding> {
        let mut seen: HashMap<String, String> = HashMap::new();
        let mut out = Vec::new();
        for d in view.documents() {
            if let Some(first) = seen.get(d.id()) {
                out.push(Finding {
                    check: "id_unique",
                    obligation: "OB-CORPUS-1",
                    severity: Severity::Error,
                    path: d.path().to_string(),
                    span: d.facet_span("id").unwrap_or(d.front_span()),
                    message: format!("identifier `{}` is already used by {}", d.id(), first),
                    remediation: "give this document a unique identifier".into(),
                    fix: None,
                });
            } else {
                seen.insert(d.id().to_string(), d.path().to_string());
            }
        }
        out
    }
}
