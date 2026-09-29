// SPDX-License-Identifier: Apache-2.0
//! Spec 12's `Scope` block, held against [`headwater_check::scope::Scope`].
//!
//! # Why this file exists
//!
//! The inputs a check may declare are stated twice: once as the fields of
//! `Scope`, which is what the runner hands an instance and keys its cache on,
//! and once as the fenced block under *Scope — the declaration everything else
//! rests on* in `docs/spec/12-check-layer.md`, which is what a rule author reads
//! before choosing one. Nothing held the two together, and by
//! [#1407](https://github.com/headwater-ai/headwater/issues/1407) the block
//! named five of the seven: `needs_claims` and `needs_observations` had landed
//! in the engine with no line in the specification.
//!
//! # The model
//!
//! `engine/crates/resolve/tests/spec_seven_subjects.rs` and
//! `engine/crates/generate/tests/spec_six_projections.rs` hold a specification
//! block against an engine list the same way. The block carries engine
//! identifiers, so no prose-to-identifier mapping can drift, and the comparison
//! runs in both directions, so neither an emptied block nor an unnamed flag
//! passes.
//!
//! The engine's list is [`Scope::flags`], whose body destructures `Scope` with
//! no `..`. A new field does not compile until `flags` names it, and this file
//! then fails until spec 12 does.
//!
//! # What this does not hold
//!
//! The grain lines of the block and the `//` comment on each flag line are
//! prose. Only the flag names are bound here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The heading the section opens with.
const SECTION: &str = "## Scope — the declaration everything else rests on";

fn spec_twelve() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/spec/12-check-layer.md")
}

/// Every flag name on a `+ needs_…:` line of the first fenced block after
/// [`SECTION`], in the order a reader meets them.
fn spec_flags() -> Vec<String> {
    let path = spec_twelve();
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let (_, after) = text
        .split_once(SECTION)
        .unwrap_or_else(|| panic!("{}: no `{SECTION}` heading", path.display()));
    let (_, from_fence) = after
        .split_once("```")
        .unwrap_or_else(|| panic!("{}: `{SECTION}` holds no fenced block", path.display()));
    let block = from_fence
        .split_once("```")
        .map(|(block, _)| block)
        .unwrap_or_else(|| panic!("{}: the `Scope` block is never closed", path.display()));

    let mut flags = Vec::new();
    for line in block.lines() {
        let Some(rest) = line.trim_start().strip_prefix('+') else {
            continue;
        };
        let rest = rest.trim_start();
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_lowercase() || *c == '_')
            .collect();
        if name.starts_with("needs_") && rest[name.len()..].trim_start().starts_with(':') {
            flags.push(name);
        }
    }
    assert!(
        !flags.is_empty(),
        "{}: the `Scope` block under `{SECTION}` holds no `+ needs_…:` line, so this case would \
         compare nothing",
        path.display()
    );
    flags
}

/// The `Scope` block of spec 12 names every input flag `Scope` declares, and
/// no other.
///
/// The decisive case for #1407.
///
/// # Watched failing
///
/// On `main` at `16eecca0`, with `Scope::flags` in place and spec 12 unedited,
/// this reddened naming `needs_claims` and `needs_observations` as declared and
/// unstated. Deleting one flag line from the block reddens it naming that flag.
/// Adding `+ needs_bogus: bool` reddens it naming `needs_bogus` as extra.
#[test]
fn the_scope_block_of_spec_12_names_every_flag_scope_declares() {
    let rows = spec_flags();
    let named: BTreeSet<&str> = rows.iter().map(String::as_str).collect();
    let declared: BTreeSet<&str> = headwater_check::coverage::SCOPE
        .flags()
        .iter()
        .map(|(name, _)| *name)
        .collect();

    let extra: Vec<&&str> = named.difference(&declared).collect();
    let missing: Vec<&&str> = declared.difference(&named).collect();
    assert!(
        extra.is_empty() && missing.is_empty(),
        "docs/spec/12-check-layer.md's `Scope` block and `Scope` in \
         engine/crates/check/src/scope.rs disagree. `Scope` declares {missing:?} and the block \
         does not name them. The block names {extra:?} and `Scope` declares no such flag. The \
         block names {named:?}"
    );

    assert_eq!(
        rows.len(),
        named.len(),
        "docs/spec/12-check-layer.md's `Scope` block names a flag twice: {rows:?}"
    );
}
