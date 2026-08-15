// SPDX-License-Identifier: Apache-2.0
//! The other half of a migration that writes: one overlay address, re-addressed
//! in place, in the file this repository owns.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#versioning-by-measured-compatibility):
//! "The payload migrates overlays, not only documents… `headwater migrate
//! --apply` rewrites overlay addresses from that map." An address is the key of
//! an operation, so the rewrite is a splice at one key and nothing else. The
//! value under the key is what the adopter wrote, and no step of a payload has
//! anything to say about it.
//!
//! # Why this is a second writer beside [`crate::migrate`]
//!
//! That one rewrites a front-matter value in a document a person authored, and
//! it refuses any splice outside the front-matter block. This one rewrites a
//! mapping key in a YAML file that is not a document at all: it has no front
//! matter, no body and no kind, and no census row covers it. The two guard
//! different files and neither reaches the other's.
//!
//! # What the read back compares
//!
//! The patched text is read again by the resolver's own operation reader
//! ([`headwater_resolve::operation::read`]), and the result is compared with the
//! reading of the source. Every operation must keep its position, its kind and
//! its value, and its address must be the address it had or the address this
//! run meant to move it to.
//!
//! That comparison is stronger than "the file still parses", and it is stronger
//! than a text diff, because it is the reading the resolver will take. A splice
//! that merged two keys, dropped a block or turned an address into something
//! that does not parse changes it, and the run refuses with nothing written.
//!
//! # A quoted address keeps its quotes
//!
//! The span of a key covers the quotes that wrote it, so the bytes at a site
//! are `kinds.decision.facets`, `"kinds.decision.facets"` or the single-quoted
//! form. This module reads which of the three it found and writes the same
//! shape back, and refuses anything else rather than guessing.

use crate::tree::Composed;
use headwater_resolve::{OpKind, Operation};
use headwater_yaml::Span;
use std::path::Path;

/// One address a migration step moves, in one overlay.
///
/// It carries no step. The caller has already decided that the step is
/// mechanical and that this operation lies under the address it names, and a
/// writer that re-decided either would be a second reading of a payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    /// The operation as the overlay writes it, for a message.
    pub at: String,
    /// The address as it stands.
    pub address: String,
    /// The address it becomes.
    pub to: String,
    /// Where the address key sits in the overlay file.
    pub span: Span,
}

/// An overlay this run will not write, and why.
///
/// Closed, on the terms [`crate::migrate::Refused`] is closed: a test is
/// written against a name, so an invariant in an anonymous arm is invisible to
/// the suite.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The file did not read off the tree.
    Unreadable { path: String, why: String },
    /// The file did not load as YAML, or the resolver refused an address in it.
    Unparsed { path: String, why: String },
    /// The bytes at the key's own span are not the address, in any of the
    /// three shapes this module writes.
    Unquotable {
        path: String,
        address: String,
        found: String,
    },
    /// The patched text read back, and it is not the overlay it was.
    Unrecognizable { path: String, why: String },
}

impl Refused {
    pub fn path(&self) -> &str {
        match self {
            Refused::Unreadable { path, .. }
            | Refused::Unparsed { path, .. }
            | Refused::Unquotable { path, .. }
            | Refused::Unrecognizable { path, .. } => path,
        }
    }
}

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Refused::Unreadable { path, why } => write!(f, "{path} did not read: {why}"),
            Refused::Unparsed { path, why } => write!(f, "{path} did not read as an overlay: {why}"),
            Refused::Unquotable {
                path,
                address,
                found,
            } => write!(
                f,
                "the key of `{address}` in {path} is written as `{found}`, which is not a plain \
                 or quoted address this engine rewrites"
            ),
            Refused::Unrecognizable { path, why } => write!(
                f,
                "the rewrite of {path} does not read back as the overlay it patched: {why}"
            ),
        }
    }
}

/// Compose the rewrite of one overlay, writing nothing.
///
/// `Ok(None)` where the move list is empty, which is a payload with no step
/// over an address of this repository.
pub fn compose(
    root: &Path,
    path: &str,
    moves: &[Move],
) -> Result<Option<(Composed, usize)>, Refused> {
    if moves.is_empty() {
        return Ok(None);
    }
    let source = std::fs::read_to_string(root.join(path)).map_err(|error| Refused::Unreadable {
        path: path.to_string(),
        why: error.to_string(),
    })?;
    let before = read(path, &source)?;

    // Back to front, so that an offset ahead of an applied splice is still the
    // offset the loader reported.
    let mut ordered: Vec<&Move> = moves.iter().collect();
    ordered.sort_by_key(|moving| std::cmp::Reverse(moving.span.start.offset));
    let mut patched = source.clone();
    for moving in &ordered {
        let found = moving
            .span
            .slice(&patched)
            .ok_or_else(|| Refused::Unrecognizable {
                path: path.to_string(),
                why: "a key's span does not lie inside the file".to_string(),
            })?;
        let replacement =
            shaped(found, &moving.address, &moving.to).ok_or_else(|| Refused::Unquotable {
                path: path.to_string(),
                address: moving.address.clone(),
                found: found.to_string(),
            })?;
        patched.replace_range(moving.span.start.offset..moving.span.end.offset, &replacement);
    }

    recognizable(path, &before, &patched, moves)?;
    Ok(Some((
        Composed {
            path: path.to_string(),
            text: patched,
        },
        moves.len(),
    )))
}

/// Every operation of one overlay text, as the resolver reads them.
fn read(path: &str, text: &str) -> Result<Vec<Operation>, Refused> {
    let loaded = headwater_yaml::load(text).map_err(|errors| Refused::Unparsed {
        path: path.to_string(),
        why: headwater_yaml::error::render(&errors),
    })?;
    headwater_resolve::operation::read(0, path, &loaded).map_err(|errors| Refused::Unparsed {
        path: path.to_string(),
        why: headwater_resolve::render_errors(&errors).trim().to_string(),
    })
}

/// The replacement bytes, in the shape the source wrote the old address in.
fn shaped(found: &str, from: &str, to: &str) -> Option<String> {
    match found {
        _ if found == from => Some(to.to_string()),
        _ if found == format!("\"{from}\"") => Some(format!("\"{to}\"")),
        _ if found == format!("'{from}'") => Some(format!("'{to}'")),
        _ => None,
    }
}

/// The read back: the patched text is the overlay it patched, re-addressed.
///
/// Every operation keeps its position, its kind and its value. Its address is
/// the one it had, or the one a move of this run named for it.
fn recognizable(
    path: &str,
    before: &[Operation],
    patched: &str,
    moves: &[Move],
) -> Result<(), Refused> {
    let after = read(path, patched)?;
    let refuse = |why: String| Refused::Unrecognizable {
        path: path.to_string(),
        why,
    };

    if before.len() != after.len() {
        return Err(refuse(format!(
            "the patched overlay declares {} operations and it declared {}",
            after.len(),
            before.len()
        )));
    }
    for (was, now) in before.iter().zip(after.iter()) {
        let expected = moves
            .iter()
            .find(|moving| moving.at == was.at())
            .map(|moving| moving.to.clone())
            .unwrap_or_else(|| was.address.to_string());
        if now.address.to_string() != expected {
            return Err(refuse(format!(
                "`{}` reads `{}` and `{expected}` was expected",
                was.at(),
                now.address
            )));
        }
        if now.kind != was.kind {
            return Err(refuse(format!(
                "`{}` is now `{}` and it was `{}`",
                was.at(),
                now.kind.word(),
                was.kind.word()
            )));
        }
        let same = match (&was.value, &now.value) {
            (None, None) => was.kind == OpKind::Remove,
            (Some(was), Some(now)) => headwater_resolve::merge::same(&was.value, &now.value),
            _ => false,
        };
        if !same {
            return Err(refuse(format!(
                "the value under `{}` is not the value it was",
                was.at()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A directory that cleans itself up.
    ///
    /// `label` names the case and not the target. Cargo runs the cases of one
    /// target as threads of one process, so a directory keyed on the process
    /// identifier alone is one that a second case removes while the first is
    /// reading it.
    struct Dir(PathBuf);

    impl Dir {
        fn with(label: &str, text: &str) -> Dir {
            let root = std::env::temp_dir()
                .join(format!("headwater-overlay-{}-{label}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(&root).expect("a scratch directory");
            std::fs::write(root.join("overlay.yml"), text).expect("a fixture file");
            Dir(root)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const OVERLAY: &str = "add:\n  kinds.decision.facets:\n    require: [title]\n  kinds.decision.identifier: {scheme: decision_id}\n  facets.title:\n    role: name\n    type: string\n\noverride:\n  kinds.decision.voice: declarative\n";

    fn at(text: &str, address: &str, block: &str) -> Span {
        let loaded = headwater_yaml::load(text).expect("the fixture loads");
        headwater_resolve::operation::read(0, "overlay.yml", &loaded)
            .expect("operations")
            .into_iter()
            .find(|operation| {
                operation.address.to_string() == address && operation.kind.word() == block
            })
            .expect("the operation is there")
            .span
    }

    fn moving(text: &str, block: &str, address: &str, to: &str) -> Move {
        Move {
            at: format!("{block}.{address}"),
            address: address.to_string(),
            to: to.to_string(),
            span: at(text, address, block),
        }
    }

    #[test]
    fn every_address_under_the_step_moves_and_no_value_does() {
        let dir = Dir::with("moves", OVERLAY);
        let moves = vec![
            moving(
                OVERLAY,
                "add",
                "kinds.decision.facets",
                "kinds.ruling.facets",
            ),
            moving(
                OVERLAY,
                "add",
                "kinds.decision.identifier",
                "kinds.ruling.identifier",
            ),
            moving(
                OVERLAY,
                "override",
                "kinds.decision.voice",
                "kinds.ruling.voice",
            ),
        ];
        let (composed, count) = compose(dir.path(), "overlay.yml", &moves)
            .expect("it composes")
            .expect("there is a file");
        assert_eq!(count, 3);
        assert!(composed.text.contains("kinds.ruling.facets:"), "{composed:?}");
        assert!(
            composed.text.contains("kinds.ruling.identifier: {scheme: decision_id}"),
            "the value beside a moved key is untouched: {composed:?}"
        );
        assert!(
            composed.text.contains("  facets.title:"),
            "an address the step does not reach stays: {composed:?}"
        );
        assert!(
            !composed.text.contains("kinds.decision"),
            "no old address survives: {composed:?}"
        );
    }

    #[test]
    fn an_empty_move_list_composes_no_file() {
        let dir = Dir::with("empty", OVERLAY);
        assert!(compose(dir.path(), "overlay.yml", &[])
            .expect("it composes")
            .is_none());
    }

    #[test]
    fn a_quoted_address_keeps_its_quotes() {
        let text = "add:\n  \"kinds.decision.facets\":\n    require: [title]\n";
        let dir = Dir::with("quoted", text);
        let moves = vec![moving(text, "add", "kinds.decision.facets", "kinds.ruling.facets")];
        let (composed, _) = compose(dir.path(), "overlay.yml", &moves)
            .expect("it composes")
            .expect("there is a file");
        assert!(
            composed.text.contains("\"kinds.ruling.facets\":"),
            "{composed:?}"
        );
    }

    /// A rewrite onto an address the overlay already declares.
    ///
    /// The splice succeeds. The loader refuses the result, because two keys of
    /// one mapping now read the same, and the refusal names both positions. So
    /// this case never reaches the comparison below it, and the assertion says
    /// which guard caught it rather than that something did.
    #[test]
    fn a_rewrite_that_collides_with_another_address_is_refused() {
        let dir = Dir::with("collides", OVERLAY);
        let moves = vec![moving(
            OVERLAY,
            "add",
            "kinds.decision.facets",
            "kinds.decision.identifier",
        )];
        let refused = compose(dir.path(), "overlay.yml", &moves).expect_err("it refuses");
        assert!(matches!(refused, Refused::Unparsed { .. }), "{refused}");
    }

    /// The read back, exercised by a move that does not describe the file.
    ///
    /// The span is the `override` key and the `at` beside it names the `add`
    /// block. The splice lands where the span says, so the file changes; the
    /// comparison expects the `override` operation to have kept its address,
    /// because no move of this run names it. Nothing is written.
    ///
    /// The case is a caller that built its move list against one reading of the
    /// overlay and handed it to a writer that took another. That is the state
    /// this comparison exists for, and no earlier guard sees it: the YAML
    /// parses, every address parses, and the operation count is unchanged.
    #[test]
    fn a_move_that_does_not_describe_the_operation_at_its_span_is_refused() {
        let dir = Dir::with("mismatched", OVERLAY);
        let mut moving = moving(
            OVERLAY,
            "override",
            "kinds.decision.voice",
            "kinds.ruling.voice",
        );
        moving.at = "add.kinds.decision.voice".to_string();
        let refused = compose(dir.path(), "overlay.yml", &[moving]).expect_err("it refuses");
        assert!(
            matches!(refused, Refused::Unrecognizable { .. }),
            "{refused}"
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("overlay.yml")).expect("it reads"),
            OVERLAY,
            "a refusal composes nothing, so the file on the tree is untouched"
        );
    }

    #[test]
    fn a_span_whose_bytes_are_not_the_address_is_refused() {
        let dir = Dir::with("unquotable", OVERLAY);
        let mut moving = moving(
            OVERLAY,
            "add",
            "kinds.decision.facets",
            "kinds.ruling.facets",
        );
        moving.address = "facets.title".to_string();
        let refused = compose(dir.path(), "overlay.yml", &[moving]).expect_err("it refuses");
        assert!(matches!(refused, Refused::Unquotable { .. }), "{refused}");
    }

    #[test]
    fn an_overlay_that_is_not_there_is_refused_by_name() {
        let dir = Dir::with("missing", OVERLAY);
        let moves = vec![moving(
            OVERLAY,
            "add",
            "kinds.decision.facets",
            "kinds.ruling.facets",
        )];
        let refused = compose(dir.path(), "nowhere.yml", &moves).expect_err("it refuses");
        assert!(matches!(refused, Refused::Unreadable { .. }), "{refused}");
    }
}
