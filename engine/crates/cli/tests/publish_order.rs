// SPDX-License-Identifier: Apache-2.0
//! The publish order in `.github/workflows/publish-crates.yml`, held against
//! the workspace it claims to enumerate.
//!
//! # What it costs to be wrong here, and where it shows
//!
//! `publish-crates.yml` walks a hand-written list of crate names and publishes
//! each one to crates.io in that order. The list was walked once, when
//! [#735](https://github.com/headwater-ai/headwater/issues/735) was filed, and
//! it is deliberately not recomputed on every run — a topological order that
//! moves underneath a release is worse than one a diff shows. What nothing did
//! was compare the list against `engine/Cargo.toml`'s `members`.
//!
//! A member missing from the list is green in every job this repository runs.
//! It fails at a release, on a tag, after the version bump has landed and the
//! tag is cut, at the point where the next crate in the order fails to resolve
//! a dependency that was never published. [#479](https://github.com/headwater-ai/headwater/issues/479)
//! added `headwater-paint` and had to remember this file by hand, which is the
//! shape [HW-OBL-0172](../../../../docs/obligations/0172-nine-hand-kept-constants-enumerate-an-enum-and-nothing-holds-one-against-the-variants.md)
//! and "enumerate from the declaration" both name.
//!
//! # Why a set and an ordering, rather than the whole order
//!
//! Two different claims live in that string. The first is which crates it
//! names, and the workspace decides that: a member is publishable unless its
//! own manifest says otherwise. The second is the order, and the dependency
//! graph decides that. This file holds the set exactly, and holds the order
//! only where a member's manifest declares a dependency on another member —
//! which is the part a wrong order actually breaks.

use std::path::{Path, PathBuf};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// Every crate name the workflow's `order=` string lists, in its own order.
///
/// The string is a shell assignment continued over several lines with a
/// trailing backslash, so the parse takes everything between the opening quote
/// and the closing one and then splits on whitespace.
fn published_order() -> Vec<String> {
    let text = std::fs::read_to_string(repository_root().join(".github/workflows/publish-crates.yml"))
        .expect("the publish workflow is on disk");
    let start = text
        .find("order=\"")
        .expect("publish-crates.yml assigns the publish order to `order`");
    let rest = &text[start + "order=\"".len()..];
    let end = rest
        .find('"')
        .expect("the `order` assignment closes its quote");
    rest[..end]
        .split_whitespace()
        .filter(|word| *word != "\\")
        .map(str::to_string)
        .collect()
}

/// Every member of the engine workspace, as `(crate name, manifest text)`.
///
/// The name comes from each member's own `[package] name`, because a directory
/// name and a crate name are two different things and only one of them is what
/// `cargo publish` takes.
fn workspace_members() -> Vec<(String, String)> {
    let root = repository_root().join("engine");
    let text = std::fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest");
    let start = text.find("members = [").expect("the workspace declares members");
    let rest = &text[start..];
    let end = rest.find(']').expect("the members list closes");
    rest[..end]
        .lines()
        .filter_map(|line| line.trim().strip_prefix('"'))
        .filter_map(|line| line.split('"').next())
        .map(|relative| {
            let manifest = root.join(relative).join("Cargo.toml");
            let member = std::fs::read_to_string(&manifest)
                .unwrap_or_else(|why| panic!("{}: {why}", manifest.display()));
            let name = member
                .lines()
                .find_map(|line| line.trim().strip_prefix("name = \""))
                .and_then(|rest| rest.split('"').next())
                .unwrap_or_else(|| panic!("{} names no package", manifest.display()))
                .to_string();
            (name, member)
        })
        .collect()
}

/// The workflow names every member of the workspace, and no name that is not
/// one.
///
/// Both directions, because they fail differently: a member the list omits
/// breaks the release at the first crate that depends on it, and a name the
/// workspace no longer has makes `cargo publish` refuse a package it cannot
/// find, which reads like a registry fault.
#[test]
fn the_publish_order_names_exactly_the_members_of_the_workspace() {
    let order = published_order();
    let members: Vec<String> = workspace_members()
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    let missing: Vec<&String> = members.iter().filter(|name| !order.contains(name)).collect();
    let extra: Vec<&String> = order.iter().filter(|name| !members.contains(name)).collect();

    assert!(
        missing.is_empty() && extra.is_empty(),
        "publish-crates.yml and engine/Cargo.toml disagree about the workspace.\n\
         members the publish order omits, which fail at a release and nowhere else: {missing:?}\n\
         names the publish order carries and the workspace does not: {extra:?}"
    );
}

/// No crate is published before a member it depends on.
///
/// This is the claim the order itself makes, held against each member's own
/// `[dependencies]` rather than against a second hand-written graph. A
/// dependency on a crate outside this workspace is not read: crates.io already
/// has it.
#[test]
fn no_crate_is_published_before_a_member_it_depends_on() {
    let order = published_order();
    let position = |name: &str| order.iter().position(|one| one == name);

    let mut wrong = Vec::new();
    for (name, manifest) in workspace_members() {
        let Some(mine) = position(&name) else {
            continue; // the case above is what reports this
        };
        for line in manifest.lines() {
            let line = line.trim();
            let Some(dependency) = line.strip_prefix("headwater-") else {
                continue;
            };
            let Some(dependency) = dependency.split([' ', '.', '=']).next() else {
                continue;
            };
            let dependency = format!("headwater-{dependency}");
            if dependency == name {
                continue;
            }
            if let Some(theirs) = position(&dependency) {
                if theirs > mine {
                    wrong.push(format!(
                        "{name} is published at {mine} and depends on {dependency} at {theirs}"
                    ));
                }
            }
        }
    }

    assert!(
        wrong.is_empty(),
        "the publish order puts a crate before something it depends on:\n{}",
        wrong.join("\n")
    );
}
