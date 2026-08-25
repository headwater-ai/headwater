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

/// The two declarations an `add` collision names.
///
/// [Spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)
/// makes a collision "always a task for a human", and a task a person can act
/// on shows both sides. The values are rendered in the canonical form
/// [`crate::render`] writes the lock in, so what a reader compares is the text
/// the two sources resolve to rather than the bytes their authors typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collision {
    /// The address both sources declare.
    pub address: String,
    /// The file that carries the declaration already there. Empty where the
    /// raise knew the value and not who wrote it, which is [`crate::merge`]'s
    /// own leaf-grained raise: the tree it walks holds no source index.
    pub declared_in: String,
    /// That declaration, canonically rendered, with no trailing newline.
    pub declared: String,
    /// The value this `add` states, in the same form.
    pub adds: String,
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
    AddCollides(Collision),
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
    Invalid { rule: &'static str, message: String },
    /// The result fails a core requirement.
    CoreUnsatisfied {
        requirement: String,
        /// The operation that removed the last satisfier, when one did.
        blame: Option<String>,
    },
}

impl ResolveErrorKind {
    /// Whether this refusal is about an address that an overlay wrote.
    ///
    /// Spec 2's [`addressability`](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
    /// dimension asks whether every path an overlay can address still exists
    /// and means the same thing, and it is the one dimension whose subject is
    /// the schema rather than a corpus. This is the engine's answer, and it is
    /// here rather than at the caller because the arms are this enum's: a new
    /// refusal that names an address has to state which side of the line it is
    /// on, and the match below has no wildcard, so it will not compile until
    /// somebody says.
    ///
    /// A reference refusal is not one of these. An overlay addresses a node of
    /// the merged tree, and a reference resolves afterwards, so a reference
    /// that no longer reads is a fault of the base rather than of an address
    /// the consumer wrote.
    pub fn names_an_address(&self) -> bool {
        use ResolveErrorKind::*;
        match self {
            AddCollides(_)
            | OverrideMissing(_)
            | RemoveMissing(_)
            | ThroughNonMapping { .. }
            | ThroughReference { .. }
            | NotAList(_)
            | ItemNotInList(_) => true,
            SourceRefused(_)
            | WrongRole { .. }
            | NotConfluent { .. }
            | ReferenceUnresolved(_)
            | ReferenceChain { .. }
            | ReferenceRootUnavailable { .. }
            | RemoveBreaksReference { .. }
            | Invalid { .. }
            | CoreUnsatisfied { .. } => false,
        }
    }
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
            AddCollides(both) => write!(
                f,
                "`{}` is already declared{}, and `add` states a whole value. \
                 Reconcile the two declarations by hand: `add` and `override` differ in what \
                 the consumer inherits, so nothing promotes one to the other",
                both.address,
                match both.declared_in.is_empty() {
                    true => String::new(),
                    false => format!(" in {}", both.declared_in),
                }
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

/// Every `add` collision of one refused resolution, as the judgment task
/// [spec 2](../../../../docs/spec/02-taxonomy-model.md#customization-by-composition)
/// owes a consumer: both declarations, both files, the address, and the two
/// operations that settle it.
///
/// Empty where no refusal is a collision, so a caller prints this or prints
/// nothing and needs no second question. `apply` returns on the first refusal,
/// so one run produces at most one — the plural is here because the type
/// permits a list and a renderer that assumed one would be a claim about the
/// caller.
///
/// The two declarations are compared as rendered text rather than through
/// [`crate::merge::same`], because the rendered text is what the reader
/// compares, and a report that called two texts the same over two a reader can
/// see differ would be worse than one that said nothing.
pub fn collisions(errors: &[ResolveError]) -> String {
    let found: Vec<(&ResolveError, &Collision)> = errors
        .iter()
        .filter_map(|error| match &error.kind {
            ResolveErrorKind::AddCollides(both) => Some((error, both)),
            _ => None,
        })
        .collect();
    if found.is_empty() {
        return String::new();
    }

    let mut out = format!(
        "\n{} `add` collision{}. Spec 2 makes a collision a task for a person, and no run \
         settles one\n",
        found.len(),
        if found.len() == 1 { "" } else { "s" }
    );
    for (error, both) in found {
        out.push('\n');
        out.push_str(&format!("  {}\n", both.address));
        out.push_str(&match both.declared_in.is_empty() {
            true => String::from("    a declaration already in the merged tree declares it\n"),
            false => format!("    {} declares it\n", both.declared_in),
        });
        out.push_str(&block(&both.declared));
        out.push_str(&format!("    {} adds it, at {}\n", error.source, error.at));
        out.push_str(&block(&both.adds));
        out.push_str(&match both.declared == both.adds {
            true => String::from(
                "    the two declarations are the same text, so removing this `add` changes \
                 nothing the corpus reads\n",
            ),
            false => String::from(
                "    the two declarations differ, so what this repository inherits is a choice\n",
            ),
        });
        out.push_str(REMEDY);
    }
    out
}

/// The remedy of every collision, which is the same remedy every time.
///
/// Spec 2 admits two operations at a declared address, and which of the two a
/// consumer wants is the judgment. The last sentence names the route the
/// engine refuses, because it is the route a reader tries first;
/// `fixtures/cases/add-collides-behind-remove/` is what holds that claim.
const REMEDY: &str = concat!(
    "    task  `remove` this `add` from the overlay and inherit the base declaration whole,\n",
    "          or restate it as an `override`, which keeps every field of the base\n",
    "          declaration this overlay does not restate. Those are the two. A `remove`\n",
    "          with the same `add` written again behind it is not a third: an `add`\n",
    "          asserts its precondition about the base, and a `remove` in the same\n",
    "          overlay does not move the base\n",
);

/// A rendered value, six spaces in. An empty value is a side with nothing on
/// it, and a blank block would read as a value that is the empty string.
fn block(value: &str) -> String {
    if value.is_empty() {
        return String::from("      (no value)\n");
    }
    value
        .lines()
        .map(|line| format!("      {line}\n"))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn collision(declared: &str, adds: &str) -> ResolveError {
        ResolveError::new(
            ResolveErrorKind::AddCollides(Collision {
                address: "identifier_schemes.spec_id".to_string(),
                declared_in: "released/2.0.0/taxonomy.yml".to_string(),
                declared: declared.to_string(),
                adds: adds.to_string(),
            }),
            ".headwater/overlay.yml",
            "add.identifier_schemes.spec_id",
            Span::default(),
        )
    }

    #[test]
    fn a_refusal_that_is_not_a_collision_produces_no_section_at_all() {
        let error = ResolveError::new(
            ResolveErrorKind::RemoveMissing("kinds.report".to_string()),
            ".headwater/overlay.yml",
            "remove.kinds.report",
            Span::default(),
        );
        assert_eq!(collisions(&[error]), "");
    }

    /// Each declaration under the label of the source that wrote it.
    ///
    /// The two values a report puts side by side are the only part of it a
    /// reader cannot get anywhere else, and a report that named them both
    /// under one label would read as correct while being useless.
    #[test]
    fn each_declaration_prints_under_the_source_that_wrote_it() {
        let report = collisions(&[collision(
            "namespace: HW\nallocation: reconcile-first",
            "{}",
        )]);
        let (base_half, overlay_half) = report
            .split_once(".headwater/overlay.yml adds it")
            .expect("the report names the overlay side");
        assert!(base_half.contains("released/2.0.0/taxonomy.yml declares it"));
        assert!(base_half.contains("      allocation: reconcile-first\n"));
        assert!(!base_half.contains("{}"));
        assert!(overlay_half.contains("      {}\n"));
        assert!(overlay_half.contains("at add.identifier_schemes.spec_id"));
    }

    /// A base that grew exactly the declaration the overlay adds is the one
    /// collision that costs nothing to settle, and a report that called it a
    /// choice would send a reader to weigh two identical texts.
    #[test]
    fn a_collision_between_two_identical_declarations_is_not_reported_as_a_choice() {
        let report = collisions(&[collision("purpose: behavior", "purpose: behavior")]);
        assert!(
            report.contains("changes nothing the corpus reads"),
            "{report}"
        );
        assert!(!report.contains("is a choice"), "{report}");
    }

    /// An `add` with no value is a source the meta-schema refuses, so this arm
    /// is unreachable through `resolve`. It prints a word rather than a blank
    /// run of six spaces, which a reader would read as an empty string value.
    #[test]
    fn a_side_with_nothing_on_it_says_so() {
        let report = collisions(&[collision("purpose: behavior", "")]);
        assert!(report.contains("      (no value)\n"), "{report}");
    }

    /// The remedy names two operations, and names the third route as refused.
    /// `fixtures/cases/add-collides-behind-remove/` is the case that holds it.
    #[test]
    fn the_remedy_names_two_operations_and_the_route_that_is_not_a_third() {
        let report = collisions(&[collision("purpose: behavior", "purpose: rationale")]);
        assert!(
            report.contains("inherit the base declaration whole"),
            "{report}"
        );
        assert!(report.contains("restate it as an `override`"), "{report}");
        assert!(report.contains("is not a third"), "{report}");
    }
}
