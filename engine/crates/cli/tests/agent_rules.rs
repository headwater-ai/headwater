// SPDX-License-Identifier: Apache-2.0
//! The `agent_rules` projection over the Beacon corpus of the `standards-spec`
//! library entry (#2089, HW-DR-0115).
//!
//! # The defect this target exists for
//!
//! A rule file is read by an agent harness that never runs Headwater, so the
//! two things it must get right are the ones the harness reads by itself: the
//! `applyTo` glob, which decides which edits load the file, and the body, which
//! must point at the governing document and never copy it. The first case
//! asserts that the glob admits a file under the governed path with the
//! engine's own pattern matcher, not with a string compare, and that no heading
//! of the standard reaches the body.
//!
//! The second half of the first case is why the file is held by
//! `generate --check`: an edit of the standard's summary moves the pointer, and
//! the committed copy goes stale.
//!
//! Every case runs the built binary over the real maintained source, published
//! the way `taxonomy publish --from` publishes it, and over the Beacon corpus
//! assembled the way the entry's `fixtures/README.md` says.

use headwater_meta::pattern::Pattern;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn entry() -> PathBuf {
    repository().join("docs/taxonomies/standards-spec/fixtures")
}

#[derive(Debug)]
struct Ran {
    code: Option<i32>,
    out: String,
    err: String,
}

/// A scratch tree that removes itself. The label keeps two cases apart, because
/// cargo runs them as threads of one process and the pid alone names one
/// directory for both.
struct Root(PathBuf);

impl Root {
    fn scratch(label: &str) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-agent-rules-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the scratch tree is made");
        Root(at)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(root: &Path, arguments: &[&str]) -> Ran {
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .expect("the binary runs");
    Ran {
        code: output.status.code(),
        out: String::from_utf8_lossy(&output.stdout).into_owned(),
        err: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("the path has a parent"))
        .expect("the parent is made");
    std::fs::write(path, text).expect("the file is written");
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).expect("the directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the type reads").is_dir() {
            true => copy_tree(&entry.path(), &target),
            false => {
                std::fs::create_dir_all(to).expect("the directory is made");
                std::fs::copy(entry.path(), &target).expect("the file is copied");
            }
        }
    }
}

/// The real maintained source, published into `<root>/release`, and the
/// version and digest a consumer pins it by.
fn publish(root: &Root) -> (PathBuf, String, String) {
    let artifact = root.path().join("release");
    let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
        .arg("taxonomy")
        .arg("publish")
        .arg("--from")
        .arg(repository().join("taxonomy-source/headwater-standard"))
        .arg("--out")
        .arg(&artifact)
        .arg("--root")
        .arg(repository())
        .output()
        .expect("the binary runs");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let release = headwater_resolve::release::read(
        &std::fs::read_to_string(artifact.join("release.yml")).expect("the release reads"),
    )
    .expect("the published record reads");
    let manifest =
        std::fs::read_to_string(artifact.join("package.yml")).expect("the manifest reads");
    let version = manifest
        .lines()
        .find_map(|line| line.strip_prefix("version: "))
        .expect("the manifest declares a version")
        .trim()
        .to_string();
    (artifact, version, release.digest)
}

/// The projection every case declares: rule files for the `standards` shelf.
const PROJECTION: &str = "    - kind: agent_rules\n      for: [standards]\n      output: \".github/instructions/{name}.instructions.md\"\n      budget: {per_file: 600, aggregate: 8192}\n";

/// The clause a pointer adds when nobody accepted the governing document.
const UNACCEPTED: &str = "No person has accepted that document.";

/// The rule file the API design standard owns.
const RULE: &str = ".github/instructions/api-design.instructions.md";

/// Gives the document at `path` a `governs` edge to `src/api/**`, written under
/// its existing `relations` block.
fn govern(path: &Path) {
    let text = read(path);
    assert!(
        text.contains("relations:\n"),
        "{} has no relations block, so no governs edge would be written",
        path.display()
    );
    let text = text.replacen(
        "relations:\n",
        "relations:\n  governs:\n    - src/api/**\n",
        1,
    );
    std::fs::write(path, text).expect("the document is rewritten");
}

/// The Beacon corpus, assembled as the entry's `fixtures/README.md` says, with
/// the agent rules projection declared and `governs: src/api/**` on the API
/// design standard. Resolved, and not yet generated.
fn beacon(root: &Root) -> PathBuf {
    beacon_declaring(root, PROJECTION)
}

/// [`beacon`] with `projection` declared in place of [`PROJECTION`].
fn beacon_declaring(root: &Root, projection: &str) -> PathBuf {
    let (artifact, version, digest) = publish(root);
    let consumer = root.path().join("consumer");
    copy_tree(&entry().join("corpus/docs"), &consumer.join("docs"));
    govern(&consumer.join("docs/standards/api-design.md"));
    write(&consumer.join("src/api/x.rs"), "fn main() {}\n");
    write(
        &consumer.join(".headwater/overlay.yml"),
        &format!(
            "add:\n  identifier_schemes.standard_id.namespace: SCR\n  identifier_schemes.functional_spec_id.namespace: SCR\n  identifier_schemes.technical_spec_id.namespace: SCR\n  identifier_schemes.decision_id.namespace: SCR\nadd_to:\n  projections:\n{projection}"
        ),
    );
    write(
        &consumer.join(".headwater/taxonomy.yml"),
        &format!(
            "taxonomy:\n  package: headwater/standard\n  version: {version}\n  digest: {digest}\n  bundles: [standards-spec]\n  overlay: .headwater/overlay.yml\ncorpus:\n  root: docs\n"
        ),
    );
    let vendored = run(
        &consumer,
        &[
            "taxonomy",
            "vendor",
            artifact.to_str().expect("the artifact is UTF-8"),
        ],
    );
    assert_eq!(vendored.code, Some(0), "{vendored:?}");
    let resolved = run(&consumer, &["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
    consumer
}

/// The front matter of a rule file, which must open on line 1.
fn front_matter(text: &str) -> &str {
    let rest = text
        .strip_prefix("---\n")
        .unwrap_or_else(|| panic!("line 1 of the rule file is `---`\n{text}"));
    let end = rest
        .find("\n---\n")
        .unwrap_or_else(|| panic!("the front matter closes\n{text}"));
    &rest[..end]
}

#[test]
fn a_standard_that_governs_src_api_writes_one_rule_file_whose_apply_to_admits_a_file_under_it_and_check_catches_a_summary_edit(
) {
    let root = Root::scratch("one");
    let consumer = beacon(&root);

    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&consumer.join(RULE));

    let loaded = headwater_yaml::load(front_matter(&text))
        .unwrap_or_else(|errors| panic!("the front matter loads: {errors:?}\n{text}"))
        .value;
    let members = loaded.as_map().expect("the front matter is a mapping");
    let apply_to = members
        .get("applyTo")
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .unwrap_or_else(|| panic!("`applyTo` is a string\n{text}"));
    assert_eq!(apply_to, "src/api/**", "{text}");
    assert!(
        Pattern::new(&apply_to).matches("src/api/x.rs"),
        "`applyTo: {apply_to}` admits src/api/x.rs\n{text}"
    );
    assert!(
        !Pattern::new(&apply_to).matches("src/other/x.rs"),
        "`applyTo: {apply_to}` admits nothing outside src/api\n{text}"
    );
    assert!(
        members
            .get("headwater:generated")
            .and_then(|node| node.value.as_scalar())
            .is_some_and(|scalar| scalar.text.starts_with("agent_rules.")),
        "the marker is a front-matter member\n{text}"
    );

    assert!(
        text.contains(
            "The rules every Beacon component that exposes an HTTP interface answers to."
        ),
        "the body carries the summary\n{text}"
    );
    assert!(
        text.contains("docs/standards/api-design.md"),
        "the body names the source\n{text}"
    );
    assert!(
        text.contains("](../../docs/standards/api-design.md)"),
        "the link is relative to the rule file\n{text}"
    );
    for heading in [
        "## Scope",
        "## Requirements",
        "## Conformance",
        "# API design",
    ] {
        assert!(
            !text.contains(heading),
            "the body points at the standard and copies none of it, and it holds `{heading}`\n{text}"
        );
    }
    assert!(text.len() < 600, "{} bytes\n{text}", text.len());
    assert!(
        !text.contains(UNACCEPTED),
        "the standard is accepted, so the pointer warns of nothing\n{text}"
    );

    let clean = run(&consumer, &["generate", "--check"]);
    assert_eq!(clean.code, Some(0), "{clean:?}");

    let standard = consumer.join("docs/standards/api-design.md");
    let edited = read(&standard).replace(
        "summary: The rules every Beacon component",
        "summary: The rules each Beacon component",
    );
    std::fs::write(&standard, edited).expect("the summary is edited");
    let stale = run(&consumer, &["generate", "--check"]);
    assert_eq!(stale.code, Some(1), "{stale:?}");
    assert!(stale.out.contains(RULE), "{stale:?}");

    let regenerated = run(&consumer, &["generate"]);
    assert_eq!(regenerated.code, Some(0), "{regenerated:?}");
    assert!(read(&consumer.join(RULE)).contains("The rules each Beacon component"));
    let current = run(&consumer, &["generate", "--check"]);
    assert_eq!(current.code, Some(0), "{current:?}");

    // The gate in its other two directions: a rule file that is gone, and one
    // edited by hand, are each stale, and each names the file.
    std::fs::remove_file(consumer.join(RULE)).expect("the rule file is removed");
    let missing = run(&consumer, &["generate", "--check"]);
    assert_eq!(missing.code, Some(1), "{missing:?}");
    assert!(missing.out.contains(RULE), "{missing:?}");
    let restored = run(&consumer, &["generate"]);
    assert_eq!(restored.code, Some(0), "{restored:?}");
    let rule = consumer.join(RULE);
    std::fs::write(
        &rule,
        read(&rule).replace("Read that document", "Skim that document"),
    )
    .expect("the rule file is edited by hand");
    let tampered = run(&consumer, &["generate", "--check"]);
    assert_eq!(tampered.code, Some(1), "{tampered:?}");
    assert!(tampered.out.contains(RULE), "{tampered:?}");
}

#[test]
fn a_standard_with_no_governs_edge_gets_no_rule_file() {
    let root = Root::scratch("ungoverned");
    let consumer = beacon(&root);
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    assert!(consumer.join(RULE).is_file(), "{generated:?}");
    let ungoverned = consumer.join(".github/instructions/logging-baseline.instructions.md");
    assert!(
        !ungoverned.exists(),
        "logging-baseline.md governs nothing, so no edit should load a rule for it"
    );
}

#[test]
fn two_sources_with_one_file_stem_write_neither_rule_file_and_say_why() {
    let root = Root::scratch("collision");
    let consumer = beacon(&root);
    let second = consumer.join("docs/standards/v2/api-design.md");
    write(
        &second,
        &read(&consumer.join("docs/standards/api-design.md"))
            .replace("id: SCR-STD-api-design", "id: SCR-STD-api-design-v2"),
    );

    let generated = run(&consumer, &["generate"]);
    assert!(
        !consumer.join(RULE).exists(),
        "two standards named api-design.md own one rule path, and neither may overwrite the other\n{generated:?}"
    );
    let printed = format!("{}{}", generated.out, generated.err);
    for named in [
        "docs/standards/api-design.md",
        "docs/standards/v2/api-design.md",
        "HW-DR-0115",
        RULE,
    ] {
        assert!(
            printed.contains(named),
            "the refusal names `{named}`\n{generated:?}"
        );
    }
}

#[test]
fn a_document_on_a_shelf_the_for_does_not_name_gets_no_rule_file() {
    let root = Root::scratch("other-shelf");
    let consumer = beacon(&root);
    govern(&consumer.join("docs/component-specs/signing/functional.md"));
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    assert!(consumer.join(RULE).is_file(), "{generated:?}");
    assert!(
        !consumer
            .join(".github/instructions/functional.instructions.md")
            .exists(),
        "the functional spec governs src/api, and the declaration's `for` names only `standards`"
    );
}

/// A pattern that opens with `@` is a YAML reserved indicator when it is not
/// quoted, and one that opens with `*` is an alias, so a
/// harness that parses the front matter would read no `applyTo` at all. The
/// third `governs` target is one anchor over two patterns that the first two
/// already declare, so each pattern appears once, at its first declaration.
#[test]
fn an_apply_to_that_opens_with_a_yaml_indicator_still_loads_and_holds_every_pattern_in_declared_order(
) {
    let root = Root::scratch("star");
    let consumer = beacon(&root);
    let standard = consumer.join("docs/standards/api-design.md");
    std::fs::write(
        &standard,
        read(&standard).replacen(
            "  governs:\n    - src/api/**\n",
            "  governs:\n    - \"@generated/**\"\n    - src/api/**\n    - [\"src/api/**\", \"@generated/**\"]\n",
            1,
        ),
    )
    .expect("the standard is rewritten");
    write(&consumer.join("@generated/y.rs"), "fn y() {}\n");

    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&consumer.join(RULE));
    let loaded = headwater_yaml::load(front_matter(&text))
        .unwrap_or_else(|errors| panic!("the front matter loads: {errors:?}\n{text}"))
        .value;
    let apply_to = loaded
        .as_map()
        .and_then(|members| members.get("applyTo"))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .unwrap_or_else(|| panic!("`applyTo` is a string\n{text}"));
    assert_eq!(apply_to, "@generated/**,src/api/**", "{text}");
    let patterns: Vec<Pattern> = apply_to.split(',').map(Pattern::new).collect();
    for admitted in ["@generated/y.rs", "src/api/x.rs"] {
        assert!(
            patterns.iter().any(|pattern| pattern.matches(admitted)),
            "`applyTo: {apply_to}` admits {admitted}\n{text}"
        );
    }
}

#[test]
fn an_output_with_no_name_placeholder_is_refused_and_writes_nothing() {
    let root = Root::scratch("no-name");
    let consumer = beacon_declaring(
        &root,
        "    - kind: agent_rules\n      for: [standards]\n      output: \".github/instructions/all.instructions.md\"\n      budget: {per_file: 600, aggregate: 8192}\n",
    );
    let generated = run(&consumer, &["generate"]);
    assert!(
        !consumer
            .join(".github/instructions/all.instructions.md")
            .exists(),
        "every source would own one path\n{generated:?}"
    );
    assert!(
        generated.out.contains("names no `{name}`"),
        "the refusal says why\n{generated:?}"
    );
}

/// Spec 5: a pointer to a document that nobody accepted says so beside the
/// summary, because an agent that follows it must know before it reads.
#[test]
fn a_rule_file_for_an_unaccepted_source_says_that_nobody_accepted_it() {
    let root = Root::scratch("asserted");
    let consumer = beacon(&root);
    let standard = consumer.join("docs/standards/api-design.md");
    let text = read(&standard);
    assert!(text.contains("  warrant: accepted\n"), "{text}");
    std::fs::write(
        &standard,
        text.replacen("  warrant: accepted\n", "  warrant: asserted\n", 1)
            .replacen("  accepted_by: fixture-acceptor\n", "", 1),
    )
    .expect("the standard is rewritten");
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let rule = read(&consumer.join(RULE));
    assert!(
        rule.contains(UNACCEPTED),
        "the source's warrant is `asserted`, and the pointer does not say so\n{rule}"
    );
}

/// A pattern may hold a `"`, because a path may. Inside the quoted `applyTo`
/// it is escaped, or the front matter does not load. The `source-tree`
/// resolver binds no pattern with a `\`, so the unit case in `agent_rules.rs`
/// holds that escape.
#[test]
fn an_apply_to_whose_pattern_holds_a_quote_loads_to_the_declared_pattern() {
    let root = Root::scratch("escape");
    let consumer = beacon(&root);
    let standard = consumer.join("docs/standards/api-design.md");
    std::fs::write(
        &standard,
        read(&standard).replacen(
            "  governs:\n    - src/api/**\n",
            "  governs:\n    - 'src/q\"uote/**'\n",
            1,
        ),
    )
    .expect("the standard is rewritten");
    write(&consumer.join("src/q\"uote/a.rs"), "fn a() {}\n");

    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let text = read(&consumer.join(RULE));
    let loaded = headwater_yaml::load(front_matter(&text))
        .unwrap_or_else(|errors| panic!("the front matter loads: {errors:?}\n{text}"))
        .value;
    let apply_to = loaded
        .as_map()
        .and_then(|members| members.get("applyTo"))
        .and_then(|node| node.value.as_scalar())
        .map(|scalar| scalar.text.clone())
        .unwrap_or_else(|| panic!("`applyTo` is a string\n{text}"));
    assert_eq!(apply_to, "src/q\"uote/**", "{text}");
}

/// The check rule that reports a rule file past its budget (#2090).
const BUDGET_RULE: &str = "projection.budget.exceeded";

/// The three rules of the prune cases, in derived reading precedence. The
/// standard `regulates` the delivery functional spec, which is the governance
/// family, so it precedes it. The technical spec `realizes` the functional
/// spec, `nucleus: to`, so the functional spec is its nucleus and precedes it.
const STANDARD: &str = ".github/instructions/api-design.instructions.md";
const NUCLEUS: &str = ".github/instructions/functional.instructions.md";
const SATELLITE: &str = ".github/instructions/technical.instructions.md";

/// An `agent_rules` over every shelf with the two budget figures given.
fn every_shelf(per_file: u64, aggregate: u64) -> String {
    format!(
        "    - kind: agent_rules\n      output: \".github/instructions/{{name}}.instructions.md\"\n      budget: {{per_file: {per_file}, aggregate: {aggregate}}}\n"
    )
}

/// The Beacon corpus with `governs: src/api/**` on the standard, on the
/// delivery functional spec and on the delivery technical spec, and the
/// projection declared over every shelf with a budget that admits all three.
/// Generated once, and the byte size of each of the three rule files, in the
/// order [`STANDARD`], [`NUCLEUS`], [`SATELLITE`].
fn three_rules(root: &Root) -> (PathBuf, [u64; 3]) {
    let consumer = beacon_declaring(root, &every_shelf(1_000_000, 1_000_000));
    govern(&consumer.join("docs/component-specs/delivery/functional.md"));
    govern(&consumer.join("docs/component-specs/delivery/technical.md"));
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let size = |path: &str| read(&consumer.join(path)).len() as u64;
    let sizes = [size(STANDARD), size(NUCLEUS), size(SATELLITE)];
    (consumer, sizes)
}

/// Declares the budget again and resolves, so that the next run reads it.
fn rebudget(consumer: &Path, per_file: u64, aggregate: u64) {
    write(
        &consumer.join(".headwater/overlay.yml"),
        &read(&consumer.join(".headwater/overlay.yml")).replace(
            &every_shelf(1_000_000, 1_000_000),
            &every_shelf(per_file, aggregate),
        ),
    );
    let resolved = run(consumer, &["taxonomy", "resolve"]);
    assert_eq!(resolved.code, Some(0), "{resolved:?}");
}

/// The lines of a `check` run that the budget rule printed.
fn budget_findings(ran: &Ran) -> Vec<&str> {
    ran.out
        .lines()
        .filter(|line| line.contains(BUDGET_RULE))
        .collect()
}

/// HW-DR-0115: the prune drops from the satellite end of the derived reading
/// precedence. Two of three rules fit the aggregate. A prune by path order
/// would drop the standard, which sorts last, and a prune by size would drop
/// the largest. Only the precedence order drops the satellite and keeps its
/// nucleus.
#[test]
fn an_aggregate_two_rules_fit_prunes_the_satellite_and_check_reports_the_projection_once() {
    let root = Root::scratch("aggregate");
    let (consumer, [standard, nucleus, satellite]) = three_rules(&root);
    let aggregate = standard + nucleus;
    let total = standard + nucleus + satellite;
    rebudget(&consumer, 1_000_000, aggregate);

    let checked = run(&consumer, &["check"]);
    let found = budget_findings(&checked);
    assert_eq!(found.len(), 1, "exactly one aggregate finding\n{checked:?}");
    for named in [
        "projections.0".to_string(),
        "agent_rules".to_string(),
        format!("{total} bytes"),
        format!("{aggregate}"),
        "withholds 1".to_string(),
    ] {
        assert!(
            checked.out.contains(&named),
            "the aggregate finding names `{named}`\n{checked:?}"
        );
    }

    // The satellite's rule that the larger budget wrote is still on disk, and
    // it carries the marker, so it would keep loading: the gate names it.
    let stale = run(&consumer, &["generate", "--check"]);
    assert_eq!(stale.code, Some(1), "{stale:?}");
    let verdict = stale
        .out
        .lines()
        .skip_while(|line| !line.ends_with(SATELLITE))
        .nth(1)
        .unwrap_or_else(|| panic!("the gate names {SATELLITE}\n{stale:?}"));
    assert!(
        verdict.contains("the budget withholds it"),
        "the gate holds the pruned rule as stale, and says `{verdict}`\n{stale:?}"
    );
    let pruned = run(&consumer, &["generate"]);
    assert_eq!(pruned.code, Some(0), "{pruned:?}");
    assert!(!consumer.join(SATELLITE).exists(), "{pruned:?}");
    assert!(consumer.join(STANDARD).is_file(), "{pruned:?}");
    assert!(consumer.join(NUCLEUS).is_file(), "{pruned:?}");
    let clean = run(&consumer, &["generate", "--check"]);
    assert_eq!(clean.code, Some(0), "{clean:?}");

    // From an empty directory, the same two files and not the third.
    std::fs::remove_dir_all(consumer.join(".github/instructions"))
        .expect("the rules are removed");
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    assert!(consumer.join(STANDARD).is_file(), "{generated:?}");
    assert!(consumer.join(NUCLEUS).is_file(), "{generated:?}");
    assert!(
        !consumer.join(SATELLITE).exists(),
        "the satellite's rule is pruned before its nucleus's\n{generated:?}"
    );
    assert!(
        generated.out.contains(SATELLITE) && generated.out.contains(&format!("{aggregate}")),
        "the plan names the pruned rule and the aggregate\n{generated:?}"
    );
    let current = run(&consumer, &["generate", "--check"]);
    assert_eq!(current.code, Some(0), "{current:?}");
}

/// Done-when 1: a rule file past `per_file` is withheld, and the finding names
/// the file, its size, the figure and the two layers that can trim it.
#[test]
fn a_rule_past_per_file_is_not_written_and_check_names_its_size_and_the_layer_to_trim() {
    let root = Root::scratch("per-file");
    let (consumer, sizes) = three_rules(&root);
    let [standard, nucleus, satellite] = sizes;
    let largest = sizes.into_iter().max().expect("three sizes");
    assert_eq!(
        sizes.iter().filter(|size| **size == largest).count(),
        1,
        "one rule is the largest, so one alone is past the figure: {sizes:?}"
    );
    let (over, source) = match largest {
        size if size == standard => (STANDARD, "docs/standards/api-design.md"),
        size if size == nucleus => (NUCLEUS, "docs/component-specs/delivery/functional.md"),
        size if size == satellite => (SATELLITE, "docs/component-specs/delivery/technical.md"),
        _ => unreachable!(),
    };
    let per_file = largest - 1;
    rebudget(&consumer, per_file, 1_000_000);

    let checked = run(&consumer, &["check"]);
    let found = budget_findings(&checked);
    assert_eq!(found.len(), 1, "exactly one per-file finding\n{checked:?}");
    for named in [
        over.to_string(),
        format!("{largest} bytes"),
        format!("{per_file}"),
        format!("the `summary` of `{source}`"),
        "projections.0.budget.per_file".to_string(),
    ] {
        assert!(
            checked.out.contains(&named),
            "the per-file finding names `{named}`\n{checked:?}"
        );
    }

    std::fs::remove_dir_all(consumer.join(".github/instructions"))
        .expect("the rules are removed");
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    assert!(!consumer.join(over).exists(), "{generated:?}");
    for kept in [STANDARD, NUCLEUS, SATELLITE]
        .into_iter()
        .filter(|path| *path != over)
    {
        assert!(consumer.join(kept).is_file(), "{kept}\n{generated:?}");
    }
    let current = run(&consumer, &["generate", "--check"]);
    assert_eq!(current.code, Some(0), "{current:?}");
}

/// The gate in its third direction: a budget every rule fits exactly gives no
/// finding and withholds nothing.
#[test]
fn a_budget_every_rule_fits_exactly_reports_nothing_and_writes_all_three() {
    let root = Root::scratch("fits");
    let (consumer, [standard, nucleus, satellite]) = three_rules(&root);
    let largest = standard.max(nucleus).max(satellite);
    rebudget(&consumer, largest, standard + nucleus + satellite);
    let checked = run(&consumer, &["check"]);
    assert!(budget_findings(&checked).is_empty(), "{checked:?}");
    let generated = run(&consumer, &["generate"]);
    assert_eq!(generated.code, Some(0), "{generated:?}");
    let current = run(&consumer, &["generate", "--check"]);
    assert_eq!(current.code, Some(0), "{current:?}");
    for kept in [STANDARD, NUCLEUS, SATELLITE] {
        assert!(consumer.join(kept).is_file(), "{kept}");
    }
}
