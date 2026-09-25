// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for #952: a `governs` edge goes suspect when the bytes
//! it reaches change.
//!
//! #946 fixed a hook, and six documents that govern that hook stayed wrong in
//! silence, because `relation.target.suspect` instantiated only over a
//! relation an importer writes and `source-tree` answered no revision. The
//! design is the governs evaluation's "Aging" section: an edge records a
//! digest of what it reached, and a later run compares the digest against the
//! bytes that are there now.
//!
//! The case table is `engine/crates/import/tests/drift.rs`'s, moved from a
//! snapshot's revision to a digest of working-tree bytes. Each case builds its
//! corpus in a scratch directory keyed on its own label and the pid, because
//! cargo runs the cases of one target as threads of one process.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Context, Date, Declared, Observations, Patch, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const RULE: &str = headwater_check::suspect::RULE;
const LOCK: &str = "sha256:governs-suspect-fixture";
const TODAY: &str = "2026-09-24";
const YESTERDAY: &str = "2026-09-23";
const DOCUMENT: &str = "governs-suspect/hooks.md";

/// The four hook files #946 touched, with the bytes each one starts at.
const HOOKS: [(&str, &str); 4] = [
    (
        ".githooks/pre-commit",
        "#!/bin/sh\nexec headwater check --strict\n",
    ),
    (
        ".githooks/merge-regenerate",
        "#!/bin/sh\nexec headwater generate\n",
    ),
    (
        ".claude/hooks/write.sh",
        "#!/bin/sh\n. .claude/hooks/lib.sh\n",
    ),
    (
        ".claude/hooks/lib.sh",
        "refuse() { printf '%s\\n' \"$1\"; }\n",
    ),
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn scratch(label: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("hw-governs-suspect-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (path, bytes) in HOOKS {
        write(&root, path, bytes);
    }
    root
}

fn write(root: &Path, path: &str, bytes: &str) {
    let at = root.join(path);
    std::fs::create_dir_all(at.parent().expect("a parent")).expect("the directory creates");
    std::fs::write(at, bytes).expect("the file writes");
}

/// The digest of what a set of tree entries holds, computed here from the
/// bytes and not through the resolver, so that the definition the resolver
/// implements is pinned by a second statement of it: each entry's relative
/// path and the digest of its bytes, sorted by path, one line each.
fn expected(root: &Path, paths: &[&str]) -> String {
    let mut sorted: Vec<&str> = paths.to_vec();
    sorted.sort_unstable();
    let mut manifest = String::new();
    for path in sorted {
        let bytes = std::fs::read(root.join(path)).expect("the governed file reads");
        manifest.push_str(path);
        manifest.push('\0');
        manifest.push_str(&headwater_hash::digest(&bytes));
        manifest.push('\n');
    }
    headwater_hash::digest(manifest.as_bytes())
}

/// A governing document, with `entries` written verbatim under `governs:`.
fn document(root: &Path, last_verified: &str, entries: &[String]) {
    let mut text = format!(
        "---\nid: GS-FIX-hooks\nstatus: current\nstatus_since: 2026-01-05\nlast_verified: \
         {last_verified}\nsummary: governs the hooks a commit runs\nrelations:\n  governs:\n"
    );
    for entry in entries {
        text.push_str(entry);
        text.push('\n');
    }
    text.push_str("---\n\n# Governs the hooks\n\nThe hooks a commit runs.\n");
    write(root, DOCUMENT, &text);
}

/// Every hook, each entry carrying the digest of its bytes as they are now.
fn recorded(root: &Path) -> Vec<String> {
    HOOKS
        .iter()
        .map(|(path, _)| {
            format!(
                "    - to: {path}\n      verified_revision: \"{}\"",
                expected(root, &[path])
            )
        })
        .collect()
}

fn taxonomy() -> String {
    std::fs::read_to_string(fixtures_dir().join("governs-suspect.taxonomy.yml"))
        .expect("the fixture taxonomy")
}

fn run(root: &Path, today: &str, cache: &mut Cache) -> Run {
    run_under(root, today, cache, &taxonomy())
}

fn run_under(root: &Path, today: &str, cache: &mut Cache, source: &str) -> Run {
    let corpus = Corpus::new(root, "governs-suspect");
    let value = headwater_yaml::load(source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&value).expect("the taxonomy reads");
    let declarations = Declarations::read(&value).expect("the declarations read");
    let register = Register::read(&value).expect("the register reads");
    let shape = Shape::read(&value).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &config,
    );
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: LOCK,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            observations: &Observations::empty(),
            source: "engine/crates/check/fixtures/governs-suspect.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(today).expect("the date parses")),
        cache,
    )
}

fn cold(root: &Path, today: &str) -> Run {
    run(root, today, &mut Cache::disabled())
}

fn suspect(run: &Run) -> Vec<&headwater_check::finding::Finding> {
    run.findings.iter().filter(|f| f.rule == RULE).collect()
}

/// The count of instances of this rule a run made, from its coverage.
fn instances(run: &Run) -> usize {
    run.instances
        .iter()
        .filter(|instance| instance.rule == RULE)
        .count()
}

/// Push a file's modification time forward without touching a byte of it.
fn touch(path: &Path) {
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(3_600);
    std::fs::File::options()
        .write(true)
        .open(path)
        .expect("the file opens")
        .set_modified(later)
        .expect("the modification time sets");
}

/// The decisive case, #946 replayed: four hooks, each recorded, then a touch,
/// then one byte.
#[test]
fn one_changed_byte_in_one_hook_is_one_suspect_entry_and_a_touch_is_none() {
    let root = scratch("decisive");
    document(&root, TODAY, &recorded(&root));

    // Run 1. Every entry matches the bytes it reached.
    let first = cold(&root, TODAY);
    assert!(suspect(&first).is_empty(), "{:?}", suspect(&first));
    assert_eq!(
        instances(&first),
        HOOKS.len(),
        "one instance per governs entry, suspect or not"
    );

    // Run 2. Every hook is touched and no byte moves.
    for (path, _) in HOOKS {
        touch(&root.join(path));
    }
    let second = cold(&root, TODAY);
    assert!(
        suspect(&second).is_empty(),
        "a modification time is not content: {:?}",
        suspect(&second)
    );

    // Run 3. One byte of one hook changes.
    write(
        &root,
        ".githooks/pre-commit",
        "#!/bin/sh\nexec headwater check --strict \n",
    );
    let third = cold(&root, TODAY);
    let reported = suspect(&third);
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert_eq!(
        reported[0].path, DOCUMENT,
        "reported at the declaring document"
    );
    assert!(
        reported[0].message.contains(".githooks/pre-commit"),
        "{}",
        reported[0].message
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// HW-OBL-0117's argument: the revision is part of the cache key, so a warm
/// cache cannot serve the verdict the changed byte falsified.
#[test]
fn a_warm_cache_reports_the_changed_byte_too() {
    let root = scratch("warm");
    document(&root, TODAY, &recorded(&root));

    let mut cache = Cache::at(&root, LOCK, &headwater_check::rules_digest());
    let first = run(&root, TODAY, &mut cache);
    assert!(suspect(&first).is_empty(), "{:?}", suspect(&first));
    cache.write(&root);

    write(
        &root,
        ".githooks/pre-commit",
        "#!/bin/sh\nexec headwater check --strict \n",
    );
    let mut cache = Cache::at(&root, LOCK, &headwater_check::rules_digest());
    let warm = run(&root, TODAY, &mut cache);
    assert!(cache.report().hits > 0, "the second run read the cache");
    let reported = suspect(&warm);
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert!(reported[0].message.contains(".githooks/pre-commit"));
    let _ = std::fs::remove_dir_all(&root);
}

/// Two trees that hold the same bytes at different modification times report
/// one digest, and one changed byte reports another.
#[test]
fn the_digest_is_a_function_of_bytes_and_not_of_modification_time() {
    let one = scratch("bytes-one");
    let two = scratch("bytes-two");
    touch(&two.join(".githooks/pre-commit"));
    let resolve = |root: &Path| {
        let corpus = Corpus::new(root, "governs-suspect");
        match Resolvers::over(&corpus)
            .get("source-tree")
            .expect("a source-tree resolver")
            .resolve(".githooks/pre-commit")
        {
            headwater_graph::anchors::Binding::Resolved { revision, .. } => revision,
            other => panic!("the hook resolves: {other:?}"),
        }
    };
    let first = resolve(&one);
    assert!(first.is_some(), "a bound file reports a revision");
    assert_eq!(first, resolve(&two));
    assert_eq!(
        first.as_deref(),
        Some(expected(&one, &[".githooks/pre-commit"]).as_str())
    );

    write(
        &two,
        ".githooks/pre-commit",
        "#!/bin/sh\nexec headwater check --strict \n",
    );
    assert_ne!(first, resolve(&two));
    let _ = std::fs::remove_dir_all(&one);
    let _ = std::fs::remove_dir_all(&two);
}

/// An entry with no recorded revision, on a document nobody verified today,
/// keeps the deliberate silence: no finding and no patch.
#[test]
fn an_unrecorded_entry_verified_before_today_is_silent() {
    let root = scratch("unrecorded-old");
    document(
        &root,
        YESTERDAY,
        &["    - .githooks/pre-commit".to_string()],
    );
    let ran = cold(&root, TODAY);
    assert!(suspect(&ran).is_empty(), "{:?}", suspect(&ran));
    assert!(ran
        .findings
        .iter()
        .all(|finding| finding.patch.is_none() || finding.rule != RULE));
    let _ = std::fs::remove_dir_all(&root);
}

/// An entry with no recorded revision, on a document verified today, is the
/// one advisory the rule raises over an unrecorded edge, and its patch records
/// the digest the edge reaches now.
#[test]
fn an_unrecorded_entry_verified_today_offers_the_digest() {
    let root = scratch("unrecorded-today");
    document(&root, TODAY, &["    - .githooks/pre-commit".to_string()]);
    let ran = cold(&root, TODAY);
    let reported = suspect(&ran);
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert_eq!(
        reported[0].severity,
        headwater_check::finding::Severity::Info
    );
    let digest = expected(&root, &[".githooks/pre-commit"]);
    match &reported[0].patch {
        Some(Patch::Half {
            path,
            relation,
            id,
            attributes,
        }) => {
            assert_eq!(path, DOCUMENT);
            assert_eq!(relation, "governs");
            assert_eq!(id, ".githooks/pre-commit");
            assert_eq!(attributes, &vec![("verified_revision".to_string(), digest)]);
        }
        other => panic!("a patch that records the digest: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// A moved digest on a document verified today carries the patch that
/// records the new one. On a document verified before today it carries none,
/// because the patch would record a verification that nobody performed.
#[test]
fn a_moved_digest_is_fixable_only_on_a_document_verified_today() {
    for (label, last_verified, fixable) in [
        ("moved-today", TODAY, true),
        ("moved-old", YESTERDAY, false),
    ] {
        let root = scratch(label);
        document(&root, last_verified, &recorded(&root));
        write(&root, ".claude/hooks/lib.sh", "refuse() { :; }\n");
        let ran = cold(&root, TODAY);
        let reported = suspect(&ran);
        assert_eq!(reported.len(), 1, "{label}: {reported:?}");
        assert_eq!(
            reported[0].severity,
            headwater_check::finding::Severity::Warn
        );
        assert_eq!(
            reported[0].patch.is_some(),
            fixable,
            "{label}: {:?}",
            reported[0].patch
        );
        if let Some(Patch::Half { attributes, .. }) = &reported[0].patch {
            assert_eq!(
                attributes,
                &vec![(
                    "verified_revision".to_string(),
                    expected(&root, &[".claude/hooks/lib.sh"])
                )]
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// A wildcard entry names the pattern and how many entries it matches now,
/// and says the per-entry count is not recorded, because one digest over a
/// set says that the set moved and not how many members did.
#[test]
fn a_moved_wildcard_names_its_pattern_and_its_match_count() {
    let root = scratch("wildcard");
    let digest = expected(&root, &[".claude/hooks/lib.sh", ".claude/hooks/write.sh"]);
    document(
        &root,
        YESTERDAY,
        &[format!(
            "    - to: .claude/hooks/*.sh\n      verified_revision: \"{digest}\""
        )],
    );
    assert!(suspect(&cold(&root, TODAY)).is_empty());

    write(&root, ".claude/hooks/write.sh", "#!/bin/sh\n");
    let ran = cold(&root, TODAY);
    let reported = suspect(&ran);
    assert_eq!(reported.len(), 1, "{reported:?}");
    let message = &reported[0].message;
    assert!(message.contains(".claude/hooks/*.sh"), "{message}");
    assert!(message.contains("2 entries"), "{message}");
    assert!(message.contains("not recorded"), "{message}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A list entry carries one digest over the union of what its members match,
/// so it goes suspect when any member's bytes move.
#[test]
fn a_list_entry_goes_suspect_when_one_member_moves() {
    let root = scratch("list");
    let digest = expected(
        &root,
        &[".githooks/pre-commit", ".githooks/merge-regenerate"],
    );
    document(
        &root,
        YESTERDAY,
        &[format!(
            "    - to: [.githooks/pre-commit, .githooks/merge-regenerate]\n      verified_revision: \"{digest}\""
        )],
    );
    assert!(suspect(&cold(&root, TODAY)).is_empty());

    write(&root, ".githooks/merge-regenerate", "#!/bin/sh\n");
    let reported = suspect(&cold(&root, TODAY)).len();
    assert_eq!(reported, 1);
    let _ = std::fs::remove_dir_all(&root);
}

/// A literal that names a directory gets no digest: reach under a directory
/// is the pattern language's to state, and a digest over one would be a walk
/// that the author did not write (HW-OBL-0104). So such an edge could never
/// go suspect, and the rule says so at `Info` and names `<literal>/**` as the
/// remedy, whether or not the entry records a revision and whenever its
/// document was verified. It offers no patch, because the remedy widens what
/// the edge reaches, and that is the author's decision.
#[test]
fn a_directory_literal_is_reported_with_the_wildcard_as_its_remedy() {
    for (label, last_verified, entry) in [
        (
            "directory-recorded",
            TODAY,
            "    - to: .githooks\n      verified_revision: \"sha256:0\"",
        ),
        ("directory-bare-old", YESTERDAY, "    - .githooks"),
    ] {
        let root = scratch(label);
        document(&root, last_verified, &[entry.to_string()]);
        let ran = cold(&root, TODAY);
        let reported = suspect(&ran);
        assert_eq!(reported.len(), 1, "{label}: {reported:?}");
        assert_eq!(
            reported[0].severity,
            headwater_check::finding::Severity::Info,
            "{label}"
        );
        assert_eq!(reported[0].path, DOCUMENT, "{label}");
        let message = &reported[0].message;
        assert!(message.contains("`.githooks`"), "{label}: {message}");
        assert!(message.contains("directory"), "{label}: {message}");
        assert!(
            reported[0].remediation.contains("`.githooks/**`"),
            "{label}: {}",
            reported[0].remediation
        );
        assert!(reported[0].patch.is_none(), "{label}");
        let _ = std::fs::remove_dir_all(&root);
    }
}

/// The wildcard the remedy names is one the rule reads: the same directory
/// written as `<literal>/**` carries a digest, and goes quiet once recorded.
#[test]
fn the_wildcard_remedy_carries_a_digest() {
    let root = scratch("directory-remedy");
    let hooks: Vec<&str> = HOOKS
        .iter()
        .map(|(path, _)| *path)
        .filter(|path| path.starts_with(".githooks/"))
        .collect();
    let digest = expected(&root, &hooks);
    document(
        &root,
        YESTERDAY,
        &[format!(
            "    - to: .githooks/**\n      verified_revision: \"{digest}\""
        )],
    );
    assert!(suspect(&cold(&root, TODAY)).is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

/// A relation that does not declare `verified_revision` is offered no fix and
/// no advisory on an unrecorded edge, because spec 2 makes an attribute the
/// relation does not declare a finding, and a fix must not write one.
#[test]
fn no_fix_is_offered_where_the_relation_does_not_declare_the_attribute() {
    let root = scratch("undeclared");
    document(&root, TODAY, &["    - .githooks/pre-commit".to_string()]);
    let undeclared = taxonomy().replace(
        "    attributes:\n      verified_revision: {type: string, owner: edge}\n",
        "",
    );
    assert_ne!(undeclared, taxonomy(), "the declaration was removed");
    let ran = run_under(&root, TODAY, &mut Cache::disabled(), &undeclared);
    assert!(suspect(&ran).is_empty(), "{:?}", suspect(&ran));
    let _ = std::fs::remove_dir_all(&root);
}
