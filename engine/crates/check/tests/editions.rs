// SPDX-License-Identifier: Apache-2.0
//! The verdict ledger: a rule whose verdicts moved while its `VERSION` did not.
//!
//! `VERSION` is the one component of a cache key that no input supplies
//! ([`headwater_check::scope`] says why). An author raises it by hand when a
//! rule starts to decide something else, and a warm cache keeps serving the
//! earlier edition's verdicts to anyone whose author forgot. That happened to
//! `link.fragment.unresolved`, to `identifier.claim.missing` and to
//! `relation.target.suspect` (#952, #1105). Every other test of this workspace
//! runs under one binary, so none of them could see it: a cold run and a warm
//! run of the same compiled rule agree.
//!
//! `fixtures/editions.ledger` records one row for each rule over each
//! recorded corpus it reaches: the rule's `VERSION`, a fingerprint of the
//! corpus, and a digest of every verdict the rule reaches there. The digest is
//! over what a cache would store for each instance: the instance's identity
//! (its grain and the paths it read, without their byte hashes) and
//! [`headwater_check::cached_form`] of its outcome. A skip is not cached and is
//! digested as the word `skipped`. So a message, a remediation or a patch that
//! changes moves the digest too, because a cache stores those. The fingerprint
//! is over the bytes of every file in the corpus's input directory, its
//! taxonomy, its lock string, its clock and its observations.
//!
//! The test recomputes the ledger and compares it row by row:
//!
//! - a digest that moved while neither `VERSION` nor the fingerprint moved
//!   fails, names the rule, the file that declares it and the corpus, and
//!   **fails under `HEADWATER_BLESS` too**. `DEVELOPING.md` tells an author to
//!   re-record with a blanket `HEADWATER_BLESS=1 cargo test -p
//!   headwater-check`, and a bless that rewrote this row would ship exactly
//!   the #952 defect;
//! - a `VERSION` that moved and a row that is new fail outside bless and are
//!   re-recorded under it;
//! - a corpus whose fingerprint moved at an unchanged `VERSION` is replayed:
//!   the test rebuilds the corpus from git as the ledger recorded it, at the
//!   commit that last wrote the ledger, and runs this binary over it. Where
//!   that replay reaches the recorded digest and count, the corpus moved the
//!   verdicts, so the row fails outside bless and is re-recorded under it. An
//!   author who edits a recorded corpus re-records it without raising any
//!   `VERSION`. Where the replay reaches anything else, the code moved them
//!   too, and the row **fails under bless too**, as the first arm does. A
//!   corpus that git cannot rebuild with the recorded fingerprint fails in
//!   both modes as well, because nothing can tell the two edits apart
//!   (HW-OBL-0222);
//! - a row that reached no instance fails outside bless, because it pins
//!   nothing, and bless drops it.
//!
//! What the ledger cannot see: a change that no recorded corpus exercises, a
//! rule with no instance on any of them (listed in [`UNCOVERED`] with the
//! reason), and a change visible only against a warm cache that an older
//! binary wrote. The last is the digest over the compiled rule
//! that
//! [HW-OBL-0074](../../../../docs/obligations/0074-a-check-version-is-raised-by-hand-and-nothing-catches-a-stale.md)
//! asks for. The ledger approximates it over fixtures and does not close it.
//! A corpus that reads a file outside its input directory and its taxonomy
//! would report an edit to that file as a rule change, and bless would refuse
//! it. Its replay would also lack that file. None of the recorded corpora
//! below reads one.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-check --test editions
//!
//! Read the diff before committing it. A re-recorded row is a rule that now
//! decides something else.

use headwater_census::census;
use headwater_census::shelves::Taxonomy;
use headwater_census::walk::Corpus;
use headwater_check::{
    adoption, basis, command, coverage, dependency, endpoint, harvest, import_pin,
    initial_dependency, lock_stale, outside_root, pin, reciprocity, register, surface, suspect,
    target, verification, Cache, Context, Date, Declared, Observation, Observations, Outcome,
    Register, Run, Shape, RULES,
};
use headwater_graph::anchors::Resolvers;
use headwater_graph::declarations::Declarations;
use headwater_graph::{Config, Graph};
use headwater_yaml::Mapping;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The rules whose instance is an edge. The issue this ledger exists for is
/// one of them, and each must pin at least one verdict.
const EDGE_RULES: [&str; 8] = [
    target::RULE,
    suspect::RULE,
    reciprocity::RULE,
    endpoint::RULE,
    dependency::RULE,
    initial_dependency::RULE,
    basis::RULE,
    verification::RULE,
];

/// The rules no recorded corpus reaches, each with the reason. A rule here has
/// no ledger row, so a change to what it decides passes this test. The list is
/// held honest from both sides: a rule that gains an instance must leave it.
const UNCOVERED: &[(&str, &str)] = &[
    (
        surface::RULE,
        "its cases are unit tests in src/surface.rs, and no recorded corpus declares a \
         `surface` and an adopter shelf",
    ),
    (
        command::RULE,
        "its cases are unit tests in src/command.rs, and no recorded corpus declares a \
         `surface` and an adopter shelf",
    ),
    (
        coverage::RULE,
        "the runner reaches it outside any instance, so no cache entry holds its verdict",
    ),
    (
        outside_root::RULE,
        "the runner reaches it from the census outside any instance, so no cache entry holds \
         its verdict; its cases are in tests/outside_root.rs",
    ),
    (
        pin::RULE,
        "the runner reaches it from the vendored package outside any instance, so no cache \
         entry holds its verdict; its cases are unit tests in src/pin.rs and the CLI case in \
         cli/tests/publish.rs",
    ),
    (
        lock_stale::RULE,
        "the runner reaches it from the lock and its sources outside any instance, so no cache \
         entry holds its verdict; its cases are unit tests in src/lock_stale.rs and the CLI \
         cases in cli/tests/lock_stale.rs",
    ),
    (
        harvest::RULE,
        "the runner reaches it from the pinned exports the caller read, outside any instance, \
         so no cache entry holds its verdict; its cases are unit tests in src/harvest.rs and the \
         CLI cases in cli/tests/harvest.rs",
    ),
    (
        import_pin::RULE,
        "the runner reaches it from the imports snapshots the caller read, outside any instance, \
         so no cache entry holds its verdict; its cases are unit tests in src/import_pin.rs and \
         the CLI cases in cli/tests/import_pin.rs",
    ),
    (
        register::DISPOSITION,
        "the runner reaches it from the taxonomy outside any instance, so no cache entry \
         holds its verdict",
    ),
    (
        register::MECHANISM,
        "the runner reaches it from the taxonomy outside any instance, so no cache entry \
         holds its verdict",
    ),
    (
        register::OBSERVATION,
        "the runner reaches it from the snapshot outside any instance, so no cache entry \
         holds its verdict",
    ),
    (
        adoption::RULE,
        "the runner reaches it from the lock outside any instance, so no cache entry holds \
         its verdict",
    ),
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// The ledger, relative to the crate.
const LEDGER: &str = "fixtures/editions.ledger";

fn ledger_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(LEDGER)
}

fn load_map(path: &Path) -> Mapping {
    let source = std::fs::read_to_string(path).expect("the fixture taxonomy");
    headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone()
}

/// What one recorded corpus is run with. Each keeps the clock and the lock
/// string of the test that owns the corpus, so a verdict here is the verdict
/// that test records.
struct Recorded {
    /// The name a ledger message prints, relative to `fixtures/`.
    label: &'static str,
    /// The corpus base, relative to the fixtures root the run reads.
    base: &'static str,
    name: &'static str,
    taxonomy: &'static str,
    /// The lock string, or `None` for the hex of the taxonomy's own bytes.
    lock: Option<&'static str>,
    clock: &'static str,
    observations: Observations,
    /// Whether the run reads `check-rule` anchors and the claim store under
    /// the base, as `tests/fixtures.rs` does for the check tree.
    full: bool,
    /// The directory whose every file the run can read, relative to the
    /// fixtures root. See [`fingerprint`].
    inputs: &'static str,
}

fn recorded() -> Vec<Recorded> {
    vec![
        Recorded {
            label: "check",
            base: "",
            name: "check",
            taxonomy: "check.taxonomy.yml",
            lock: None,
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: true,
            inputs: "check",
        },
        Recorded {
            label: "terminal-dependency",
            base: "",
            name: "terminal-dependency",
            taxonomy: "terminal-dependency.taxonomy.yml",
            lock: Some("sha256:terminal-dependency-fixture"),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
            inputs: "terminal-dependency",
        },
        Recorded {
            label: "state-set-twice",
            base: "",
            name: "state-set-twice",
            taxonomy: "state-set-twice.taxonomy.yml",
            lock: Some("sha256:state-set-twice-fixture"),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
            inputs: "state-set-twice",
        },
        // The corpus of `lifecycle.state.not_set_by_edge`, which reaches both
        // directions of an edge, a finding with a patch and one with none, and
        // two skips. It is its own corpus rather than one more rule on a shared
        // one (HW-OBL-0222, #1198).
        Recorded {
            label: "state-not-set-by-edge",
            base: "",
            name: "state-not-set-by-edge",
            taxonomy: "state-not-set-by-edge.taxonomy.yml",
            lock: Some("sha256:state-not-set-by-edge-fixture"),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
            inputs: "state-not-set-by-edge",
        },
        // The corpus of `relation.pair.invalid`: a conflict both ends declare,
        // a one-sided one, a superseded end, a relation with no condition, a
        // condition over a facet other than the state facet, and one over two
        // facets that holds at one end alone (HW-OBL-0042, #1491).
        Recorded {
            label: "conflicting-pair",
            base: "",
            name: "conflicting-pair",
            taxonomy: "conflicting-pair.taxonomy.yml",
            lock: Some("sha256:conflicting-pair-fixture"),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
            inputs: "conflicting-pair",
        },
        // The corpus of `relation.target.is_source`: an entry that names its
        // own document through either half, a control that names another
        // document, and a self-path that binds to nothing (#1232). Five
        // `code_path` anchors: two literals onto the declaring file, one of
        // them spelled in a non-canonical form, which are reported, and a
        // literal onto another file, a wildcard over the own file and a list
        // that holds it, which pass (#1350).
        Recorded {
            label: "self-target",
            base: "",
            name: "self-target",
            taxonomy: "self-target.taxonomy.yml",
            lock: Some("sha256:self-target-fixture"),
            clock: "2026-08-12",
            observations: Observations::empty(),
            full: false,
            inputs: "self-target",
        },
        Recorded {
            label: "acceptance-criterion-proven",
            base: "",
            name: "acceptance-criterion-proven",
            taxonomy: "acceptance-criterion-proven.taxonomy.yml",
            lock: Some("sha256:verification-suspect-fixture"),
            clock: "2026-08-12",
            // A snapshot whose digest is not the criterion's, so the rule
            // reaches its suspect arm and a finding with a message.
            observations: Observations::of(vec![Observation::Verification {
                verification: "ACP-FIX-verification-one".to_string(),
                commit: "788885a9".to_string(),
                criterion_digest: "sha256:0".to_string(),
            }]),
            full: false,
            inputs: "acceptance-criterion-proven",
        },
        Recorded {
            label: "editions/governs-suspect",
            base: "editions",
            name: "governs-suspect",
            taxonomy: "governs-suspect.taxonomy.yml",
            lock: Some("sha256:editions-governs-suspect"),
            clock: "2026-09-24",
            observations: Observations::empty(),
            full: false,
            inputs: "editions",
        },
    ]
}

/// The lock string a recorded corpus runs with, read from `root`.
fn lock_of(recorded: &Recorded, root: &Path) -> String {
    match recorded.lock {
        Some(lock) => lock.to_string(),
        None => {
            let source = std::fs::read(root.join(recorded.taxonomy)).expect("the fixture taxonomy");
            headwater_hash::hex(&source)
        }
    }
}

/// One recorded corpus run under this binary, read from `root`: the fixtures
/// directory, or a copy of it that git rebuilt (see [`replay`]).
fn run(recorded: &Recorded, root: &Path) -> Run {
    let corpus = Corpus::new(root.join(recorded.base), recorded.name);
    let lock = lock_of(recorded, root);
    let root = load_map(&root.join(recorded.taxonomy));
    let taxonomy = Taxonomy::read(&root).expect("the taxonomy reads");
    let declarations = Declarations::read(&root).expect("the declarations read");
    let register = Register::read(&root).expect("the register reads");
    let shape = Shape::read(&root).expect("the shape reads");
    let taken = census::take(&corpus, &taxonomy);
    let config = Config::default();
    let resolvers = if recorded.full {
        Resolvers::over(&corpus)
            .with(Box::new(headwater_check::anchors::Rules::shipped()))
            .expect("the check-rule resolver is the only one of its name")
    } else {
        Resolvers::over(&corpus)
    };
    let claims = if recorded.full {
        headwater_check::claim::Claims::at(&corpus.base)
    } else {
        headwater_check::claim::Claims::empty()
    };
    let graph = Graph::build(&taken, &declarations, &resolvers, &corpus, &config);
    let source = format!("engine/crates/check/fixtures/{}", recorded.taxonomy);
    headwater_check::run(
        &taken,
        &graph,
        &Declared {
            lock: &lock,
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            observations: &recorded.observations,
            pin: None,
            lock_stale: None,
            harvests: &[],
            imports: &[],
            adoption: None,
            source: &source,
        },
        &claims,
        &Context::at(Date::parse(recorded.clock).expect("the pinned date")),
        &mut Cache::disabled(),
    )
}

/// The fingerprint of what one recorded corpus hands its rules: the bytes of
/// every file under its input directory, the bytes of its taxonomy, its lock
/// string, its clock and its observations.
///
/// A row records it so that a corpus edit and a rule edit are told apart. A
/// verdict that moved over an unchanged fingerprint moved because the code
/// moved. A verdict that moved over a moved fingerprint may have moved for
/// either reason, and bless re-records it.
fn fingerprint(recorded: &Recorded, root: &Path) -> String {
    let inputs = root.join(recorded.inputs);
    let mut files = Vec::new();
    walk(&inputs, &mut files);
    files.sort();
    let mut text = String::new();
    for path in files {
        let relative = path
            .strip_prefix(&inputs)
            .expect("a walked path is under its root")
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = std::fs::read(&path).expect("a corpus file reads");
        text.push_str(&format!(
            "file\t{relative}\t{}\n",
            headwater_hash::digest(&bytes)
        ));
    }
    let taxonomy = std::fs::read(root.join(recorded.taxonomy)).expect("the fixture taxonomy");
    text.push_str(&format!(
        "taxonomy\t{}\nlock\t{}\nclock\t{}\nobservations\t{:?}\n",
        headwater_hash::digest(&taxonomy),
        lock_of(recorded, root),
        recorded.clock,
        recorded.observations,
    ));
    headwater_hash::digest(text.as_bytes())
}

fn walk(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("a corpus directory reads") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            walk(&path, into);
        } else {
            into.push(path);
        }
    }
}

/// One rule over one recorded corpus: what the ledger records.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    rule: String,
    corpus: String,
    version: u32,
    /// The corpus's [`fingerprint`] when the row was recorded.
    inputs: String,
    digest: String,
    instances: usize,
}

impl Row {
    fn key(&self) -> (&str, &str) {
        (&self.rule, &self.corpus)
    }
}

/// The rows the recorded corpora give under this binary, sorted by rule and
/// then by corpus.
fn compute() -> Vec<Row> {
    rows(&recorded(), &fixtures_dir())
}

/// The rows that `corpora` give under this binary when read from `root`.
fn rows(corpora: &[Recorded], root: &Path) -> Vec<Row> {
    let mut versions: BTreeMap<&'static str, u32> = BTreeMap::new();
    let mut lines: BTreeMap<(&'static str, &'static str), Vec<String>> = BTreeMap::new();
    let mut inputs: BTreeMap<&'static str, String> = BTreeMap::new();
    for recorded in corpora {
        let ran = run(recorded, root);
        inputs.insert(recorded.label, fingerprint(recorded, root));
        for (rule, version) in &ran.read_set.versions {
            let earlier = versions.insert(rule, *version);
            assert!(
                earlier.is_none_or(|earlier| earlier == *version),
                "{rule} reported two versions in one binary"
            );
        }
        for instance in &ran.instances {
            let reads: Vec<&str> = instance.reads.iter().map(|i| i.path.as_str()).collect();
            let verdict = headwater_check::cached_form(&instance.outcome).unwrap_or_else(|| {
                debug_assert!(matches!(instance.outcome, Outcome::Skipped(_)));
                "skipped".to_string()
            });
            lines
                .entry((instance.rule, recorded.label))
                .or_default()
                .push(format!(
                    "{:?}\t{}\t{verdict}",
                    instance.grain,
                    reads.join(",")
                ));
        }
    }
    lines
        .into_iter()
        .map(|((rule, corpus), mut instances)| {
            instances.sort_unstable();
            let mut text = instances.join("\n");
            text.push('\n');
            Row {
                rule: rule.to_string(),
                corpus: corpus.to_string(),
                version: *versions
                    .get(rule)
                    .unwrap_or_else(|| panic!("{rule} reached a verdict and reported no version")),
                inputs: inputs[corpus].clone(),
                digest: headwater_hash::digest(text.as_bytes()),
                instances: instances.len(),
            }
        })
        .collect()
}

fn computed() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(compute)
}

const HEADER: &str = "\
# The verdict ledger of tests/editions.rs: rule, recorded corpus, VERSION,
# fingerprint of the corpus, digest of every verdict, instance count.
# HEADWATER_BLESS=1 re-records a row whose VERSION moved, or whose corpus moved
# where the corpus as recorded still gives the recorded verdicts. It never
# re-records a row whose verdicts moved over the same corpus at the same VERSION:
# raise VERSION for that.
";

fn render(rows: &[Row]) -> String {
    let mut text = HEADER.to_string();
    for row in rows {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row.rule, row.corpus, row.version, row.inputs, row.digest, row.instances
        ));
    }
    text
}

/// A ledger as rows, and a line it cannot read as a failure.
fn parse(text: &str) -> (Vec<Row>, Vec<String>) {
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let row = match fields.as_slice() {
            [rule, corpus, version, inputs, digest, instances] => {
                match (version.parse(), instances.parse()) {
                    (Ok(version), Ok(instances)) => Some(Row {
                        rule: rule.to_string(),
                        corpus: corpus.to_string(),
                        version,
                        inputs: inputs.to_string(),
                        digest: digest.to_string(),
                        instances,
                    }),
                    _ => None,
                }
            }
            _ => None,
        };
        match row {
            Some(row) => rows.push(row),
            None => failures.push(format!("the ledger line `{line}` does not read")),
        }
    }
    (rows, failures)
}

/// The file that declares a rule: the one whose `pub const <NAME>: &str =`
/// line holds the rule's name. Found by that line so that no second table of
/// rules sits beside [`RULES`].
fn declaring_file(rule: &str) -> String {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let declaration = format!(": &str = \"{rule}\";");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&source)
        .expect("the source directory reads")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();
    files
        .into_iter()
        .find(|path| {
            std::fs::read_to_string(path).is_ok_and(|text| {
                text.lines().any(|line| {
                    line.trim_start().starts_with("pub const ") && line.ends_with(&declaration)
                })
            })
        })
        .map(|path| {
            format!(
                "engine/crates/check/src/{}",
                path.file_name().expect("a file name").to_string_lossy()
            )
        })
        .unwrap_or_else(|| "the module that declares it".to_string())
}

/// What this binary reaches for one rule over a corpus rebuilt as the ledger
/// recorded it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Replayed {
    digest: String,
    instances: usize,
}

/// The replays of the rows whose corpus moved, keyed by rule and corpus. A
/// row with no entry is a row whose recorded corpus could not be rebuilt.
type Replays = BTreeMap<(String, String), Replayed>;

/// What a comparison decided: the ledger to write under bless, and every
/// failure. Under bless the ledger is written and the failures still fail.
#[derive(Debug)]
struct Judgement {
    ledger: String,
    failures: Vec<String>,
}

/// Compare a recorded ledger with the rows this binary gives.
///
/// A pure function of its arguments, because a compiled `VERSION` cannot
/// change inside one process: the tests below stand a hand-made ledger in for
/// the one an older binary recorded, as `tests/cache.rs` stands in a second
/// rule list for an upgrade.
///
/// The order of the arms is the point. A moved `VERSION` is a reason for a
/// verdict to move, so it lets bless re-record the row with no replay. A
/// moved corpus is a reason too, but only when this binary over the corpus as
/// the ledger recorded it still reaches the recorded digest and count: then
/// the corpus moved the verdicts, and bless re-records the row. A replay that
/// differs means that the code moved them too, and a row with no replay means
/// that nothing can tell. Both are refused in both modes, as a moved verdict
/// over an unmoved corpus is.
fn judge(recorded: &str, computed: &[Row], replays: &Replays, bless: bool) -> Judgement {
    let (rows, mut failures) = parse(recorded);
    let mut old: BTreeMap<(&str, &str), &Row> = rows.iter().map(|row| (row.key(), row)).collect();
    let mut kept = Vec::new();
    for row in computed {
        let (take, failure) = match old.remove(&row.key()) {
            None => (
                true,
                format!(
                    "{} has no row for {} in the ledger: record it with HEADWATER_BLESS=1",
                    row.rule, row.corpus
                ),
            ),
            Some(was) if was.version != row.version => (
                true,
                format!(
                    "{} is at VERSION {} and the ledger records VERSION {} over {}: re-record \
                     with HEADWATER_BLESS=1",
                    row.rule, row.version, was.version, row.corpus
                ),
            ),
            Some(was) if was.inputs != row.inputs => {
                match replays.get(&(was.rule.clone(), was.corpus.clone())) {
                    Some(replayed)
                        if replayed.digest == was.digest && replayed.instances == was.instances =>
                    {
                        (
                            true,
                            format!(
                                "the recorded corpus {} changed, so the row for {} over it is \
                                 stale: re-record with HEADWATER_BLESS=1",
                                row.corpus, row.rule
                            ),
                        )
                    }
                    Some(_) => {
                        failures.push(format!(
                            "{} changed its verdicts over {} as the ledger recorded it, at \
                             VERSION {}, and the corpus moved too: raise VERSION in {}, then \
                             re-record with HEADWATER_BLESS=1. A warm cache keeps serving what \
                             VERSION {} decided, and HEADWATER_BLESS does not re-record this row",
                            row.rule,
                            row.corpus,
                            row.version,
                            declaring_file(&row.rule),
                            row.version,
                        ));
                        kept.push(was.clone());
                        continue;
                    }
                    None => {
                        failures.push(format!(
                            "the recorded corpus {} changed, and this test could not rebuild it \
                             from git as the ledger recorded it, so it cannot tell a corpus edit \
                             from a change to {} at VERSION {}. Restore {} with `git checkout`, \
                             or commit the ledger, and re-record with HEADWATER_BLESS=1; or raise \
                             VERSION in {}. HEADWATER_BLESS does not re-record this row",
                            row.corpus,
                            row.rule,
                            row.version,
                            LEDGER,
                            declaring_file(&row.rule),
                        ));
                        kept.push(was.clone());
                        continue;
                    }
                }
            }
            Some(was) if was.digest != row.digest || was.instances != row.instances => {
                failures.push(format!(
                    "{} changed its verdicts over {} at VERSION {}, and neither VERSION nor the \
                     corpus moved: raise VERSION in {}, then re-record with HEADWATER_BLESS=1. A \
                     warm cache keeps serving what VERSION {} decided, and HEADWATER_BLESS does \
                     not re-record this row",
                    row.rule,
                    row.corpus,
                    row.version,
                    declaring_file(&row.rule),
                    row.version,
                ));
                kept.push(was.clone());
                continue;
            }
            Some(was) => {
                kept.push(was.clone());
                continue;
            }
        };
        if !bless {
            failures.push(failure);
        }
        if take {
            kept.push(row.clone());
        }
    }
    for ((rule, corpus), _) in old {
        if !bless {
            failures.push(format!(
                "{rule} has a row for {corpus} in the ledger and reached no verdict there, so the \
                 row pins nothing: re-record with HEADWATER_BLESS=1, and list the rule in \
                 UNCOVERED with the reason if no corpus reaches it now"
            ));
        }
    }
    kept.sort_by(|a, b| a.key().cmp(&b.key()));
    Judgement {
        ledger: render(&kept),
        failures,
    }
}

/// A git command run in the crate directory, or `None` if it did not run or
/// failed.
fn git(args: &[&str]) -> Option<Vec<u8>> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    output.status.success().then_some(output.stdout)
}

/// The commit that last wrote the ledger. Its tree holds the corpora as the
/// ledger records them, whether the author blessed before committing a corpus
/// edit or committed the edit and blesses after it.
fn ledger_commit() -> Option<String> {
    let out = git(&["log", "-1", "--format=%H", "--", LEDGER])?;
    let commit = String::from_utf8(out).ok()?.trim().to_string();
    (!commit.is_empty()).then_some(commit)
}

/// A directory of its own under the system temp directory. Keyed on the pid,
/// a counter and the clock, because cargo runs the cases as threads of one
/// process.
fn scratch() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    std::env::temp_dir().join(format!(
        "headwater-editions-{}-{}-{nanos}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}

/// Write the files under `fixtures/<path>` at `commit` into `into`, keeping
/// their paths below `fixtures/`. `None` if git cannot read the path there.
fn extract(commit: &str, path: &str, into: &Path) -> Option<()> {
    let listed = git(&[
        "ls-tree",
        "-r",
        "-z",
        "--name-only",
        commit,
        "--",
        &format!("fixtures/{path}"),
    ])?;
    let names: Vec<&[u8]> = listed
        .split(|b| *b == 0)
        .filter(|n| !n.is_empty())
        .collect();
    if names.is_empty() {
        return None;
    }
    for name in names {
        let name = std::str::from_utf8(name).ok()?;
        let bytes = git(&["show", &format!("{commit}:./{name}")])?;
        let target = into.join(name.strip_prefix("fixtures/")?);
        std::fs::create_dir_all(target.parent()?).ok()?;
        std::fs::write(&target, bytes).ok()?;
    }
    Some(())
}

/// The rows this binary gives over one recorded corpus rebuilt from git at
/// `commit`: its input directory and its taxonomy, which are what
/// [`fingerprint`] reads. `None` if git cannot rebuild it, if the rebuilt
/// corpus's fingerprint is not `expected`, or if this binary cannot run it.
/// The fingerprint check is what makes a replay the corpus the ledger
/// recorded, rather than whatever that commit held.
///
/// The `check` corpus also reads `check-rule` anchors, which are compiled
/// into the binary, and a claim store under its base, which `fixtures/` does
/// not have. A corpus that read a file outside these two paths would replay
/// without it, reach another digest, and be refused, which is the safe
/// direction.
fn replay(commit: &str, label: &str, expected: &str) -> Option<Vec<Row>> {
    let recorded = recorded().into_iter().find(|r| r.label == label)?;
    let root = scratch();
    let replayed = (|| {
        extract(commit, recorded.inputs, &root)?;
        extract(commit, recorded.taxonomy, &root)?;
        if fingerprint(&recorded, &root) != expected {
            return None;
        }
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rows(std::slice::from_ref(&recorded), &root)
        }))
        .ok()
    })();
    let _ = std::fs::remove_dir_all(&root);
    replayed
}

/// For every row of `recorded` whose corpus moved at an unchanged `VERSION`,
/// what this binary reaches over that corpus rebuilt at `commit`. A row the
/// replay cannot answer has no entry, and [`judge`] refuses it.
fn replays(recorded: &str, computed: &[Row], commit: &str) -> Replays {
    let (rows, _) = parse(recorded);
    let now: BTreeMap<(&str, &str), &Row> = computed.iter().map(|row| (row.key(), row)).collect();
    let moved: Vec<&Row> = rows
        .iter()
        .filter(|was| {
            now.get(&was.key())
                .is_some_and(|row| row.version == was.version && row.inputs != was.inputs)
        })
        .collect();
    let mut corpora: BTreeMap<(&str, &str), Option<Vec<Row>>> = BTreeMap::new();
    for was in &moved {
        corpora
            .entry((was.corpus.as_str(), was.inputs.as_str()))
            .or_insert_with(|| replay(commit, &was.corpus, &was.inputs));
    }
    let mut replays = Replays::new();
    for was in moved {
        let Some(Some(rows)) = corpora.get(&(was.corpus.as_str(), was.inputs.as_str())) else {
            continue;
        };
        let replayed = rows.iter().find(|row| row.key() == was.key()).map_or(
            Replayed {
                digest: "no verdict".to_string(),
                instances: 0,
            },
            |row| Replayed {
                digest: row.digest.clone(),
                instances: row.instances,
            },
        );
        replays.insert((was.rule.clone(), was.corpus.clone()), replayed);
    }
    replays
}

/// The replays the standing test hands [`judge`]: each moved corpus rebuilt
/// at the commit that last wrote the ledger, and none where git cannot name
/// that commit.
fn standing_replays(recorded: &str, computed: &[Row]) -> Replays {
    match ledger_commit() {
        Some(commit) => replays(recorded, computed, &commit),
        None => Replays::new(),
    }
}

fn bless() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

/// The standing test: the ledger on disk against this binary.
#[test]
fn every_ledgered_rule_decides_what_its_version_recorded() {
    let path = ledger_path();
    let recorded = std::fs::read_to_string(&path).unwrap_or_default();
    let judgement = judge(
        &recorded,
        computed(),
        &standing_replays(&recorded, computed()),
        bless(),
    );
    if bless() && judgement.ledger != recorded {
        std::fs::write(&path, &judgement.ledger).expect("cannot write the ledger");
    }
    assert!(
        judgement.failures.is_empty(),
        "\n{}\n",
        judgement.failures.join("\n")
    );
}

/// Every rule is ledgered or listed as uncovered with a reason, and never both.
/// Every edge rule is ledgered.
#[test]
fn every_rule_is_ledgered_or_named_as_uncovered() {
    let ledgered: Vec<&str> = computed().iter().map(|row| row.rule.as_str()).collect();
    let uncovered: Vec<&str> = UNCOVERED.iter().map(|(rule, _)| *rule).collect();
    for rule in EDGE_RULES {
        assert!(
            ledgered.contains(&rule),
            "the edge rule {rule} reaches no verdict on any recorded corpus"
        );
    }
    for rule in RULES {
        match (ledgered.contains(&rule), uncovered.contains(&rule)) {
            (true, true) => panic!("{rule} reaches a verdict now: take it out of UNCOVERED"),
            (false, false) => panic!(
                "{rule} reaches no verdict on any recorded corpus: add a fixture that reaches \
                 it, or list it in UNCOVERED with the reason"
            ),
            _ => {}
        }
    }
    for (rule, reason) in UNCOVERED {
        assert!(RULES.contains(rule), "{rule} in UNCOVERED is not a rule");
        assert!(!reason.is_empty(), "{rule} in UNCOVERED gives no reason");
    }
}

/// Every ledgered rule is traced to the file whose `pub const` declares it,
/// and that file is the module the rule's constant lives in.
#[test]
fn every_ledgered_rule_names_the_file_that_declares_it() {
    for (rule, file) in [
        (suspect::RULE, "suspect.rs"),
        (headwater_check::voice::RULE, "voice.rs"),
        (headwater_check::retired::RULE, "retired.rs"),
        (headwater_check::sections::RULE, "sections.rs"),
        (headwater_check::claim::MISSING, "claim.rs"),
    ] {
        assert_eq!(
            declaring_file(rule),
            format!("engine/crates/check/src/{file}"),
            "{rule}"
        );
    }
    for row in computed() {
        assert_ne!(
            declaring_file(&row.rule),
            "the module that declares it",
            "{} has no `pub const` declaration this test can find",
            row.rule
        );
    }
}

/// The sentence a session editing a rule meets beside the rule's version
/// constant, so that the rule of this file reaches it in the file it edits
/// (#1976).
const RAISE_SENTENCE: &str = "raise the rule version when a rule's behavior changes";

/// Every `VERSION: u32` constant under `src/`, and each declaration of one in
/// a check trait, carries [`RAISE_SENTENCE`] in its own doc block. A
/// cross-reference to another constant does not count, because a session that
/// edits one rule reads the lines above its constant and follows nothing.
#[test]
fn every_version_constant_states_that_a_changed_rule_raises_it() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&source)
        .expect("the source directory reads")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();
    let mut found = 0;
    let mut missing = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("a source file reads");
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let code = line.trim_start();
            let is_constant = !code.starts_with("//")
                && (code.contains("VERSION: u32 =") || code.contains("VERSION: u32;"));
            if !is_constant {
                continue;
            }
            found += 1;
            let mut doc = Vec::new();
            for above in lines[..index].iter().rev() {
                let above = above.trim_start();
                if let Some(rest) = above.strip_prefix("///") {
                    doc.push(rest.trim());
                } else if above.starts_with("#[") {
                    continue;
                } else {
                    break;
                }
            }
            doc.reverse();
            if !doc.join(" ").to_lowercase().contains(RAISE_SENTENCE) {
                missing.push(format!(
                    "engine/crates/check/src/{}:{}",
                    path.file_name().expect("a file name").to_string_lossy(),
                    index + 1
                ));
            }
        }
    }
    assert!(
        found >= 40,
        "the walk found {found} version constants, fewer than the 40 it expects"
    );
    assert!(
        missing.is_empty(),
        "{} of {found} version constants do not say \"{RAISE_SENTENCE}\" in their own doc block:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

/// A copy of the computed rows with one row changed.
fn with(rows: &[Row], rule: &str, corpus: &str, change: impl Fn(&mut Row)) -> Vec<Row> {
    let mut found = false;
    let changed = rows
        .iter()
        .map(|r| {
            let mut r = r.clone();
            if r.rule == rule && r.corpus == corpus {
                change(&mut r);
                found = true;
            }
            r
        })
        .collect();
    assert!(found, "{rule} has a row over {corpus}");
    changed
}

const SUSPECT_CORPUS: &str = "editions/governs-suspect";

/// The decisive case, #952 as data: the ledger an older binary wrote holds
/// `relation.target.suspect` over the same corpus at the version this binary
/// runs, with a digest this binary does not reach. That fails, names the rule,
/// its file and the corpus, and fails under bless too, where the row stays as
/// it was.
#[test]
fn a_moved_digest_at_an_unchanged_version_fails_and_bless_keeps_the_row() {
    let rows = computed();
    let version = rows
        .iter()
        .find(|row| row.rule == suspect::RULE)
        .expect("relation.target.suspect is ledgered")
        .version;
    let older = with(rows, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.digest = "sha256:recorded-by-an-older-binary".to_string();
    });
    let ledger = render(&older);

    for bless in [false, true] {
        let judgement = judge(&ledger, rows, &Replays::new(), bless);
        assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
        let message = &judgement.failures[0];
        assert!(message.contains(suspect::RULE), "{message}");
        assert!(message.contains(&format!("VERSION {version}")), "{message}");
        assert!(
            message.contains("engine/crates/check/src/suspect.rs"),
            "{message}"
        );
        assert!(message.contains(SUSPECT_CORPUS), "{message}");
        assert_eq!(
            judgement.ledger, ledger,
            "bless={bless} rewrote a row whose digest moved at an unchanged VERSION"
        );
    }
}

/// A corpus edit is not a rule edit. One more `governs` entry moves the
/// suspect rule's digest, its instance count and the corpus fingerprint
/// together, at an unchanged VERSION, and this binary over the corpus as the
/// ledger recorded it reaches the recorded digest and count. Outside bless
/// that asks for a re-record, and bless takes it without a raised VERSION.
#[test]
fn a_moved_corpus_asks_for_a_re_record_and_bless_takes_it() {
    let rows = computed();
    let older = with(rows, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.inputs = "sha256:the-corpus-before-an-edit".to_string();
        r.digest = "sha256:the-verdicts-before-an-edit".to_string();
        r.instances -= 1;
    });
    let ledger = render(&older);
    let was = older
        .iter()
        .find(|row| row.rule == suspect::RULE && row.corpus == SUSPECT_CORPUS)
        .expect("relation.target.suspect has a row over its corpus");
    let replays = Replays::from([(
        (suspect::RULE.to_string(), SUSPECT_CORPUS.to_string()),
        Replayed {
            digest: was.digest.clone(),
            instances: was.instances,
        },
    )]);

    let judgement = judge(&ledger, rows, &replays, false);
    assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
    assert!(judgement.failures[0].contains(SUSPECT_CORPUS));
    assert!(judgement.failures[0].contains("re-record"));
    assert!(!judgement.failures[0].contains("raise VERSION"));

    let judgement = judge(&ledger, rows, &replays, true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(rows));

    // The replayed digest matches and its count does not: the code moved the
    // count, and bless refuses.
    let miscounted = Replays::from([(
        (suspect::RULE.to_string(), SUSPECT_CORPUS.to_string()),
        Replayed {
            digest: was.digest.clone(),
            instances: was.instances + 1,
        },
    )]);
    let judgement = judge(&ledger, rows, &miscounted, true);
    assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
    assert!(judgement.failures[0].contains("raise VERSION"));
    assert_eq!(judgement.ledger, ledger);
}

/// A moved corpus that could not be rebuilt as the ledger recorded it has no
/// replay, so nothing can tell a corpus edit from a rule edit. The row is
/// refused in both modes, the message says why, and bless keeps the row.
#[test]
fn a_moved_corpus_that_cannot_be_replayed_fails_and_bless_keeps_the_row() {
    let rows = computed();
    let older = with(rows, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.inputs = "sha256:the-corpus-before-an-edit".to_string();
        r.digest = "sha256:the-verdicts-before-an-edit".to_string();
        r.instances -= 1;
    });
    let ledger = render(&older);

    for bless in [false, true] {
        let judgement = judge(&ledger, rows, &Replays::new(), bless);
        assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
        let message = &judgement.failures[0];
        assert!(message.contains(suspect::RULE), "{message}");
        assert!(message.contains(SUSPECT_CORPUS), "{message}");
        assert!(message.contains("could not rebuild"), "{message}");
        assert!(message.contains(LEDGER), "{message}");
        assert_eq!(
            judgement.ledger, ledger,
            "bless={bless} rewrote a row whose corpus it could not replay"
        );
    }
}

/// The replay itself, over git: `HEAD`'s tree is the tree `HEAD`'s ledger
/// records on any green commit, and CI's checkout holds `HEAD` alone, so this
/// case walks no history. The suspect corpus rebuilt at `HEAD` reaches the
/// digest and count of every row `HEAD`'s ledger records over it. A replay
/// whose fingerprint is not the recorded one is no replay.
#[test]
fn a_corpus_rebuilt_from_head_reaches_the_rows_head_records() {
    let ledger = String::from_utf8(
        git(&["show", &format!("HEAD:./{LEDGER}")]).expect("git reads the ledger at HEAD"),
    )
    .expect("the ledger is text");
    let (recorded, failures) = parse(&ledger);
    assert!(failures.is_empty(), "{failures:?}");
    let over: Vec<&Row> = recorded
        .iter()
        .filter(|row| row.corpus == SUSPECT_CORPUS)
        .collect();
    assert!(
        !over.is_empty(),
        "HEAD's ledger has rows over {SUSPECT_CORPUS}"
    );

    let replayed = replay("HEAD", SUSPECT_CORPUS, &over[0].inputs)
        .expect("the suspect corpus rebuilds at HEAD with the recorded fingerprint");
    for was in &over {
        let row = replayed
            .iter()
            .find(|row| row.key() == was.key())
            .unwrap_or_else(|| panic!("{} reaches no verdict over the replay", was.rule));
        assert_eq!(row.inputs, was.inputs, "{}", was.rule);
        assert_eq!(row.digest, was.digest, "{}", was.rule);
        assert_eq!(row.instances, was.instances, "{}", was.rule);
    }
    assert_eq!(replayed.len(), over.len());

    assert!(
        replay("HEAD", SUSPECT_CORPUS, "sha256:not-the-recorded-corpus").is_none(),
        "a replay whose fingerprint is not the recorded one was trusted"
    );
}

/// `replays` end to end, at `HEAD`: a computed row whose corpus moved gets
/// the replay of the corpus `HEAD` records. Where the recorded verdicts are
/// what that replay reaches, bless re-records the row. Where they are not,
/// the code moved them, and bless refuses.
#[test]
fn replays_answers_a_moved_corpus_from_the_corpus_head_records() {
    let ledger = String::from_utf8(
        git(&["show", &format!("HEAD:./{LEDGER}")]).expect("git reads the ledger at HEAD"),
    )
    .expect("the ledger is text");
    let (recorded, _) = parse(&ledger);
    let moved = with(&recorded, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.inputs = "sha256:the-corpus-after-an-edit".to_string();
        r.digest = "sha256:the-verdicts-after-an-edit".to_string();
    });

    let answered = replays(&ledger, &moved, "HEAD");
    assert_eq!(answered.len(), 1, "{answered:?}");
    let judgement = judge(&ledger, &moved, &answered, true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(&moved));

    let rewritten = render(&with(&recorded, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.digest = "sha256:verdicts-the-recorded-corpus-does-not-give".to_string();
    }));
    let answered = replays(&rewritten, &moved, "HEAD");
    let judgement = judge(&rewritten, &moved, &answered, true);
    assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
    assert!(judgement.failures[0].contains("raise VERSION"));
    assert_eq!(judgement.ledger, rewritten);

    // A recorded row whose rule reaches no verdict over the replayed corpus:
    // the replay answers "no verdict", which is not what the row records, and
    // bless refuses it.
    let head = recorded
        .iter()
        .find(|row| row.corpus == SUSPECT_CORPUS)
        .expect("HEAD's ledger has a row over the suspect corpus");
    let phantom = Row {
        rule: "no.such.rule".to_string(),
        corpus: SUSPECT_CORPUS.to_string(),
        version: 1,
        inputs: head.inputs.clone(),
        digest: "sha256:a-verdict-no-rule-reaches".to_string(),
        instances: 1,
    };
    let mut with_phantom = recorded.clone();
    with_phantom.push(phantom.clone());
    let ledger = render(&with_phantom);
    let mut moved = moved.clone();
    moved.push(Row {
        inputs: "sha256:the-corpus-after-an-edit".to_string(),
        ..phantom
    });
    let answered = replays(&ledger, &moved, "HEAD");
    assert_eq!(
        answered.get(&("no.such.rule".to_string(), SUSPECT_CORPUS.to_string())),
        Some(&Replayed {
            digest: "no verdict".to_string(),
            instances: 0,
        })
    );
    let judgement = judge(&ledger, &moved, &answered, true);
    assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
    assert!(judgement.failures[0].contains("no.such.rule"));
    assert!(judgement.failures[0].contains("raise VERSION"));
}

/// The decisive case of HW-OBL-0222: one commit edits the corpus and changes
/// the rule. The fingerprint, the digest and the instance count of
/// `relation.target.suspect` all moved at an unchanged VERSION, and this
/// binary over the corpus as the ledger recorded it reaches a digest other
/// than the recorded one. So the code moved the verdicts, and the row is
/// refused in both modes, where it stays as it was.
#[test]
fn a_moved_corpus_whose_verdicts_also_moved_over_the_recorded_corpus_fails_and_bless_keeps_the_row()
{
    let rows = computed();
    let version = rows
        .iter()
        .find(|row| row.rule == suspect::RULE)
        .expect("relation.target.suspect is ledgered")
        .version;
    let older = with(rows, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.inputs = "sha256:the-corpus-before-an-edit".to_string();
        r.digest = "sha256:the-verdicts-before-an-edit".to_string();
        r.instances -= 1;
    });
    let ledger = render(&older);
    let replays = Replays::from([(
        (suspect::RULE.to_string(), SUSPECT_CORPUS.to_string()),
        Replayed {
            digest: "sha256:what-this-binary-reaches-over-the-recorded-corpus".to_string(),
            instances: rows
                .iter()
                .find(|row| row.rule == suspect::RULE && row.corpus == SUSPECT_CORPUS)
                .expect("relation.target.suspect has a row over its corpus")
                .instances
                - 1,
        },
    )]);

    for bless in [false, true] {
        let judgement = judge(&ledger, rows, &replays, bless);
        assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
        let message = &judgement.failures[0];
        assert!(message.contains(suspect::RULE), "{message}");
        assert!(message.contains(&format!("VERSION {version}")), "{message}");
        assert!(
            message.contains("engine/crates/check/src/suspect.rs"),
            "{message}"
        );
        assert!(message.contains(SUSPECT_CORPUS), "{message}");
        assert!(message.contains("raise VERSION"), "{message}");
        assert_eq!(
            judgement.ledger, ledger,
            "bless={bless} rewrote a row whose verdicts moved over the recorded corpus"
        );
    }
}

/// A raised `VERSION` with an unchanged digest asks for a re-record outside
/// bless, and bless re-records it. Raising a version conservatively is not a
/// defect.
#[test]
fn a_raised_version_asks_for_a_re_record_and_bless_takes_it() {
    let rows = computed();
    let older = with(rows, suspect::RULE, SUSPECT_CORPUS, |r| r.version -= 1);
    let ledger = render(&older);

    let judgement = judge(&ledger, rows, &Replays::new(), false);
    assert_eq!(judgement.failures.len(), 1, "{:?}", judgement.failures);
    assert!(judgement.failures[0].contains(suspect::RULE));
    assert!(judgement.failures[0].contains("re-record"));

    let judgement = judge(&ledger, rows, &Replays::new(), true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(rows));
}

/// A row whose rule reached nothing pins nothing, and a rule with no row is
/// recorded only under bless.
#[test]
fn a_row_that_pins_nothing_and_a_rule_with_no_row_fail_outside_bless() {
    let rows = computed();
    let mut ledger = render(&rows[1..]);
    ledger.push_str("no.such.rule\tcheck\t1\tsha256:0\tsha256:0\t1\n");

    let judgement = judge(&ledger, rows, &Replays::new(), false);
    assert_eq!(judgement.failures.len(), 2, "{:?}", judgement.failures);
    assert!(judgement
        .failures
        .iter()
        .any(|f| f.contains("no.such.rule") && f.contains("pins nothing")));
    assert!(judgement
        .failures
        .iter()
        .any(|f| f.contains(&rows[0].rule) && f.contains("no row")));

    let judgement = judge(&ledger, rows, &Replays::new(), true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(rows));
}

/// The standing test's replays come from the commit that last wrote the
/// ledger. A computed row whose corpus moved against the ledger of that
/// commit gets a replay, and bless re-records it, because nothing but the
/// corpus fingerprint moved.
#[test]
fn the_standing_test_replays_from_the_commit_that_last_wrote_the_ledger() {
    let commit = ledger_commit().expect("git names the commit that last wrote the ledger");
    let ledger = String::from_utf8(
        git(&["show", &format!("{commit}:./{LEDGER}")]).expect("git reads that ledger"),
    )
    .expect("the ledger is text");
    let (recorded, _) = parse(&ledger);
    let moved = with(&recorded, suspect::RULE, SUSPECT_CORPUS, |r| {
        r.inputs = "sha256:the-corpus-after-an-edit".to_string();
    });

    let answered = standing_replays(&ledger, &moved);
    assert_eq!(answered.len(), 1, "{answered:?}");
    let judgement = judge(&ledger, &moved, &answered, true);
    assert!(judgement.failures.is_empty(), "{:?}", judgement.failures);
    assert_eq!(judgement.ledger, render(&moved));
}
