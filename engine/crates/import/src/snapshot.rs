// SPDX-License-Identifier: Apache-2.0
//! The payload of a committed snapshot: the items an external system of record
//! holds, and the links it says reach them.
//!
//! The pin over the snapshot is not here. It is
//! [`headwater_resolve::release`], unchanged, because
//! [spec 7](../../../../docs/spec/07-distribution-and-federation.md#upstream-awareness)
//! rules that "a taxonomy pin, a requirements snapshot pin and a source-export
//! pin all work the same way", and a second implementation of one pattern is
//! where two readings of one artifact start to differ. So a snapshot directory
//! carries `release.yml` beside this file, the digest covers every byte of the
//! directory including this one, and every refusal that
//! `headwater taxonomy vendor` makes over a package is a refusal this crate
//! makes over a snapshot for free.
//!
//! # What this file is, and who writes it
//!
//! A requirements tool exports what it holds. Nothing about that export is
//! Headwater's, and [Q19](../../../../docs/decisions/0019-inbound-integration-an-external-system-of-record.md)
//! declines to privilege ReqIF or a vendor's API shape: "the shape of a snapshot
//! is a property of the resolver". This file is what a fetch script leaves
//! behind after it has read one, and the engine reads it rather than the export
//! it came from.
//!
//! Q19 states three properties, and the reader below is each one. The snapshot
//! is committed inside the governed repository, so `at` takes a path under the
//! root and opens no socket. It carries its fetch time and the upstream identity
//! and revision of every item, so `fetched`, `id` and `revision` are required
//! and a missing one is a refusal. And it is a pin, which is the paragraph
//! above.

use headwater_yaml::Mapping;
use std::path::Path;

/// The payload file, beside the pin record in the same directory.
pub const PAYLOAD: &str = "snapshot.yml";

/// The format of that file, on the terms the lock and the release record take.
pub const FORMAT: u32 = 1;

/// A committed snapshot's payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    /// The external system of record this came out of, as its operator names
    /// it. It is a label for a reader and the engine decides nothing from it.
    pub source: String,
    /// When the fetch happened, as the fetch reported it. Q19 requires it, and
    /// nothing here verifies it: a date inside an artifact is the artifact's
    /// claim about itself.
    pub fetched: String,
    pub items: Vec<Item>,
    pub links: Vec<Link>,
}

/// One thing the upstream system holds, with the revision it was at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The upstream identity, as the far end spells it. This is what an anchor
    /// resolver normalizes and what an edge's target names.
    pub id: String,
    /// The upstream revision at the moment of the fetch. It is what an imported
    /// edge records, so that a later fetch can say which edges a change reaches
    /// rather than only that the snapshot moved.
    pub revision: String,
    /// A human label, where the export carried one. Nothing reads it.
    pub title: Option<String>,
}

/// One edge the upstream system says exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// The identifier of a document in this corpus.
    pub from: String,
    /// The relation to declare it under, by the name the taxonomy of this
    /// repository gives it.
    pub relation: String,
    /// The identity of an item, which the snapshot must also name.
    pub to: String,
}

/// Why a payload did not read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PayloadError {
    Absent(String),
    Unreadable(String),
    Malformed(String),
    Format { found: String },
}

impl std::fmt::Display for PayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayloadError::Absent(dir) => write!(
                f,
                "{dir} carries no {PAYLOAD}, so it is a directory rather than a snapshot of \
                 anything"
            ),
            PayloadError::Unreadable(why) => {
                write!(f, "the snapshot payload cannot be read: {why}")
            }
            PayloadError::Malformed(what) => write!(f, "the snapshot payload is malformed: {what}"),
            PayloadError::Format { found } => write!(
                f,
                "the snapshot payload declares format `{found}` and this engine reads {FORMAT}"
            ),
        }
    }
}

/// Read the payload of a snapshot directory.
pub fn at(dir: &Path) -> Result<Snapshot, PayloadError> {
    let path = dir.join(PAYLOAD);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(PayloadError::Absent(dir.display().to_string()))
        }
        Err(error) => {
            return Err(PayloadError::Unreadable(format!(
                "{}: {error}",
                path.display()
            )))
        }
    };
    read(&text)
}

/// Read a snapshot payload from its text.
pub fn read(text: &str) -> Result<Snapshot, PayloadError> {
    let root = headwater_yaml::load(text)
        .map_err(|errors| PayloadError::Unreadable(headwater_yaml::error::render(&errors)))?;
    let map = root
        .value
        .as_map()
        .ok_or_else(|| PayloadError::Malformed("the root is not a mapping".to_string()))?;
    let header = map
        .get("snapshot")
        .and_then(|node| node.value.as_map())
        .ok_or_else(|| PayloadError::Malformed("no `snapshot` block".to_string()))?;

    let format = text_of(header, "format").unwrap_or_default();
    if format != FORMAT.to_string() {
        return Err(PayloadError::Format {
            found: format.to_string(),
        });
    }

    let required = |key: &str| {
        text_of(header, key).map(str::to_string).ok_or_else(|| {
            PayloadError::Malformed(format!("the `snapshot` block declares no {key}"))
        })
    };
    let source = required("source")?;
    let fetched = required("fetched")?;

    let mut items = Vec::new();
    for (position, entry) in sequence(header, "items").iter().enumerate() {
        let id = text_of(entry, "id").ok_or_else(|| {
            PayloadError::Malformed(format!("item {} names no `id`", position + 1))
        })?;
        // Q19 asks a snapshot to carry the revision of every item, because the
        // drift report is per edge rather than per snapshot. An item with no
        // revision would make every edge into it unreportable, so it is a
        // refusal here rather than an empty string that reads as a revision.
        let revision = text_of(entry, "revision").ok_or_else(|| {
            PayloadError::Malformed(format!(
                "item `{id}` names no `revision`, and an edge into it would then record nothing \
                 for a later fetch to compare against"
            ))
        })?;
        items.push(Item {
            id: id.to_string(),
            revision: revision.to_string(),
            title: text_of(entry, "title").map(str::to_string),
        });
    }

    let mut links = Vec::new();
    for (position, entry) in sequence(header, "links").iter().enumerate() {
        let at = |key: &str| {
            text_of(entry, key).map(str::to_string).ok_or_else(|| {
                PayloadError::Malformed(format!("link {} names no `{key}`", position + 1))
            })
        };
        links.push(Link {
            from: at("from")?,
            relation: at("relation")?,
            to: at("to")?,
        });
    }

    Ok(Snapshot {
        source,
        fetched,
        items,
        links,
    })
}

impl Snapshot {
    /// The item an identity names, and nothing where the snapshot holds none.
    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }
}

fn sequence<'a>(map: &'a Mapping, key: &str) -> Vec<&'a Mapping> {
    map.get(key)
        .and_then(|node| node.value.as_seq())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.value.as_map())
                .collect()
        })
        .unwrap_or_default()
}

fn text_of<'a>(map: &'a Mapping, key: &str) -> Option<&'a str> {
    map.get(key)
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "\
snapshot:
  format: 1
  source: acme/work-items
  fetched: 2026-08-14
  items:
    - id: \"12345\"
      revision: \"7\"
      title: The service refuses an unauthenticated request
    - id: \"12346\"
      revision: \"2\"
  links:
    - from: SPEC-FIX-one
      relation: audited_by
      to: \"12345\"
";

    #[test]
    fn a_payload_reads_its_items_and_its_links() {
        let snapshot = read(SOURCE).expect("the payload reads");
        assert_eq!(snapshot.source, "acme/work-items");
        assert_eq!(snapshot.fetched, "2026-08-14");
        assert_eq!(snapshot.items.len(), 2);
        assert_eq!(snapshot.item("12345").expect("named").revision, "7");
        assert_eq!(snapshot.item("12346").expect("named").title, None);
        assert!(snapshot.item("99999").is_none());
        assert_eq!(
            snapshot.links,
            vec![Link {
                from: "SPEC-FIX-one".to_string(),
                relation: "audited_by".to_string(),
                to: "12345".to_string(),
            }]
        );
    }

    /// An item with no revision is refused, because an edge into it would record
    /// nothing for a later fetch to compare against.
    #[test]
    fn an_item_with_no_revision_is_refused() {
        let source = SOURCE.replace("      revision: \"7\"\n", "");
        let error = read(&source).expect_err("a revision is required");
        assert!(error.to_string().contains("names no `revision`"), "{error}");
    }

    #[test]
    fn a_later_format_is_named_rather_than_guessed_at() {
        let source = SOURCE.replace("format: 1", "format: 2");
        assert_eq!(
            read(&source),
            Err(PayloadError::Format {
                found: "2".to_string()
            })
        );
    }

    #[test]
    fn a_payload_with_no_fetch_time_is_refused() {
        let source = SOURCE.replace("  fetched: 2026-08-14\n", "");
        let error = read(&source).expect_err("a fetch time is required");
        assert!(error.to_string().contains("declares no fetched"), "{error}");
    }
}
