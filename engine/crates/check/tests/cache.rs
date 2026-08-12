// SPDX-License-Identifier: Apache-2.0
//! The standing test of a correctness root: a cache that cannot change a
//! verdict.
//!
//! [Spec 12](../../../../docs/spec/12-check-layer.md#the-correctness-roots):
//! "a cache that can change a verdict is a store under another name.
//! `headwater check --no-cache` and `headwater check` produce byte-identical
//! output, and that comparison is a fixture rather than an assumption."
//!
//! Each test below runs one corpus more than once and holds the renders to
//! each other. A test that only asserted equality would pass over a cache that
//! never served anything, so each one also asserts what the cache did.
//!
//! The corpus is a copy of the fixture tree in this crate's target directory,
//! because two of these tests edit a document and re-run. The tree under
//! `fixtures/check/` is never written to.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{Cache, Detail, Register, Run};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use std::path::{Path, PathBuf};

/// A lock digest. The runs below share a taxonomy, so they share this too, and
/// one test changes it on purpose.
const LOCK: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// A private copy of the fixture tree, under the name of the test that uses
/// it, so that two tests never write to one corpus.
fn corpus_for(case: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(case);
    let _ = std::fs::remove_dir_all(&root);
    copy(&fixtures_dir().join("check"), &root.join("check"));
    root
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the copy directory");
    for entry in std::fs::read_dir(from).expect("the fixture tree reads") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("a file type").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), target).expect("a fixture copies");
            }
        }
    }
}

fn run_over(root: &Path, cache: &mut Cache) -> Run {
    let source = std::fs::read_to_string(fixtures_dir().join("check.taxonomy.yml"))
        .expect("the fixture taxonomy");
    let loaded = headwater_yaml::load(&source).expect("it loads");
    let declared = loaded.value.as_map().expect("a mapping");

    let corpus = Corpus::new(root, "check");
    let taxonomy = Taxonomy::read(declared).expect("the taxonomy reads");
    let declarations = Declarations::read(declared).expect("the declarations read");
    let register = Register::read(declared).expect("the register reads");
    let taken = census::take(&corpus, &taxonomy);
    let graph = Graph::build(
        &taken,
        &declarations,
        &Resolvers::over(&corpus),
        &corpus,
        &Config::default(),
    );
    headwater_check::run(&taken, &graph, &taxonomy, &declarations, &register, cache)
}

/// Three runs over one tree: no cache, a cold cache, a warm one.
///
/// The renders are equal, which is the property. The counters are what stops
/// the test passing vacuously: a cache that stored nothing would satisfy the
/// equality and fail the third assertion.
#[test]
fn a_cached_run_and_a_run_with_no_cache_write_the_same_report() {
    let root = corpus_for("differential");

    let without = run_over(&root, &mut Cache::disabled());

    let mut cold = Cache::at(&root, LOCK);
    let first = run_over(&root, &mut cold);
    cold.write(&root);

    let mut warm = Cache::at(&root, LOCK);
    let second = run_over(&root, &mut warm);

    assert_eq!(
        without.render(Detail::EveryInstance),
        first.render(Detail::EveryInstance)
    );
    assert_eq!(
        without.render(Detail::EveryInstance),
        second.render(Detail::EveryInstance)
    );

    assert_eq!(without.cache.hits, 0, "a disabled cache served something");
    assert_eq!(first.cache.hits, 0, "an empty cache served something");
    assert!(first.cache.misses > 0, "{:?}", first.cache);
    assert_eq!(
        second.cache.hits, first.cache.misses,
        "the warm run did not serve every verdict the cold run stored: {:?}",
        second.cache
    );
    assert_eq!(second.cache.misses, 0, "{:?}", second.cache);

    // A disabled cache keys nothing at all, which is what makes `--no-cache` a
    // path that cannot read an entry rather than one that ignores what it read.
    assert_eq!(without.cache.unkeyed, without.instances.len());

    // What stays unkeyed on a cached run is the skipped instances. A cache
    // holds verdicts, and a skip is the statement that no verdict was reached.
    assert_eq!(
        second.cache.unkeyed,
        second
            .instances
            .iter()
            .filter(|instance| !instance.ran())
            .count()
    );
    assert!(second.cache.unkeyed > 0, "the fixture tree skips nothing");
    assert!(Cache::path(&root).is_file(), "no cache file was written");
}

/// An edit to a document is not served from the entry written before it.
///
/// This is the failure the content hash exists to prevent: a key over a path
/// alone would survive every edit to the file it names, and the second run
/// would then report the verdict of a document that no longer exists.
#[test]
fn an_edited_document_is_evaluated_again() {
    const EDITED: &str = "check/evaluations/gamma.md";
    let root = corpus_for("edited");

    let mut cold = Cache::at(&root, LOCK);
    let before = run_over(&root, &mut cold);
    cold.write(&root);

    // `gamma.md` restates the discriminator on a homogeneous shelf, which is
    // the one placement finding in the tree. Removing that line removes the
    // finding, and nothing else about the corpus moves.
    let path = root.join(EDITED);
    let source = std::fs::read_to_string(&path).expect("the fixture reads");
    let mut edited = String::new();
    for line in source.lines().filter(|line| !line.starts_with("doc_type:")) {
        edited.push_str(line);
        edited.push('\n');
    }
    assert_ne!(source, edited, "the fixture no longer restates its kind");
    std::fs::write(&path, &edited).expect("the copy writes");

    let mut warm = Cache::at(&root, LOCK);
    let after = run_over(&root, &mut warm);

    assert_ne!(
        before.render(Detail::EveryInstance),
        after.render(Detail::EveryInstance),
        "the edit changed nothing, so this test proves nothing"
    );
    assert_eq!(
        after.render(Detail::EveryInstance),
        run_over(&root, &mut Cache::disabled()).render(Detail::EveryInstance),
        "the cached run reported a verdict over the document as it was"
    );

    // One document moved, so exactly the instances that read it were evaluated
    // again — the document-scoped one over it, and every edge whose pair it is
    // an endpoint of. A cache that invalidated everything would also pass the
    // equality above, and it would not be a cache.
    let touched = after
        .instances
        .iter()
        .filter(|instance| instance.ran() && instance.paths().contains(&EDITED))
        .count();
    assert!(touched > 1, "the edited document is read by one instance");
    assert_eq!(after.cache.misses, touched, "{:?}", after.cache);
    assert!(after.cache.hits > touched, "{:?}", after.cache);
}

/// A taxonomy that moved invalidates every entry, with nobody clearing a
/// directory. The lock digest is a key component for exactly this.
#[test]
fn a_lock_that_moved_serves_nothing() {
    let root = corpus_for("relocked");

    let mut cold = Cache::at(&root, LOCK);
    run_over(&root, &mut cold);
    cold.write(&root);

    let mut relocked = Cache::at(&root, "sha256:something-else");
    let after = run_over(&root, &mut relocked);
    assert_eq!(after.cache.hits, 0, "{:?}", after.cache);
    assert!(after.cache.misses > 0, "{:?}", after.cache);

    // And the file that run writes holds only what that run used, so the
    // entries of the old lock are gone rather than accumulating.
    relocked.write(&root);
    let text = std::fs::read_to_string(Cache::path(&root)).expect("the cache reads");
    assert_eq!(text.lines().count(), after.cache.misses + 1, "{text}");
}

/// A cache file this engine cannot read is an empty cache and never a refusal.
///
/// Spec 12 decides every doubtful case toward re-running: a false invalidation
/// costs one run, and a false survival ships an invalid corpus with a green
/// report.
#[test]
fn a_damaged_cache_file_costs_one_run_and_nothing_else() {
    let root = corpus_for("damaged");

    let mut cold = Cache::at(&root, LOCK);
    let expected = run_over(&root, &mut cold);
    cold.write(&root);

    for damage in ["", "headwater check cache 99\n", "not a cache at all\n"] {
        std::fs::write(Cache::path(&root), damage).expect("the cache writes");
        let mut cache = Cache::at(&root, LOCK);
        let run = run_over(&root, &mut cache);
        assert_eq!(
            expected.render(Detail::EveryInstance),
            run.render(Detail::EveryInstance)
        );
        assert_eq!(run.cache.hits, 0, "{damage:?} served an entry");
    }
}
