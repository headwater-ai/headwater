// SPDX-License-Identifier: Apache-2.0
//! The pinned-export anchor resolver: a document identity that another
//! repository's committed export holds.
//!
//! [Spec 7](../../../../docs/spec/07-distribution-and-federation.md#the-tier-above-a-corpus-harvests-it)
//! says how a tier above a corpus reaches the corpora below it. It declares an
//! anchor kind, exactly one resolver owns that kind, and "that resolver reads
//! pinned corpus exports, in the way that the `code_path` resolver reads a
//! source tree". Spec 2 names three classes of resolver: the source tree, a
//! committed snapshot, and a pinned corpus export. This module is the third
//! ([#1233](https://github.com/headwater-ai/headwater/issues/1233)).
//!
//! # One pattern, the third instance
//!
//! Spec 7 also says that "a taxonomy pin, a requirements snapshot pin, and a
//! source-export pin all work the same way". So this module copies
//! [`crate::anchors`] and adds no mechanism of its own:
//!
//! - The pin is declared in `.headwater/taxonomy.yml`, under
//!   `harvests.<name>`, with the same fields as `imports.<name>`: `at`,
//!   `digest`, `channel` and `resolver`. It is not in the lock. The lock is
//!   what `headwater taxonomy resolve` writes, and a pin is what a person
//!   writes down.
//! - The digest is checked before any document is read, and a file whose bytes
//!   are not the pinned bytes binds nothing.
//! - A pin that did not open still supplies its resolver, and that resolver
//!   refuses every string with a reason that names the pin. An absent resolver
//!   would make the finding say that the run does not have a resolver it was
//!   declared to have, and send the reader to look for a missing feature
//!   instead of a missing file.
//! - An identity is looked up and never normalized. The whole normalization is
//!   [`str::trim`], because the publishing corpus owns the identity of its
//!   documents.
//!
//! # A missing export is never a narrower answer
//!
//! Spec 7: "a pinned export that the tier cannot read is a **finding that
//! names the pin**. It is never a narrower answer, delivered quietly." Each pin
//! supplies its own resolver, named in its own declaration, and each anchor
//! kind names one resolver. So an anchor into one repository is looked up in
//! that repository's export and nowhere else, and a second export that happens
//! to hold the same identifier cannot answer for the first. The fixture in
//! `engine/crates/cli/tests/harvest.rs` holds that.
//!
//! # The digest is over the one file
//!
//! An export is one file that `headwater export` writes, and it carries no
//! release record. So the pinned digest is [`headwater_hash::digest`] of the
//! bytes of that file, in the `sha256:<hex>` form every artifact of this engine
//! writes. The publisher can state it, and the tier can compute it with any
//! SHA-256 tool.
//!
//! # A withheld document, and a typo
//!
//! A filtered export can withhold a document, and spec 7 says that an anchor
//! into a withheld document resolves to `withheld` rather than to a dangling
//! reference. [HW-DR-0100](../../../../docs/decisions/0100-a-counted-tombstone-lists-a-digest-of-each-withheld-identifier-and-a-sealed-one-lists-nothing.md)
//! states what the export says for this. Under the `counted` grain each
//! tombstone lists the digest of each identifier it withheld, so a string that
//! the plain list does not hold is digested and tested against those lists. A
//! hit is [`headwater_graph::anchors::Binding::Withheld`], under the export's
//! profile, and this resolver is its one producer. A miss stays unresolved, and
//! the reason names the case: the export is not filtered, the string is not a
//! withheld identifier, the grain is `sealed` and the export lists nothing, or
//! the export predates the list.
//!
//! # No socket
//!
//! [`open`] takes a path under the repository root. Nothing here fetches, and a
//! scheduled job outside this engine is what commits a new copy.

use headwater_graph::anchors::{Binding, Resolver};
use std::path::Path;

/// One pinned export, as `.headwater/taxonomy.yml` declares it under
/// `harvests.<name>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    /// The name the declaration is keyed by: which source corpus this is.
    pub name: String,
    /// Where the committed export sits, relative to the repository root.
    pub at: String,
    /// The digest of the export file, as the publisher stated it.
    pub digest: Option<String>,
    /// How the digest reached this repository, in the words of the person who
    /// wrote it down.
    pub channel: Option<String>,
    /// The anchor resolver this export supplies, by the name the taxonomy's
    /// `anchors` block gives it.
    pub resolver: String,
}

impl Pin {
    /// The pin as a finding names it: its name, its path and its digest.
    fn named(&self) -> String {
        format!(
            "the pinned export `{}` at `{}` ({})",
            self.name,
            self.at,
            self.digest.as_deref().unwrap_or("no digest")
        )
    }
}

/// A resolver over the documents of one committed export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    /// The name an anchor kind names it by.
    resolver: String,
    /// The pin, as a message names it.
    named: String,
    /// The digest of the bytes read. It is the revision of every binding, so a
    /// cached verdict about an edge moves when the harvest moves.
    digest: String,
    /// Every document identity the export holds, verbatim.
    ids: Vec<String>,
    /// Whether the export declares that a filter acted on it.
    filtered: bool,
    /// What the export says about what its filter withheld.
    withheld: Withheld,
    /// Why this resolver holds no documents. Every string is refused with it.
    unavailable: Option<String>,
}

impl Export {
    /// A resolver that binds nothing and says why.
    pub fn unavailable(pin: &Pin, why: String) -> Self {
        Export {
            resolver: pin.resolver.clone(),
            named: pin.named(),
            digest: String::new(),
            ids: Vec::new(),
            filtered: false,
            withheld: Withheld::default(),
            unavailable: Some(why),
        }
    }

    /// How many document identities this resolver can bind.
    pub fn held(&self) -> usize {
        self.ids.len()
    }

    /// Why this resolver binds nothing, and `None` where the export read.
    pub fn unread(&self) -> Option<&str> {
        self.unavailable.as_deref()
    }
}

impl Resolver for Export {
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
        if self.ids.iter().any(|held| held == id) {
            return Binding::Resolved {
                matched: vec![id.to_string()],
                normalized: id.to_string(),
                excluded_by: None,
                revision: Some(self.digest.clone()).into(),
            };
        }
        if !self.filtered {
            return Binding::Unresolved(format!("{} holds no document `{id}`", self.named));
        }
        let named = &self.named;
        let profile = &self.withheld.profile;
        match &self.withheld.digests {
            Some(digests) if digests.contains(&headwater_hash::digest(id.as_bytes())) => {
                Binding::Withheld {
                    profile: profile.clone(),
                }
            }
            Some(_) => Binding::Unresolved(format!(
                "{named} holds no document `{id}`, and `{id}` is not one of the identifiers it \
                 withheld under the profile `{profile}`"
            )),
            None if self.withheld.grain.as_deref() == Some("sealed") => {
                Binding::Unresolved(format!(
                    "{named} holds no document `{id}`. The export is filtered under the `sealed` \
                     grain, where the existence of a withheld document is the secret, so it lists \
                     nothing and this run cannot tell a withheld document from a typo"
                ))
            }
            None => Binding::Unresolved(format!(
                "{named} holds no document `{id}`. The export is filtered, and it predates the \
                 digest of each withheld identifier in its tombstones (export version 1.3), so \
                 this run cannot tell a withheld document from a typo"
            )),
        }
    }
}

/// What a filtered export says about the documents its filter withheld.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Withheld {
    /// The name of the profile, `profile.name`. A withheld binding names it.
    profile: String,
    /// The grain, `profile.tombstone`, where the export states one.
    grain: Option<String>,
    /// The digest of every withheld identifier, over every tombstone. `None`
    /// where no tombstone carries an `identifiers` list: a `sealed` export, and
    /// one older than export version 1.3.
    digests: Option<Vec<String>>,
}

/// Read the committed export one pin names, and build its resolver.
///
/// Every failure is an [`Export::unavailable`] rather than an error, because
/// the caller is a graph build that has a corpus to report on either way.
pub fn open(root: &Path, pin: &Pin) -> Export {
    let Some(pinned) = pin.digest.as_deref() else {
        return Export::unavailable(
            pin,
            format!(
                "nothing pins {}, so no document in it is a pinned identity. Write the digest \
                 the publisher gave you as `harvests.{}.digest` in `.headwater/taxonomy.yml`",
                pin.named(),
                pin.name
            ),
        );
    };
    let bytes = match std::fs::read(root.join(&pin.at)) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Export::unavailable(pin, format!("{} did not read: {error}", pin.named()));
        }
    };
    let actual = headwater_hash::digest(&bytes);
    if actual != pinned {
        return Export::unavailable(
            pin,
            format!(
                "{} is not the pinned artifact: its bytes digest to {actual}",
                pin.named()
            ),
        );
    }
    match documents(&bytes) {
        Ok((ids, filtered, withheld)) => Export {
            resolver: pin.resolver.clone(),
            named: pin.named(),
            digest: actual,
            ids,
            filtered,
            withheld,
            unavailable: None,
        },
        Err(why) => Export::unavailable(pin, format!("{} did not read: {why}", pin.named())),
    }
}

/// Every document identity a native export holds, whether it declares a
/// filter, and what it says about what the filter withheld.
fn documents(bytes: &[u8]) -> Result<(Vec<String>, bool, Withheld), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "it is not UTF-8".to_string())?;
    let root =
        headwater_yaml::load(text).map_err(|errors| headwater_yaml::error::render(&errors))?;
    let map = root
        .value
        .as_map()
        .ok_or_else(|| "it is not a native export: the top level is not an object".to_string())?;
    let declaration = map
        .get("profile")
        .and_then(|profile| profile.value.as_map());
    let member = |key: &str| {
        declaration
            .and_then(|profile| profile.get(key))
            .and_then(|node| node.value.as_scalar())
    };
    let filtered = member("filtered")
        .and_then(headwater_yaml::core_schema::as_bool)
        .unwrap_or(false);
    let mut digests: Option<Vec<String>> = None;
    for stone in map
        .get("tombstones")
        .and_then(|stones| stones.value.as_seq())
        .into_iter()
        .flatten()
        .filter_map(|stone| stone.value.as_map())
    {
        if let Some(listed) = stone
            .get("identifiers")
            .and_then(|identifiers| identifiers.value.as_seq())
        {
            digests.get_or_insert_with(Vec::new).extend(
                listed
                    .iter()
                    .filter_map(|digest| digest.value.as_scalar())
                    .map(|digest| digest.text.clone()),
            );
        }
    }
    let withheld = Withheld {
        profile: member("name")
            .map(|name| name.text.clone())
            .unwrap_or_default(),
        grain: member("tombstone").map(|grain| grain.text.clone()),
        digests,
    };
    let documents = map
        .get("graph")
        .and_then(|graph| graph.value.as_map())
        .and_then(|graph| graph.get("documents"))
        .and_then(|documents| documents.value.as_seq())
        .ok_or_else(|| {
            "it is not a native export: it holds no `graph.documents` list".to_string()
        })?;
    let ids = documents
        .iter()
        .filter_map(|document| document.value.as_map())
        .filter_map(|document| document.get("id"))
        .filter_map(|id| id.value.as_scalar())
        .map(|id| id.text.clone())
        .collect();
    Ok((ids, filtered, withheld))
}

/// Every resolver this repository's pinned exports supply, in declaration
/// order.
pub fn over(root: &Path, pins: &[Pin]) -> Vec<Export> {
    pins.iter().map(|pin| open(root, pin)).collect()
}

/// What a run of the checks is told about each pin, beside the resolver
/// [`over`] built for it: its name, its path, the digest of the bytes there,
/// and why it binds nothing where it does not. `exports` is what [`over`]
/// returned for `pins`, in the same order. See [`headwater_check::harvest`].
pub fn readings(
    root: &Path,
    pins: &[Pin],
    exports: &[Export],
) -> Vec<headwater_check::harvest::Harvest> {
    pins.iter()
        .zip(exports)
        .map(|(pin, export)| headwater_check::harvest::Harvest {
            name: pin.name.clone(),
            at: pin.at.clone(),
            digest: std::fs::read(root.join(&pin.at))
                .ok()
                .map(|bytes| headwater_hash::digest(&bytes)),
            unread: export.unread().map(str::to_string),
        })
        .collect()
}

/// Every pinned export this repository declares, in declaration order.
///
/// Read out of `harvests` in `.headwater/taxonomy.yml`, beside `imports`. A
/// pin with no `at` or no `resolver` is refused by name. An import with no
/// resolver can still write edges that a later declaration binds, but a pin
/// exists only to supply a resolver, so a pin without one is a mistake and not
/// a stage. A pin with no `digest` is read, and its resolver refuses every
/// string with a reason that says what to write.
pub fn declared(root: &Path) -> Result<Vec<Pin>, String> {
    let path = root.join(headwater_resolve::package::CONSUMER);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("{}: {error}", path.display())),
    };
    let loaded = headwater_yaml::load(&text)
        .map_err(|errors| headwater_yaml::error::render(&errors))
        .map_err(|why| format!("{}: {why}", path.display()))?;
    let Some(map) = loaded.value.as_map() else {
        return Err(format!("{} is not a mapping", path.display()));
    };
    let Some(harvests) = crate::block(map, "harvests", "a mapping of pinned exports by name")?
    else {
        return Ok(Vec::new());
    };

    let mut out = Vec::new();
    for entry in harvests {
        let name = entry.key.value.clone();
        let Some(block) = entry.value.value.as_map() else {
            return Err(format!(
                "`harvests.{name}` is {}, and a pinned export is a mapping",
                entry.value.value.kind_name()
            ));
        };
        let text_of = |key: &str| {
            block
                .get(key)
                .and_then(|node| node.value.as_scalar())
                .map(|scalar| scalar.text.trim().to_string())
                .filter(|text| !text.is_empty())
        };
        let at = text_of("at").ok_or_else(|| {
            format!(
                "`harvests.{name}` names no `at`, so nothing says where the committed export is"
            )
        })?;
        // A resolver reads repository content and nothing else (spec 2), so a
        // path that leaves the repository root is refused rather than read.
        if !crate::contained(root, &at) {
            return Err(format!(
                "`harvests.{name}.at` is `{at}`, and a pinned export is read from a path under \
                 the repository root: write it relative, with no `..` segment and no symlink \
                 that leads out of the root"
            ));
        }
        let resolver = text_of("resolver").ok_or_else(|| {
            format!(
                "`harvests.{name}` names no `resolver`, so no anchor kind can reach the export it \
                 pins"
            )
        })?;
        out.push(Pin {
            name,
            at,
            digest: text_of("digest"),
            channel: text_of("channel"),
            resolver,
        });
    }
    Ok(out)
}
