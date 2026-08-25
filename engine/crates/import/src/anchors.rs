// SPDX-License-Identifier: Apache-2.0
//! The snapshot anchor resolver: an item identity that a committed pin holds.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#behavior-at-the-limits)
//! rules that exactly one resolver owns each anchor kind, and that "a resolver
//! reads repository content or a committed snapshot, and never a live service".
//! `headwater_graph::anchors::SourceTree` is the first half of that sentence.
//! This module is the second, and until it existed every edge that
//! [`crate::plan`] wrote was reported as resolving to nothing
//! ([HW-OBL-0116](../../../../docs/obligations/0116-no-anchor-resolver-reads-a-committed-snapshot-so-every-imported-edge-lands-unresolved.md)).
//!
//! # It looks an identity up, and it never normalizes one
//!
//! This is the whole instrument, and it is the opposite of what the resolver
//! beside it does. `SourceTree` normalizes, because two spellings of one
//! repository path name one file and the rules of that equality are the rules of
//! a path. An upstream identity has no such rules. `12345`, `12345 ` and `#12345`
//! are three strings, the far end decides which of them name one item, and
//! nothing in this repository can know that. A resolver that guessed would bind
//! a typo to a real item, which is worse than binding nothing: an edge that
//! reports as unresolved sends an author to the line, and an edge bound to the
//! wrong item is a correct check result over a wrong graph, which is the failure
//! [spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots)
//! names for an importer.
//!
//! So the whole of the normalization here is [`str::trim`], and it is a trim
//! rather than an equality rule: leading space in a YAML scalar is the format's
//! and not the identity's. Everything after it is a lookup in the item list the
//! snapshot pinned. **An identity the snapshot does not hold is reported.**
//!
//! [`crate::plan`] refuses a link into an item the snapshot does not name, and
//! that refusal and this lookup are not one guard twice. The importer sees the
//! links a snapshot declares. This sees the target string of every edge that is
//! already in the corpus, including one a person typed by hand and one that an
//! earlier import wrote against a snapshot that has since lost the item.
//!
//! # A run binds against pinned bytes or against nothing
//!
//! An import declaration carries `digest` and `channel` because a snapshot is a
//! pin ([Q19](../../../../docs/decisions/0019-inbound-integration-an-external-system-of-record.md)),
//! and [`crate::plan`] refuses to read a directory that nothing pins. The same
//! rule holds here for the same reason. [`open`] runs
//! [`headwater_resolve::release::verify`] before it reads an item, so a snapshot
//! whose bytes moved after the import binds nothing.
//!
//! What it does then is report rather than disappear. A declaration whose
//! snapshot did not open still contributes a resolver, and that resolver refuses
//! every string with the reason. The alternative is an absent resolver, which
//! makes the graph say "`ado_work_item` names the resolver `ado-snapshot`, which
//! this run does not have" over a repository that declares exactly that resolver
//! and has it. The reader would go looking for a missing feature instead of a
//! moved byte.
//!
//! # No socket
//!
//! [`open`] takes a path under the repository root, on the shape the crate
//! comment sets. Nothing here fetches, and there is no code path that could.

use crate::snapshot::Snapshot;
use crate::Declaration;
use headwater_graph::anchors::{Binding, Resolver};
use headwater_resolve::release;
use std::path::Path;

/// A resolver over the items of one committed snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Items {
    /// The name an anchor kind names it by, out of `imports.<name>.resolver`.
    resolver: String,
    /// The upstream system as the snapshot labels itself, for a message.
    source: String,
    /// Every item identity the snapshot pinned, verbatim, with the revision
    /// the snapshot pinned it at.
    ///
    /// The revision travels because the binding carries it and the graph keys
    /// on it. It is what `headwater_check::suspect` compares an edge's
    /// `verified_revision` against, and it is what stops a cached verdict about
    /// that edge outliving the advance that falsifies it.
    items: Vec<(String, String)>,
    /// Why this resolver holds no items. Set where the snapshot did not open,
    /// and every string is then refused with it.
    unavailable: Option<String>,
}

impl Items {
    /// The resolver one verified snapshot supplies.
    pub fn of(resolver: &str, snapshot: &Snapshot) -> Self {
        Items {
            resolver: resolver.to_string(),
            source: snapshot.source.clone(),
            items: snapshot
                .items
                .iter()
                .map(|item| (item.id.clone(), item.revision.clone()))
                .collect(),
            unavailable: None,
        }
    }

    /// A resolver that binds nothing and says why, for a declared import whose
    /// snapshot did not open. The module comment argues why this is not an
    /// absent resolver.
    pub fn unavailable(resolver: &str, why: String) -> Self {
        Items {
            resolver: resolver.to_string(),
            source: String::new(),
            items: Vec::new(),
            unavailable: Some(why),
        }
    }

    /// How many item identities this resolver can bind.
    pub fn held(&self) -> usize {
        self.items.len()
    }
}

impl Resolver for Items {
    fn name(&self) -> &str {
        &self.resolver
    }

    fn resolve(&self, raw: &str) -> Binding {
        if let Some(why) = &self.unavailable {
            return Binding::Unresolved(why.clone());
        }
        let id = raw.trim();
        if id.is_empty() {
            return Binding::Unresolved("an empty anchor names nothing".to_string());
        }
        match self.items.iter().find(|(held, _)| held == id) {
            // The identity is the string the far end spells, so the normalized
            // form is that string and never a form this engine invented. The
            // revision beside it is what the snapshot says the item is at now,
            // and it is the only thing in the answer that can move while the
            // identity stands still.
            Some((_, revision)) => Binding::Resolved {
                normalized: id.to_string(),
                excluded_by: None,
                revision: Some(revision.clone()),
            },
            // A corpus exclusion is about a path in this repository, and an
            // external item is under no path here, so `excluded_by` above is
            // always empty and nothing is withheld: an export filter is
            // declared over this corpus and never over somebody else's.
            None => Binding::Unresolved(format!(
                "the snapshot of {} holds no item `{id}`",
                self.source
            )),
        }
    }
}

/// Open the snapshot one declaration names, and build its resolver.
///
/// Nothing where the declaration names no resolver. Such a declaration supplies
/// no anchor identity, so there is nothing to add, and an entry named for
/// nothing would sit in the set claiming an empty name. [`crate::plan`] refuses
/// that declaration, so no import writes an edge under it, and an edge already
/// in the corpus is reported as naming a resolver this run does not have, which
/// is then exactly what is true.
///
/// Every other failure is an [`Items::unavailable`] rather than an error,
/// because the caller is a graph build that has a corpus to report on either
/// way. A verb that refuses an unpinned snapshot is [`crate::plan`], and it
/// refuses before it reads a byte.
pub fn open(root: &Path, declaration: &Declaration) -> Option<Items> {
    let resolver = match declaration.resolver.as_deref().map(str::trim) {
        Some(resolver) if !resolver.is_empty() => resolver,
        _ => return None,
    };
    let Some(pinned) = declaration.digest.as_deref() else {
        return Some(Items::unavailable(
            resolver,
            format!(
                "nothing pins the snapshot of `{}`, so no item in it is a pinned identity. Write \
                 the digest the publisher gave you as `imports.{}.digest` in \
                 `.headwater/taxonomy.yml`",
                declaration.name, declaration.name
            ),
        ));
    };
    let dir = root.join(&declaration.at);
    if let Err(error) = release::verify(&dir, pinned) {
        return Some(Items::unavailable(
            resolver,
            format!(
                "the snapshot of `{}` is not the pinned artifact: {error}",
                declaration.name
            ),
        ));
    }
    Some(match crate::snapshot::at(&dir) {
        Ok(snapshot) => Items::of(resolver, &snapshot),
        Err(error) => Items::unavailable(
            resolver,
            format!(
                "the snapshot of `{}` did not read: {error}",
                declaration.name
            ),
        ),
    })
}

/// Every resolver this repository's declared imports supply, in declaration
/// order.
///
/// One call, so that a caller adds inbound integration to a resolver set in one
/// line and cannot add half of it.
pub fn over(root: &Path, declarations: &[Declaration]) -> Vec<Items> {
    declarations
        .iter()
        .filter_map(|declaration| open(root, declaration))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAYLOAD: &str = "\
snapshot:
  format: 1
  source: acme/work-items
  fetched: 2026-08-14
  items:
    - id: \"12345\"
      revision: \"7\"
";

    fn snapshot() -> Snapshot {
        crate::snapshot::read(PAYLOAD).expect("the payload reads")
    }

    #[test]
    fn an_item_the_snapshot_pinned_binds_to_the_identity_the_far_end_spells() {
        let items = Items::of("ado-snapshot", &snapshot());
        assert_eq!(items.name(), "ado-snapshot");
        assert_eq!(
            items.resolve("12345"),
            Binding::Resolved {
                normalized: "12345".to_string(),
                excluded_by: None,
                revision: Some("7".to_string()),
            }
        );
    }

    /// The binding carries the revision the snapshot pinned, and that is the
    /// one thing about an item that moves while its identity stands still.
    ///
    /// Two components read it. `headwater_check::suspect` compares it against
    /// the `verified_revision` an import wrote onto the edge, and
    /// `headwater_graph::Target::resolution` renders it into the cache key, so
    /// an advance divides the key of every instance about that edge
    /// ([#160](https://github.com/headwater-ai/headwater/issues/160)).
    #[test]
    fn the_binding_carries_the_revision_the_snapshot_pinned() {
        let items = Items::of("ado-snapshot", &snapshot());
        let Binding::Resolved {
            normalized,
            revision,
            ..
        } = items.resolve("12345")
        else {
            panic!("the item the snapshot pins did not bind");
        };
        assert_eq!(normalized, "12345");
        assert_eq!(revision.as_deref(), Some("7"));

        // And the same snapshot at a later fetch answers with the later
        // revision under one identity, which is the whole of the drift.
        let advanced = crate::snapshot::read(&PAYLOAD.replace("revision: \"7\"", "revision: \"8\""))
            .expect("the payload reads");
        let Binding::Resolved { revision, .. } =
            Items::of("ado-snapshot", &advanced).resolve("12345")
        else {
            panic!("the item the snapshot pins did not bind");
        };
        assert_eq!(revision.as_deref(), Some("8"));
    }

    /// The case the whole module exists for. A resolver that answered anything
    /// but this would make a typo into a bound edge, and no check would report
    /// it, because the graph would agree with it.
    #[test]
    fn an_identity_the_snapshot_does_not_hold_is_reported_and_never_bound() {
        let items = Items::of("ado-snapshot", &snapshot());
        for typo in ["12346", "1234", "#12345", "12345x", "ado-12345", ""] {
            let Binding::Unresolved(why) = items.resolve(typo) else {
                panic!("`{typo}` bound to something, and the snapshot holds one item");
            };
            assert!(!why.is_empty(), "{typo}");
        }
    }

    /// The one thing that is trimmed, and the reason it is not an equality
    /// rule: a leading space belongs to the YAML scalar rather than to the
    /// upstream identity.
    #[test]
    fn surrounding_space_is_trimmed_and_nothing_else_is() {
        let items = Items::of("ado-snapshot", &snapshot());
        assert!(matches!(
            items.resolve("  12345  "),
            Binding::Resolved { .. }
        ));
        assert!(matches!(items.resolve("123 45"), Binding::Unresolved(_),));
    }

    /// A declaration that names no resolver supplies none, rather than an entry
    /// under an empty name. `plan` is what refuses the declaration itself.
    #[test]
    fn a_declaration_that_names_no_resolver_supplies_nothing() {
        let root = Path::new("/nowhere");
        let mut declaration = Declaration {
            name: "ado".to_string(),
            at: "imports/ado".to_string(),
            digest: Some("sha256:0".to_string()),
            channel: Some("a fetch job".to_string()),
            resolver: None,
        };
        assert_eq!(open(root, &declaration), None);
        declaration.resolver = Some("   ".to_string());
        assert_eq!(open(root, &declaration), None);
        assert!(over(root, std::slice::from_ref(&declaration)).is_empty());
    }

    /// An unpinned snapshot binds nothing, because a run binds against pinned
    /// bytes or against nothing. `plan` refuses one for the same reason.
    #[test]
    fn an_unpinned_snapshot_supplies_a_resolver_that_binds_nothing() {
        let items = open(
            Path::new("/nowhere"),
            &Declaration {
                name: "ado".to_string(),
                at: "imports/ado".to_string(),
                digest: None,
                channel: Some("a fetch job".to_string()),
                resolver: Some("ado-snapshot".to_string()),
            },
        )
        .expect("it names a resolver");
        let Binding::Unresolved(why) = items.resolve("12345") else {
            panic!("an unpinned snapshot bound something");
        };
        assert!(why.contains("nothing pins"), "{why}");
    }

    #[test]
    fn a_snapshot_that_did_not_open_refuses_every_string_with_the_reason() {
        let items = Items::unavailable("ado-snapshot", "the pin moved".to_string());
        assert_eq!(items.held(), 0);
        assert_eq!(
            items.resolve("12345"),
            Binding::Unresolved("the pin moved".to_string())
        );
    }
}
