// SPDX-License-Identifier: Apache-2.0
//! The pinned-export resolver, one case per refusal it owes.
//!
//! The end-to-end case, through the binary and a warm cache, is
//! `engine/crates/cli/tests/harvest.rs`. These cases hold the resolver and the
//! declaration reader on their own, so that a regression names the component.

use headwater_graph::anchors::{Binding, Resolver, Resolvers};
use headwater_import::harvest::{self, Pin};
use std::path::{Path, PathBuf};

/// A scratch repository root, removed when it is dropped.
struct Scratch {
    at: PathBuf,
}

impl Scratch {
    fn new(case: &str) -> Self {
        let at = std::env::temp_dir().join(format!(
            "headwater-import-harvest-{}-{case}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch root is made");
        Scratch { at }
    }

    fn path(&self) -> &Path {
        &self.at
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.at.join(relative);
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the directory is made");
        std::fs::write(path, text).expect("the file writes");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

fn export(ids: &[&str], filtered: bool) -> String {
    let documents: Vec<String> = ids
        .iter()
        .map(|id| format!("{{\"path\":\"docs/{id}.md\",\"kind\":\"service\",\"id\":\"{id}\"}}"))
        .collect();
    format!(
        "{{\"profile\":{{\"name\":\"full\",\"target\":\"native\",\"filtered\":{filtered}}},\
         \"graph\":{{\"documents\":[{}],\"anchors\":[],\"edges\":[]}}}}\n",
        documents.join(",")
    )
}

const AT: &str = "harvest/repo-b.json";

fn pin(digest: Option<String>) -> Pin {
    Pin {
        name: "repo-b".to_string(),
        at: AT.to_string(),
        digest,
        channel: Some("the nightly harvest job".to_string()),
        resolver: "export-repo-b".to_string(),
    }
}

/// A scratch holding one export, and the pin of its bytes.
fn pinned(case: &str, text: &str) -> (Scratch, Pin) {
    let scratch = Scratch::new(case);
    scratch.write(AT, text);
    let digest = headwater_hash::digest(text.as_bytes());
    (scratch, pin(Some(digest)))
}

fn reason(binding: Binding) -> String {
    match binding {
        Binding::Unresolved(why) => why,
        other => panic!("expected an unresolved binding, got {other:?}"),
    }
}

#[test]
fn a_held_identity_binds_to_itself_at_the_digest_of_the_export() {
    let text = export(&["SVC-1"], false);
    let (scratch, pin) = pinned("held", &text);
    let resolver = harvest::open(scratch.path(), &pin);
    assert_eq!(resolver.held(), 1);
    match resolver.resolve(" SVC-1 ") {
        Binding::Resolved {
            matched,
            normalized,
            revision,
            ..
        } => {
            assert_eq!(normalized, "SVC-1");
            assert_eq!(matched, vec!["SVC-1".to_string()]);
            assert_eq!(
                revision,
                Some(headwater_hash::digest(text.as_bytes())).into(),
                "the revision moves when the harvest moves"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_identity_the_export_does_not_hold_is_unresolved_and_names_the_pin() {
    let (scratch, pin) = pinned("unknown", &export(&["SVC-1"], false));
    let why = reason(harvest::open(scratch.path(), &pin).resolve("SVC-9"));
    assert!(why.contains("`repo-b`"), "{why}");
    assert!(why.contains(AT), "{why}");
    assert!(why.contains("SVC-9"), "{why}");
    assert!(!why.contains("filtered"), "{why}");
}

#[test]
fn a_spelling_the_far_end_did_not_write_is_not_normalized_onto_it() {
    let (scratch, pin) = pinned("spelling", &export(&["SVC-1"], false));
    let resolver = harvest::open(scratch.path(), &pin);
    for raw in ["svc-1", "#SVC-1", "SVC_1"] {
        reason(resolver.resolve(raw));
    }
}

#[test]
fn an_export_that_is_not_the_pinned_artifact_binds_nothing_and_names_the_pin() {
    let (scratch, pin) = pinned("moved", &export(&["SVC-1"], false));
    scratch.write(AT, &export(&["SVC-1", "SVC-2"], false));
    let resolver = harvest::open(scratch.path(), &pin);
    assert_eq!(resolver.held(), 0);
    let why = reason(resolver.resolve("SVC-1"));
    assert!(why.contains("`repo-b`"), "{why}");
    assert!(why.contains("not the pinned artifact"), "{why}");
    assert!(
        why.contains(pin.digest.as_deref().expect("it is pinned")),
        "the pinned digest is named: {why}"
    );
}

#[test]
fn a_missing_export_binds_nothing_and_names_the_pin() {
    let scratch = Scratch::new("missing");
    let resolver = harvest::open(scratch.path(), &pin(Some("sha256:00".to_string())));
    assert_eq!(
        resolver.name(),
        "export-repo-b",
        "the resolver is still there"
    );
    let why = reason(resolver.resolve("SVC-1"));
    assert!(why.contains("`repo-b`") && why.contains(AT), "{why}");
}

#[test]
fn a_pin_with_no_digest_binds_nothing_and_says_what_to_write() {
    let (scratch, _) = pinned("undigested", &export(&["SVC-1"], false));
    let why = reason(harvest::open(scratch.path(), &pin(None)).resolve("SVC-1"));
    assert!(why.contains("harvests.repo-b.digest"), "{why}");
}

#[test]
fn a_file_that_is_not_a_native_export_binds_nothing() {
    let (scratch, pin) = pinned("shape", "{\"profile\":{}}\n");
    let why = reason(harvest::open(scratch.path(), &pin).resolve("SVC-1"));
    assert!(why.contains("graph.documents"), "{why}");
}

#[test]
fn a_filtered_export_reports_an_absent_identity_as_unresolved_and_says_it_is_filtered() {
    let (scratch, pin) = pinned("filtered", &export(&["SVC-1"], true));
    let why = reason(harvest::open(scratch.path(), &pin).resolve("SVC-9"));
    assert!(why.contains("filtered"), "{why}");
}

#[test]
fn two_pins_that_name_one_resolver_are_refused() {
    let (scratch, first) = pinned("twice", &export(&["SVC-1"], false));
    let mut second = first.clone();
    second.name = "repo-c".to_string();
    let exports = harvest::over(scratch.path(), &[first, second]);
    let mut resolvers = Resolvers::new(Vec::new());
    let mut refused = 0;
    for export in exports {
        match resolvers.with(Box::new(export)) {
            Ok(next) => resolvers = next,
            Err(why) => {
                assert!(why.contains("export-repo-b"), "{why}");
                refused += 1;
                break;
            }
        }
    }
    assert_eq!(refused, 1);
}

const CONSUMER: &str = ".headwater/taxonomy.yml";

#[test]
fn the_declaration_reads_every_pin_in_order() {
    let scratch = Scratch::new("declared");
    scratch.write(
        CONSUMER,
        "harvests:\n  repo-a:\n    at: a.json\n    digest: sha256:aa\n    resolver: export-a\n  \
         repo-b:\n    at: b.json\n    resolver: export-b\n",
    );
    let pins = harvest::declared(scratch.path()).expect("it reads");
    assert_eq!(pins.len(), 2);
    assert_eq!(pins[0].name, "repo-a");
    assert_eq!(pins[0].digest.as_deref(), Some("sha256:aa"));
    assert_eq!(pins[1].resolver, "export-b");
    assert_eq!(pins[1].digest, None);
}

#[test]
fn a_pin_with_no_resolver_is_refused_by_name() {
    let scratch = Scratch::new("no-resolver");
    scratch.write(
        CONSUMER,
        "harvests:\n  repo-a:\n    at: a.json\n    digest: sha256:aa\n",
    );
    let why = harvest::declared(scratch.path()).expect_err("it is refused");
    assert!(
        why.contains("harvests.repo-a") && why.contains("resolver"),
        "{why}"
    );
}

#[test]
fn a_pin_with_no_path_is_refused_by_name() {
    let scratch = Scratch::new("no-at");
    scratch.write(
        CONSUMER,
        "harvests:\n  repo-a:\n    digest: sha256:aa\n    resolver: export-a\n",
    );
    let why = harvest::declared(scratch.path()).expect_err("it is refused");
    assert!(
        why.contains("harvests.repo-a") && why.contains("`at`"),
        "{why}"
    );
}

#[test]
fn a_repository_that_declares_no_pin_has_none() {
    let scratch = Scratch::new("none");
    assert_eq!(harvest::declared(scratch.path()), Ok(Vec::new()));
    scratch.write(CONSUMER, "imports: {}\n");
    assert_eq!(harvest::declared(scratch.path()), Ok(Vec::new()));
}
