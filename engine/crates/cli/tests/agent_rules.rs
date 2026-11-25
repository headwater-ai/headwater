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

/// The rule file the API design standard owns.
const RULE: &str = ".github/instructions/api-design.instructions.md";

/// Gives the standard at `path` a `governs` edge to `src/api/**`, written under
/// its existing `relations` block.
fn govern(path: &Path) {
    let text = read(path);
    let text = text.replacen("relations:\n", "relations:\n  governs:\n    - src/api/**\n", 1);
    std::fs::write(path, text).expect("the standard is rewritten");
}

/// The Beacon corpus, assembled as the entry's `fixtures/README.md` says, with
/// the agent rules projection declared and `governs: src/api/**` on the API
/// design standard. Resolved, and not yet generated.
fn beacon(root: &Root) -> PathBuf {
    let (artifact, version, digest) = publish(root);
    let consumer = root.path().join("consumer");
    copy_tree(&entry().join("corpus/docs"), &consumer.join("docs"));
    govern(&consumer.join("docs/standards/api-design.md"));
    write(&consumer.join("src/api/x.rs"), "fn main() {}\n");
    write(
        &consumer.join(".headwater/overlay.yml"),
        &format!(
            "add:\n  identifier_schemes.standard_id.namespace: SCR\n  identifier_schemes.functional_spec_id.namespace: SCR\n  identifier_schemes.technical_spec_id.namespace: SCR\n  identifier_schemes.decision_id.namespace: SCR\nadd_to:\n  projections:\n{PROJECTION}"
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
    for heading in ["## Scope", "## Requirements", "## Conformance", "# API design"] {
        assert!(
            !text.contains(heading),
            "the body points at the standard and copies none of it, and it holds `{heading}`\n{text}"
        );
    }
    assert!(text.len() < 600, "{} bytes\n{text}", text.len());

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
