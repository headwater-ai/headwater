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
use headwater_check::change::{Change, Unbound};
use headwater_check::{Cache, Context, Date, Declared, Observations, Patch, Register, Run, Shape};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

const RULE: &str = headwater_check::suspect::RULE;
const LOCK: &str = "sha256:governs-suspect-fixture";
const TODAY: &str = "2026-09-24";
const YESTERDAY: &str = "2026-09-23";
const TOMORROW: &str = "2026-09-25";
const DOCUMENT: &str = "governs-suspect/hooks.md";
/// A second governing document, which no change in this file re-verifies.
const OTHER: &str = "governs-suspect/other.md";

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

/// A directory under the temporary directory that is removed when this value
/// is dropped, so a case that fails an assertion leaves nothing behind (#1158).
struct Scratch(std::path::PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Scratch {
    type Target = std::path::Path;
    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

impl AsRef<std::path::Path> for Scratch {
    fn as_ref(&self) -> &std::path::Path {
        &self.0
    }
}

impl AsRef<std::ffi::OsStr> for Scratch {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.0.as_os_str()
    }
}

fn scratch(label: &str) -> Scratch {
    let root = Scratch(std::env::temp_dir().join(format!(
        "headwater-governs-suspect-{label}-{}",
        std::process::id()
    )));
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
    write(root, DOCUMENT, &text_of("hooks", last_verified, entries));
}

/// The bytes of a governing document with the slug `slug`.
fn text_of(slug: &str, last_verified: &str, entries: &[String]) -> String {
    let mut text = format!(
        "---\nid: GS-FIX-{slug}\nstatus: current\nstatus_since: 2026-01-05\nlast_verified: \
         {last_verified}\nsummary: governs the hooks a commit runs\nrelations:\n  governs:\n"
    );
    for entry in entries {
        text.push_str(entry);
        text.push('\n');
    }
    text.push_str("---\n\n# Governs the hooks\n\nThe hooks a commit runs.\n");
    text
}

/// A change that carries each document at `path`, with the version that stood
/// before it: the bytes given, or nothing where the change adds the document.
/// The prior bytes are handed to the reader directly, so no file stands for
/// them.
fn change(entries: &[(&str, Option<String>)]) -> Change {
    stated(entries, &[])
}

/// [`change`], and a `verified` line for each path in `verified`: the change
/// states that its author re-read that document, whether or not it carries it
/// (#1376).
fn stated(entries: &[(&str, Option<String>)], verified: &[&str]) -> Change {
    let mut manifest = String::from("headwater change 1\n");
    for (path, prior) in entries {
        match prior {
            Some(_) => manifest.push_str(&format!("prior\t{path}\tprior/{path}\n")),
            None => manifest.push_str(&format!("added\t{path}\n")),
        }
    }
    for path in verified {
        manifest.push_str(&format!("verified\t{path}\n"));
    }
    let priors: Vec<(String, String)> = entries
        .iter()
        .filter_map(|(path, prior)| prior.clone().map(|bytes| (format!("prior/{path}"), bytes)))
        .collect();
    Unbound::read(&manifest, |source: &Path| {
        priors
            .iter()
            .find(|(known, _)| Path::new(known) == source)
            .map(|(_, bytes)| bytes.clone().into_bytes())
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))
    })
    .expect("the manifest reads")
    .bind(|_| true)
}

/// A change that names the document at `path` with a prior version that does
/// not open. The change then states nothing about the document, so no stamp
/// may be offered: reading the failure as an added document would record a
/// verification nobody stated.
fn unreadable(path: &str) -> Change {
    Unbound::read(
        &format!("headwater change 1\nprior\t{path}\tprior/missing\n"),
        |_: &Path| Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
    )
    .expect("the manifest reads")
    .bind(|_| true)
}

/// The same document as it stood yesterday: every entry as it is now, and the
/// freshness facet one day earlier. A change that carries this as the prior
/// version states that its author re-read the document.
fn yesterday(slug: &str, entries: &[String]) -> String {
    text_of(slug, YESTERDAY, entries)
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
    run_in(root, &at(today), cache, source)
}

fn at(today: &str) -> Context {
    Context::at(Date::parse(today).expect("the date parses"))
}

/// A run under a context the case states, so that a change can be passed.
fn run_in(root: &Path, ctx: &Context, cache: &mut Cache, source: &str) -> Run {
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
            pin: None,
            harvests: &[],
            imports: &[],
            source: "engine/crates/check/fixtures/governs-suspect.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        ctx,
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
    cache.write(&root).expect("the cache writes");

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
    assert!(first.get().is_some(), "a bound file reports a revision");
    assert_eq!(first, resolve(&two));
    assert_eq!(
        first.get(),
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

/// An entry with no recorded revision keeps the deliberate silence in a run
/// that carries no change, on any date, and in a run whose change carries the
/// document with its freshness facet unmoved: no finding and no patch. The
/// date of `last_verified` alone decides nothing (#1259).
#[test]
fn an_unrecorded_entry_the_change_did_not_re_verify_is_silent() {
    let entries = ["    - .githooks/pre-commit".to_string()];
    let root = scratch("unrecorded-unstated");
    document(&root, TODAY, &entries);
    for (label, ctx) in [
        ("no change, same day", at(TODAY)),
        ("no change, a later day", at(TOMORROW)),
        (
            "a change that left the facet",
            at(TODAY).scoped_to(change(&[(DOCUMENT, Some(text_of("hooks", TODAY, &[])))])),
        ),
        (
            "a change that names another document",
            at(TODAY).scoped_to(change(&[(OTHER, None)])),
        ),
        (
            "a change whose prior version of the document does not open",
            at(TODAY).scoped_to(unreadable(DOCUMENT)),
        ),
    ] {
        let ran = run_in(&root, &ctx, &mut Cache::disabled(), &taxonomy());
        assert!(suspect(&ran).is_empty(), "{label}: {:?}", suspect(&ran));
    }
}

/// An entry with no recorded revision, on a document the change re-verified,
/// is the one advisory the rule raises over an unrecorded edge, and its patch
/// records the digest the edge reaches now. A change that adds the document
/// re-verified it too, because its author wrote every line of it.
#[test]
fn an_unrecorded_entry_the_change_re_verified_offers_the_digest() {
    let entries = ["    - .githooks/pre-commit".to_string()];
    let root = scratch("unrecorded-restated");
    document(&root, TODAY, &entries);
    for (label, ctx) in [
        (
            "a moved facet",
            at(TODAY).scoped_to(change(&[(DOCUMENT, Some(yesterday("hooks", &entries)))])),
        ),
        (
            "a moved facet, read a day later",
            at(TOMORROW).scoped_to(change(&[(DOCUMENT, Some(yesterday("hooks", &entries)))])),
        ),
        (
            "an added document",
            at(TODAY).scoped_to(change(&[(DOCUMENT, None)])),
        ),
    ] {
        let ran = run_in(&root, &ctx, &mut Cache::disabled(), &taxonomy());
        let reported = suspect(&ran);
        assert_eq!(reported.len(), 1, "{label}: {reported:?}");
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
            other => panic!("{label}: a patch that records the digest: {other:?}"),
        }
    }
}

/// A moved digest is a warning in every run. It carries the patch that
/// records the new digest only where the change re-verified the document that
/// declares the edge. Without a change, or where the change left the facet
/// where it stood, it carries none, because the patch would record a
/// verification that nobody stated.
#[test]
fn a_moved_digest_is_fixable_only_on_a_document_the_change_re_verified() {
    let root = scratch("moved");
    let entries = recorded(&root);
    document(&root, TODAY, &entries);
    write(&root, ".claude/hooks/lib.sh", "refuse() { :; }\n");
    for (label, ctx, fixable) in [
        (
            "re-verified",
            at(TODAY).scoped_to(change(&[(DOCUMENT, Some(yesterday("hooks", &entries)))])),
            true,
        ),
        ("no change", at(TODAY), false),
        (
            "facet unmoved",
            at(TODAY).scoped_to(change(&[(
                DOCUMENT,
                Some(text_of("hooks", TODAY, &entries)),
            )])),
            false,
        ),
        // A second change on the day the document was last stamped: its
        // freshness facet already reads today, so the author cannot move it,
        // and the change states the re-verification instead (#1376).
        (
            "same day, stated",
            at(TODAY).scoped_to(stated(
                &[(DOCUMENT, Some(text_of("hooks", TODAY, &entries)))],
                &[DOCUMENT],
            )),
            true,
        ),
        // An author who re-read the document and changed nothing in it.
        (
            "stated, not carried",
            at(TODAY).scoped_to(stated(&[], &[DOCUMENT])),
            true,
        ),
        // A statement about another document says nothing about this one.
        (
            "stated elsewhere",
            at(TODAY).scoped_to(stated(
                &[(DOCUMENT, Some(text_of("hooks", TODAY, &entries)))],
                &[OTHER],
            )),
            false,
        ),
    ] {
        let ran = run_in(&root, &ctx, &mut Cache::disabled(), &taxonomy());
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
    }
}

/// The decisive fixture for #1259. Two documents are both verified today and
/// both govern a file that moved. The change re-verified one of them. Only
/// that one is offered a stamp, on the day of the run and on the day after,
/// and a run with no change offers neither. Before the fix the clock decided,
/// so the document the change never touched was stamped too.
#[test]
fn a_stamp_is_offered_only_on_the_document_the_change_re_verified() {
    let root = scratch("two-verified-today");
    let entries = recorded(&root);
    document(&root, TODAY, &entries);
    write(&root, OTHER, &text_of("other", TODAY, &entries));
    write(&root, ".claude/hooks/lib.sh", "refuse() { :; }\n");
    let re_verified = || change(&[(DOCUMENT, Some(yesterday("hooks", &entries)))]);
    for (label, ctx, stamped) in [
        ("today", at(TODAY).scoped_to(re_verified()), vec![DOCUMENT]),
        (
            "tomorrow",
            at(TOMORROW).scoped_to(re_verified()),
            vec![DOCUMENT],
        ),
        ("no change", at(TODAY), vec![]),
    ] {
        let ran = run_in(&root, &ctx, &mut Cache::disabled(), &taxonomy());
        let reported = suspect(&ran);
        assert_eq!(reported.len(), 2, "{label}: {reported:?}");
        let patched: Vec<&str> = reported
            .iter()
            .filter(|finding| finding.patch.is_some())
            .map(|finding| finding.path.as_str())
            .collect();
        assert_eq!(patched, stamped, "{label}");
    }
}

/// The patch reads the change, so the change is in the cache key. A run whose
/// change re-verified the document is not served the patchless verdict a run
/// with no change stored, a second such run is served its own verdict from the
/// cache, and a run with no change after it is not served the patch.
#[test]
fn a_warm_cache_keys_the_patch_on_the_change() {
    let root = scratch("warm-change");
    let entries = recorded(&root);
    document(&root, TODAY, &entries);
    write(&root, ".claude/hooks/lib.sh", "refuse() { :; }\n");
    let re_verified =
        || at(TODAY).scoped_to(change(&[(DOCUMENT, Some(yesterday("hooks", &entries)))]));
    let unmoved = |verified: &[&str]| {
        at(TODAY).scoped_to(stated(
            &[(DOCUMENT, Some(text_of("hooks", TODAY, &entries)))],
            verified,
        ))
    };
    for (label, ctx, fixable, served) in [
        ("no change, cold", at(TODAY), false, false),
        (
            "re-verified, after a run with none",
            re_verified(),
            true,
            false,
        ),
        ("re-verified, warm", re_verified(), true, true),
        (
            "no change, after a re-verified run",
            at(TODAY),
            false,
            false,
        ),
        // One prior version with and without the statement: the key holds
        // the statement, so neither run is served the other's verdict, and
        // a repeat of the unstated run is served its own (#1376).
        ("facet unmoved, cold", unmoved(&[]), false, false),
        (
            "same day, stated, after an unstated run",
            unmoved(&[DOCUMENT]),
            true,
            false,
        ),
        ("facet unmoved, warm", unmoved(&[]), false, true),
    ] {
        let mut cache = Cache::at(&root, LOCK, "sha256:rules");
        let ran = run_in(&root, &ctx, &mut cache, &taxonomy());
        cache.write(&root).expect("the cache writes");
        let reported = suspect(&ran);
        assert_eq!(reported.len(), 1, "{label}: {reported:?}");
        assert_eq!(reported[0].patch.is_some(), fixable, "{label}");
        if served {
            assert!(cache.report().hits > 0, "{label}: {:?}", cache.report());
        }
    }
}

/// A wildcard entry names the pattern and how many regular files its digest
/// covers now,
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
    assert!(message.contains("2 regular files"), "{message}");
    assert!(message.contains("not recorded"), "{message}");
    let _ = std::fs::remove_dir_all(&root);
}

/// A file renamed under a governed wildcard, with its bytes unchanged, moves
/// the digest, because the manifest the digest is taken over holds each
/// entry's path as well as its bytes (#1104). The new name keeps the sort
/// order (`lib.sh < main.sh < write.sh`), so a digest over bytes alone would
/// not move, and this case fails under exactly that mutation.
///
/// The recorded digest is the one the rule itself offers, not [`expected`]'s.
/// A resolver that dropped the paths would then still agree with itself
/// before the rename, and the case fails at the rename, which is the claim
/// it holds, rather than at a pinned constant.
#[test]
fn a_renamed_file_inside_a_governed_glob_goes_suspect_with_its_bytes_unchanged() {
    let root = scratch("rename");
    document(&root, TODAY, &["    - .claude/hooks/*.sh".to_string()]);
    let added = at(TODAY).scoped_to(change(&[(DOCUMENT, None)]));
    let first = run_in(&root, &added, &mut Cache::disabled(), &taxonomy());
    let offered = suspect(&first);
    assert_eq!(offered.len(), 1, "{offered:?}");
    let Some(Patch::Half { attributes, .. }) = &offered[0].patch else {
        panic!("no digest offered: {offered:?}");
    };
    let digest = attributes[0].1.clone();
    document(
        &root,
        YESTERDAY,
        &[format!(
            "    - to: .claude/hooks/*.sh\n      verified_revision: \"{digest}\""
        )],
    );
    assert!(suspect(&cold(&root, TODAY)).is_empty());

    std::fs::rename(
        root.join(".claude/hooks/lib.sh"),
        root.join(".claude/hooks/main.sh"),
    )
    .expect("the file renames");
    let ran = cold(&root, TODAY);
    let reported = suspect(&ran);
    assert_eq!(reported.len(), 1, "{reported:?}");
    assert_eq!(
        reported[0].severity,
        headwater_check::finding::Severity::Warn
    );
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
///
/// A list member that names a directory is reported the same way (#1104). One
/// directory member leaves the whole list without a digest, so the list could
/// never go suspect either. The rule gives one finding per entry, and that
/// finding names every directory member and no member that is a file.
#[test]
fn a_directory_literal_is_reported_with_the_wildcard_as_its_remedy() {
    let none: &[&str] = &[];
    for (label, last_verified, entry, directories, files) in [
        (
            "directory-recorded",
            TODAY,
            "    - to: .githooks\n      verified_revision: \"sha256:0\"",
            &[".githooks"][..],
            none,
        ),
        (
            "directory-bare-old",
            YESTERDAY,
            "    - .githooks",
            &[".githooks"][..],
            none,
        ),
        (
            "directory-list-bare-old",
            YESTERDAY,
            "    - [.githooks, .claude/hooks/lib.sh]",
            &[".githooks"][..],
            &[".claude/hooks/lib.sh"][..],
        ),
        (
            "directory-list-recorded",
            TODAY,
            "    - to: [.githooks, .claude/hooks/lib.sh]\n      verified_revision: \"sha256:0\"",
            &[".githooks"][..],
            &[".claude/hooks/lib.sh"][..],
        ),
        (
            "directory-list-two",
            YESTERDAY,
            "    - [.githooks, .claude/hooks]",
            &[".githooks", ".claude/hooks"][..],
            none,
        ),
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
        assert!(message.contains("directory"), "{label}: {message}");
        let remediation = &reported[0].remediation;
        for directory in directories {
            assert!(
                message.contains(&format!("`{directory}`")),
                "{label}: {message}"
            );
            assert!(
                remediation.contains(&format!("`{directory}/**`")),
                "{label}: {remediation}"
            );
        }
        for file in files {
            assert!(
                !message.contains(&format!("`{file}`")),
                "{label}: {message}"
            );
            assert!(
                !remediation.contains(&format!("`{file}/**`")),
                "{label}: {remediation}"
            );
        }
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

/// A `verified` line decides the fix on its own, before the rule looks for a
/// freshness facet. A taxonomy that gives no facet the freshness role has no
/// facet to move, so the statement is the one way a change can state a
/// re-verification there. A change that moves `last_verified` states nothing
/// under it, and a run with no change stamps nothing (#1376).
#[test]
fn a_stated_re_verification_needs_no_freshness_facet() {
    let root = scratch("no-freshness");
    let entries = recorded(&root);
    document(&root, TODAY, &entries);
    write(&root, ".claude/hooks/lib.sh", "refuse() { :; }\n");
    let unroled = taxonomy().replace(
        "  last_verified:\n    role: freshness\n",
        "  last_verified:\n",
    );
    assert_ne!(unroled, taxonomy(), "the role was removed");
    for (label, ctx, fixable) in [
        (
            "stated",
            at(TODAY).scoped_to(stated(&[], &[DOCUMENT])),
            true,
        ),
        (
            "facet moved",
            at(TODAY).scoped_to(change(&[(DOCUMENT, Some(yesterday("hooks", &entries)))])),
            false,
        ),
        ("no change", at(TODAY), false),
    ] {
        let ran = run_in(&root, &ctx, &mut Cache::disabled(), &unroled);
        let reported = suspect(&ran);
        assert_eq!(reported.len(), 1, "{label}: {reported:?}");
        assert_eq!(reported[0].patch.is_some(), fixable, "{label}");
    }
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
