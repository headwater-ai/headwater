// SPDX-License-Identifier: Apache-2.0
//! The network stays in one crate, and only this binary links it.
//!
//! [HW-DR-0075](../../../../docs/decisions/0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md)
//! lets `taxonomy vendor` take a location on one condition: the client that
//! fetches it lives in `headwater-fetch`, and no crate of the checking loop
//! reaches that crate or anything that opens a socket for it. That record names
//! the defect a reviewer looks for, which is a `headwater-fetch` line in the
//! manifest of any crate other than `headwater-cli`. This case is that review,
//! read off `engine/Cargo.lock`.
//!
//! The lock records an optional dependency whether or not its feature is on,
//! so the case holds for a build with the `fetch` feature and for one without.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[cfg(all(target_os = "linux", feature = "fetch"))]
mod relocations;

const FETCH: &str = "headwater-fetch";
const CLI: &str = "headwater-cli";

/// The packages that carry the client, the TLS stack, its roots and the
/// archive reader. Each must be reachable through `headwater-fetch`, and
/// through nothing else in the workspace but the binary above it.
const NETWORK: [&str; 5] = ["ureq", "rustls", "ring", "webpki-roots", "zip"];

struct Package {
    name: String,
    member: bool,
    dependencies: Vec<String>,
}

fn lock() -> Vec<Package> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} does not read: {error}", path.display()));
    let mut packages: Vec<Package> = Vec::new();
    let mut in_dependencies = false;
    for line in text.lines() {
        let line = line.trim();
        if line == "[[package]]" {
            packages.push(Package {
                name: String::new(),
                member: true,
                dependencies: Vec::new(),
            });
            in_dependencies = false;
            continue;
        }
        let Some(package) = packages.last_mut() else {
            continue;
        };
        if in_dependencies {
            if line == "]" {
                in_dependencies = false;
            } else {
                let entry = line.trim_end_matches(',').trim_matches('"');
                let name = entry.split_whitespace().next().unwrap_or(entry);
                package.dependencies.push(name.to_string());
            }
        } else if let Some(value) = line.strip_prefix("name = ") {
            package.name = value.trim_matches('"').to_string();
        } else if line.starts_with("source = ") {
            package.member = false;
        } else if line == "dependencies = [" {
            in_dependencies = true;
        }
    }
    packages
}

/// Every package name `start` reaches, `start` included. Two versions of one
/// name are one node here, which can only make a closure larger, so a case
/// that finds nothing in it has found nothing in the finer graph either.
fn closure(graph: &BTreeMap<String, BTreeSet<String>>, start: &str) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![start.to_string()];
    while let Some(name) = stack.pop() {
        if seen.insert(name.clone()) {
            if let Some(next) = graph.get(&name) {
                stack.extend(next.iter().cloned());
            }
        }
    }
    seen
}

#[test]
fn only_the_binary_reaches_the_crate_that_opens_a_socket() {
    let packages = lock();
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for package in &packages {
        graph
            .entry(package.name.clone())
            .or_default()
            .extend(package.dependencies.iter().cloned());
    }
    let members: BTreeSet<&str> = packages
        .iter()
        .filter(|package| package.member)
        .map(|package| package.name.as_str())
        .collect();

    // Not vacuous: the crate exists, the binary links it, and it carries the
    // network packages this case forbids everywhere else.
    assert!(
        members.contains(FETCH),
        "{FETCH} is not a workspace member in engine/Cargo.lock"
    );
    assert!(
        closure(&graph, CLI).contains(FETCH),
        "{CLI} does not reach {FETCH}, so `taxonomy vendor` cannot take a location"
    );
    let fetched = closure(&graph, FETCH);
    for name in NETWORK {
        assert!(
            fetched.contains(name),
            "{FETCH} does not reach `{name}`, so this case names a package it does not guard"
        );
    }

    let mut breaches: Vec<String> = Vec::new();
    for member in members.iter().filter(|m| **m != CLI && **m != FETCH) {
        let reached = closure(&graph, member);
        let found: Vec<&str> = std::iter::once(FETCH)
            .chain(NETWORK)
            .filter(|name| reached.contains(*name))
            .collect();
        if !found.is_empty() {
            breaches.push(format!("{member} reaches {}", found.join(", ")));
        }
    }
    assert!(
        breaches.is_empty(),
        "a crate other than {CLI} links the network (HW-DR-0075):\n  {}",
        breaches.join("\n  ")
    );
}

/// The control for `no_fetch.rs`, read off the binary rather than the lock.
///
/// The default binary carries the fetch, so the linker keeps a relocation
/// against each network call, and the reader must find all three.
/// `no_fetch.rs` asks the same reader of the binary built without the fetch
/// and wants none. Without this case, a reader that saw nothing would pass
/// that one on every binary (#1561).
#[cfg(all(target_os = "linux", feature = "fetch"))]
#[test]
fn the_binary_with_the_fetch_asks_the_linker_for_every_network_call() {
    let found = relocations::network_relocations(std::path::Path::new(env!(
        "CARGO_BIN_EXE_headwater"
    )));
    let wanted: BTreeSet<String> = relocations::NETWORK
        .iter()
        .map(|name| name.to_string())
        .collect();
    assert_eq!(
        found, wanted,
        "the default binary fetches, so a relocation names each network call"
    );
}
