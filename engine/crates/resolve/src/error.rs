// SPDX-License-Identifier: Apache-2.0
//! What a resolution refuses, and the text an author reads.
//!
//! Every refusal names the source that carries the operation, because an
//! overlay set is several files and a message that names a path without a file
//! sends its reader to look in the wrong one.

use headwater_yaml::Span;

/// One refusal, with the source and the operation it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolveError {
    pub kind: ResolveErrorKind,
    /// The source this came from, as the caller named it. Empty for a fault of
    /// the result rather than of an operation.
    pub source: String,
    /// The operation, written the way an overlay writes it: `add.kinds.report`.
    pub at: String,
    pub span: Span,
}

impl ResolveError {
    pub fn new(kind: ResolveErrorKind, source: &str, at: &str, span: Span) -> Self {
        Self {
            kind,
            source: source.to_string(),
            at: at.to_string(),
            span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveErrorKind {
    /// The source does not load, or the meta-schema refuses it. The engine
    /// never applies a taxonomy that does not validate, and there is no
    /// partial-load mode ([spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema)).
    SourceRefused(String),
    /// A source that has to be a taxonomy carries operations, or the reverse.
    WrongRole { expected: &'static str },
    /// Two operations from two sources do not commute.
    NotConfluent {
        other_source: String,
        other_at: String,
        path: String,
        why: &'static str,
    },
    /// `add` states the whole value, so the key must not already be there. A
    /// base release that adds a key an overlay already added is a collision,
    /// and a collision is a task for a human rather than a promotion to
    /// `override`.
    AddCollides(String),
    /// `override` keeps every field the consumer did not restate, so the path
    /// must exist.
    OverrideMissing(String),
    /// `remove` deletes a key, so the key must be there to delete.
    RemoveMissing(String),
    /// An address reaches through a node that holds no members.
    ThroughNonMapping { address: String, at: String },
    /// An address reaches through a node that holds a reference. The node an
    /// address names is in the merged tree before any reference resolves.
    ThroughReference { address: String, at: String },
    /// `add_to` and `remove_from` take a list.
    NotAList(String),
    /// `remove_from` names an item the list does not hold.
    ItemNotInList(String),
    /// A reference names a node that the merged tree does not hold.
    ReferenceUnresolved(String),
    /// A reference reads a second reference. A chain admits a cycle and no use
    /// needs one.
    ReferenceChain { reference: String, reads: String },
    /// A reference reads a root that this engine cannot supply yet.
    ReferenceRootUnavailable { reference: String, why: String },
    /// A `remove` left a reference with nothing to read.
    RemoveBreaksReference { removed: String, reference: String },
    /// A rule of `taxonomy validate` that reads the resolved taxonomy refused
    /// it. See [`crate::rules`].
    Invalid {
        rule: &'static str,
        message: String,
    },
    /// The result fails a core requirement.
    CoreUnsatisfied {
        requirement: String,
        /// The operation that removed the last satisfier, when one did.
        blame: Option<String>,
    },
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ResolveErrorKind::*;
        match &self.kind {
            SourceRefused(text) => write!(f, "{text}"),
            WrongRole { expected } => write!(f, "this source is not {expected}"),
            NotConfluent {
                other_source,
                other_at,
                path,
                why,
            } => write!(
                f,
                "does not commute with `{other_at}` in {other_source}: both reach `{path}`, and {why}"
            ),
            AddCollides(address) => write!(
                f,
                "`{address}` is already declared, and `add` states a whole value. \
                 Reconcile the two declarations by hand: `add` and `override` differ in what \
                 the consumer inherits, so nothing promotes one to the other"
            ),
            OverrideMissing(address) => {
                write!(f, "`{address}` is not declared, and `override` replaces a value that is there")
            }
            RemoveMissing(address) => {
                write!(f, "`{address}` is not declared, so there is nothing to remove")
            }
            ThroughNonMapping { address, at } => write!(
                f,
                "`{address}` reaches through `{at}`, which holds no members"
            ),
            ThroughReference { address, at } => write!(
                f,
                "`{address}` reaches through `{at}`, which holds a reference. \
                 An address names a node of the merged tree, and a reference resolves after \
                 every overlay is applied"
            ),
            NotAList(address) => write!(f, "`{address}` is not a list, and lists never silently merge"),
            ItemNotInList(item) => write!(f, "the list does not hold `{item}`"),
            ReferenceUnresolved(reference) => {
                write!(f, "`{reference}` reads a node that the resolved taxonomy does not declare")
            }
            ReferenceChain { reference, reads } => write!(
                f,
                "`{reference}` reads `{reads}`, which is a second reference. \
                 A reference points at a value"
            ),
            ReferenceRootUnavailable { reference, why } => {
                write!(f, "`{reference}` cannot resolve: {why}")
            }
            RemoveBreaksReference { removed, reference } => write!(
                f,
                "removing `{removed}` leaves `{reference}` with nothing to read"
            ),
            Invalid { rule, message } => write!(f, "{rule}: {message}"),
            CoreUnsatisfied { requirement, blame } => match blame {
                Some(operation) => write!(
                    f,
                    "the core requires {requirement}, and {operation} removed its last satisfier"
                ),
                None => write!(f, "the core requires {requirement}, and nothing satisfies it"),
            },
        }
    }
}

/// Every refusal, in the order the resolver found them.
pub fn render(errors: &[ResolveError]) -> String {
    let mut out = String::new();
    for error in errors {
        let mut head = String::new();
        if !error.source.is_empty() {
            head.push_str(&error.source);
        }
        if !error.at.is_empty() {
            if !head.is_empty() {
                head.push(' ');
            }
            head.push_str(&format!("`{}`", error.at));
        }
        if error.span != Span::default() {
            if !head.is_empty() {
                head.push(' ');
            }
            head.push_str(&format!("({})", error.span.start));
        }
        if head.is_empty() {
            out.push_str(&format!("{error}\n"));
        } else {
            out.push_str(&format!("{head}: {error}\n"));
        }
    }
    out
}
