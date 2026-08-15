// SPDX-License-Identifier: Apache-2.0
//! The overlay resolver: a base package and its overlays merged into one
//! validated taxonomy.
//!
//! [Spec 2](../../../docs/spec/02-taxonomy-model.md#customization-by-composition):
//! "A taxonomy is one logical document. The engine assembles it from a base
//! package plus zero or more overlays, and it resolves to a single validated
//! object." This crate is that assembly.
//!
//! [Spec 12](../../../docs/spec/12-check-layer.md#the-correctness-roots) puts
//! it first on the list of components that every check trusts silently: "A
//! resolver bug corrupts every check, projection, and conformance claim at
//! once. A committed and diffable lock decreases the risk but does not test the
//! resolver. The resolver carries its own round-trip and confluence fixtures."
//! Those fixtures are `fixtures/`, and they are the deliverable rather than the
//! decoration.
//!
//! # The order, and why it is not the obvious one
//!
//! 1. **Validate every source.** The engine never applies a taxonomy that does
//!    not validate, and there is no partial-load mode.
//! 2. **Check confluence, statically.** Before anything merges, so that the
//!    answer does not depend on the order the resolver happened to pick, and so
//!    that a publisher can prove ahead of a release that every subset of its
//!    bundles resolves.
//! 3. **Apply every operation.** Each one asserts a precondition about the tree
//!    it meets, and a failed precondition is an error rather than a silent
//!    promotion to another operation.
//! 4. **Resolve every reference.** Last, over the merged tree. An overlay that
//!    overrides `vocabularies.lifecycle_state` expects the facet that reads it
//!    to follow, and an early resolution would freeze the base list before any
//!    overlay was read.
//! 5. **Check the core, on the result.** Never on an operation. An overlay is
//!    rejected when the result fails a requirement, and the message names the
//!    operation that removed the last satisfier.
//!
//! # Resolving and validating are two operations, and the lock is what joins them
//!
//! [`resolve`] is the five steps above, which is what
//! [spec 6](../../../docs/spec/06-engine-architecture.md#pipeline) gives the
//! verb: "merges the base taxonomy and overlays, validates against the
//! meta-schema, and writes a content-hashed lock". [`Resolution::validate`] is
//! the rest of spec 2's list — the rules that read the result and report on it
//! rather than decide a merge. [`rules::RULES`] is the accounting of both.
//!
//! Spec 2 also rules that "the engine never applies a taxonomy that does not
//! validate" and that there is no partial-load mode. That rule is enforced by
//! the artifact rather than by the call graph: `headwater taxonomy resolve`
//! writes a lock only when every rule passes, and everything downstream reads
//! the lock and never the sources. A caller cannot forget to validate, because
//! a caller has nothing else to read.
//!
//! # What this replaced
//!
//! Two stand-ins, in two languages, both add-only and both labeled temporary in
//! their own source: `headwater_census::standin` and `tools/abox-check.py`.
//! They agreed with each other on 36 typed documents and 203 edge halves over
//! this repository, and that agreement is kept as a regression: the recorded
//! censuses and graphs did not move when the resolver replaced them. The
//! agreement was never evidence that either one was right, because both read
//! three files through one add-only shape and neither resolved a reference or
//! merged an address deeper than a member.
//!
//! # Example
//!
//! ```no_run
//! let repository = headwater_resolve::repository(std::path::Path::new(".."))
//!     .expect("this repository resolves");
//! assert_eq!(repository.consumer.corpus_root, "docs");
//! ```

pub mod confluence;
pub mod core;
pub mod error;
pub mod merge;
pub mod migration;
pub mod operation;
pub mod package;
pub mod references;
pub mod release;
pub mod render;
pub mod rules;
pub mod source;

pub use error::{render as render_errors, ResolveError, ResolveErrorKind};
pub use operation::{OpKind, Operation};
pub use package::{Adopted, Consumer};
pub use source::{Role, Source};

use headwater_meta::MetaSchema;
use headwater_yaml::{Mapping, Span, Value};
use std::path::Path;

/// Where every rule of `taxonomy validate` runs. See [`rules::RULES`], which is
/// the table, and [`rules::WAITING`], which is what no phase decides yet.
pub use rules::{Ran, RULES, WAITING};

/// One resolution.
#[derive(Clone, Debug)]
pub struct Resolution {
    /// The resolved taxonomy: one mapping, references resolved, core satisfied.
    pub taxonomy: Mapping,
    /// The sources that produced it, in application order.
    pub sources: Vec<String>,
    /// Every operation applied, in application order.
    pub operations: Vec<Operation>,
    /// Every operation that made the declaration it addresses into, rather
    /// than reaching into one that was there. See [`Founding`].
    pub founded: Vec<Founding>,
}

/// An operation that creates the declaration its address reaches into.
///
/// An overlay addresses a path: `kinds.decision.facets` writes `facets` into
/// the `decision` kind, and the author of that line believed a `decision` kind
/// was there to write into. Nothing in the merge requires one. [`apply`] grafts
/// through a missing key by creating it, so the same line against a base that
/// declares no `decision` kind *makes* one, holding two keys and nothing else.
///
/// # Why it is a reading rather than a refusal
///
/// The state it names is the one an upgrade produces and nothing reports.
/// [Spec 2](../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility)
/// measures `addressability` by whether the adopter's overlay still resolves,
/// and this case resolves: a base that renames `decision` to `ruling` leaves
/// the overlay addressing a path that now has no base under it, and the
/// operation puts the old name back. The resolver reports nothing, the
/// candidate resolves, and the corpus is left holding a declaration its
/// taxonomy removed.
///
/// So the resolver records it, `taxonomy diff` reads it as the `addressability`
/// dimension, and no verb refuses on it yet.
/// [#191](https://github.com/headwater-ai/headwater/issues/191) carries whether
/// `taxonomy validate` should.
///
/// # It is derived from the merge and not from a second walk
///
/// The value is computed inside [`apply`], against the tree each operation is
/// applied to at its own position in the application order. A second walk over
/// the sources could disagree with the merge about which key was there, and the
/// two answers would then differ on exactly the case this type is about: an
/// address whose parent one earlier overlay supplies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Founding {
    /// Index into [`Resolution::sources`].
    pub source: usize,
    /// The operation as an overlay writes it: `add.kinds.decision.facets`.
    pub at: String,
    /// The address the operation names.
    pub address: String,
    /// The shallowest path this operation creates on the way to its address.
    /// The operation makes this key and every key between it and the address.
    pub founds: String,
    pub span: Span,
}

impl Founding {
    /// One line a reader acts on, without the source, which a caller prints.
    pub fn sentence(&self) -> String {
        format!(
            "{} makes `{}` rather than reaching into it, so the address resolves because this \
             operation creates what it addresses",
            self.at, self.founds
        )
    }
}

impl Resolution {
    /// The resolved taxonomy as a source that loads to the same tree.
    ///
    /// This is also the text that the lock hashes, and the two are the same text
    /// on purpose. See [`crate::render`].
    pub fn render(&self) -> String {
        render::render(&self.taxonomy)
    }

    /// Every rule of `taxonomy validate` that reads the result.
    ///
    /// It is separate from [`resolve`] because the five steps of a resolution
    /// decide a merge and these rules report on a taxonomy. A merge fixture
    /// resolves a base cut down to the declarations one case needs, and holding
    /// that to purpose completeness would bury each case in the declarations it
    /// is not about. What holds the guarantee instead is the lock: nothing
    /// downstream reads a source, and a lock is written only when this returns
    /// nothing.
    pub fn validate(&self) -> Vec<ResolveError> {
        rules::check(&self.taxonomy)
    }
}

/// A repository resolved: what it walks, and the taxonomy it walks it with.
#[derive(Clone, Debug)]
pub struct Repository {
    pub consumer: Consumer,
    pub resolution: Resolution,
}

/// Resolve a repository from its consumer declaration.
pub fn repository(root: &Path) -> Result<Repository, Vec<ResolveError>> {
    let consumer = package::consumer(root)?;
    let sources = package::sources(root, &consumer)?;
    let resolution = resolve(&sources)?;
    Ok(Repository {
        consumer,
        resolution,
    })
}

/// Resolve a base package and its overlays.
///
/// The first source is the taxonomy, and every source after it is an overlay,
/// in the order the consumer selected them. The order is fixed so that one
/// resolution produces one lock, and the confluence check is what makes the
/// choice of order a formality rather than a decision.
pub fn resolve(sources: &[Source]) -> Result<Resolution, Vec<ResolveError>> {
    let schema = MetaSchema::shipped().map_err(|error| {
        vec![ResolveError::new(
            ResolveErrorKind::SourceRefused(error.to_string()),
            "the shipped meta-schema",
            "",
            Span::default(),
        )]
    })?;

    let Some((base, overlays)) = sources.split_first() else {
        return Err(vec![ResolveError::new(
            ResolveErrorKind::WrongRole {
                expected: "a base package, and a resolution needs one",
            },
            "",
            "",
            Span::default(),
        )]);
    };
    if base.role != Role::Taxonomy {
        return Err(vec![ResolveError::new(
            ResolveErrorKind::WrongRole {
                expected: "a taxonomy source, which is what a base package is",
            },
            &base.name,
            "",
            base.root.span,
        )]);
    }

    // 1. Every source, against the meta-schema.
    let mut errors: Vec<ResolveError> = sources
        .iter()
        .flat_map(|source| source.validate(&schema))
        .collect();
    if !errors.is_empty() {
        return Err(errors);
    }

    let names: Vec<String> = sources.iter().map(|source| source.name.clone()).collect();
    let mut operations = Vec::new();
    for (index, overlay) in overlays.iter().enumerate() {
        match operation::read(index + 1, &overlay.name, &overlay.root) {
            Ok(read) => operations.extend(read),
            Err(found) => errors.extend(found),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    // 2. Confluence, before anything merges.
    let errors = confluence::check(&operations, &names);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 3. Apply.
    let start = base
        .root
        .value
        .as_map()
        .cloned()
        .expect("a validated taxonomy source is a mapping");
    let (applied, founded) = apply(&start, &operations, &names)?;
    let merged = merge::canonical(Some(&start), &applied);

    // A `remove` that left a reference reading nothing is reported against the
    // removal, because that is the edit a reader has to reconsider.
    let broken: Vec<ResolveError> = references::dangling(&merged, &schema)
        .into_iter()
        .map(|found| match culprit(&operations, &found.wanted) {
            Some(operation) => ResolveError::new(
                ResolveErrorKind::RemoveBreaksReference {
                    removed: operation.address.to_string(),
                    reference: found.reference.clone(),
                },
                &names[operation.source],
                &operation.at(),
                operation.span,
            ),
            None => ResolveError::new(
                ResolveErrorKind::ReferenceUnresolved(found.reference.clone()),
                "",
                &found.at,
                found.span,
            ),
        })
        .collect();
    if !broken.is_empty() {
        return Err(broken);
    }

    // 4. Resolve every reference, over the merged tree.
    let taxonomy = references::resolve(&merged, &schema)?;

    // The result is a taxonomy source, so it is held to the same shape its
    // sources were. A merge that produced something the meta-schema refuses is
    // this crate's defect and not an author's, and it is caught here rather
    // than by the first check that reads a declaration.
    //
    // It is also the one place a required member can go missing: every source
    // declares one and a `remove` takes it out of the result. So this is what
    // decides `edge provenance`, and `rules::RULES` records that rather than
    // carrying a rule that could never fire.
    let after = Source {
        name: "the resolved taxonomy".to_string(),
        role: Role::Taxonomy,
        root: headwater_yaml::Spanned::new(Value::Map(taxonomy.clone()), Span::default()),
        text: String::new(),
    };
    let errors = after.validate(&schema);
    if !errors.is_empty() {
        return Err(errors);
    }

    // 5. The core, on the result.
    let errors = core::check(&start, &taxonomy, &operations, &names);
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(Resolution {
        taxonomy,
        sources: names,
        operations,
        founded,
    })
}

fn apply(
    start: &Mapping,
    operations: &[Operation],
    names: &[String],
) -> Result<(Mapping, Vec<Founding>), Vec<ResolveError>> {
    let mut tree = start.clone();
    let mut founded = Vec::new();
    for operation in operations {
        if let Some(founds) = makes(&tree, operation) {
            founded.push(Founding {
                source: operation.source,
                at: operation.at(),
                address: operation.address.to_string(),
                founds,
                span: operation.span,
            });
        }
        let path = operation.address.segments();
        let full = operation.address.to_string();
        // An `add` asserts its precondition about the base and not about
        // whatever another overlay has already put in the tree. See
        // [`merge`], which is where the reason is.
        if operation.kind == OpKind::Add && merge::lookup(start, path).is_some() {
            return Err(vec![ResolveError::new(
                ResolveErrorKind::AddCollides(full),
                &names[operation.source],
                &operation.at(),
                operation.span,
            )]);
        }
        let result = match (operation.kind, &operation.value) {
            (OpKind::Add, Some(value)) => merge::graft_leaves(&tree, path, value, &full),
            (OpKind::Override, Some(value)) => {
                merge::graft(&tree, path, value, merge::Mode::Override, &full)
            }
            (OpKind::AddTo, Some(value)) => merge::add_to(&tree, path, value, &full),
            (OpKind::RemoveFrom, Some(value)) => merge::remove_from(&tree, path, value, &full),
            (OpKind::Remove, _) => merge::prune(&tree, path, &full),
            (_, None) => continue,
        };
        match result {
            Ok(next) => tree = next,
            Err(kind) => {
                return Err(vec![ResolveError::new(
                    kind,
                    &names[operation.source],
                    &operation.at(),
                    operation.span,
                )])
            }
        }
    }
    Ok((tree, founded))
}

/// The shallowest key on the way to an operation's address that the tree does
/// not hold, or `None` where every key above the address is there.
///
/// Read against the tree the operation is about to be applied to, which is what
/// makes the answer the merge's own. An address of one segment names a whole
/// block and reaches into nothing, so it never answers.
///
/// A `remove` is excluded, and it is the one operation for which a missing key
/// above the address is already reported: `merge::prune` refuses a path it
/// cannot walk. Everything else grafts a missing key into existence.
fn makes(tree: &Mapping, operation: &Operation) -> Option<String> {
    if operation.kind == OpKind::Remove {
        return None;
    }
    let above = operation.reaches_into();
    (1..=above.len())
        .find(|depth| merge::lookup(tree, &above[..*depth]).is_none())
        .map(|depth| above[..depth].join("."))
}

/// The `remove` whose address covers a path, if one does.
fn culprit<'a>(operations: &'a [Operation], wanted: &[String]) -> Option<&'a Operation> {
    operations
        .iter()
        .filter(|operation| operation.kind == OpKind::Remove)
        .find(|operation| wanted.starts_with(operation.address.segments()))
}
