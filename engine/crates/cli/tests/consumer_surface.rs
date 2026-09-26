// SPDX-License-Identifier: Apache-2.0
//! `surface.local_path.instructed` and `surface.command.undeclared`, held
//! against the manifest they read.
//!
//! # What this target is evidence of
//!
//! [HW-DR-0077](../../../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
//! rules that no page for an adopter instructs a file only this repository
//! holds, and that such a page may cite one in a passage that says so.
//! [#933](https://github.com/headwater-ai/headwater/issues/933) refused a rule
//! that read a hard-coded list, because it reports zero by construction. So the
//! decisive case is three documents that hold the same two paths and differ in
//! one thing each:
//!
//! - `x.md` is on the manifest's adopter list and instructs both paths. Both
//!   are reported.
//! - `y.md` is on the list and holds the same paths under a directive that
//!   marks the passage as how this repository does it. Nothing is reported.
//! - `z.md` holds the same paths and is **not** on the list. Nothing is
//!   reported, and that is the case that proves the rule reads the manifest.
//!
//! Every case runs against this repository's own resolved taxonomy, with the
//! adopter list narrowed by an `override` so that the three documents share a
//! shelf, a kind and every other declaration.

use std::path::{Path, PathBuf};
use std::process::Command;

const RULE: &str = "surface.local_path.instructed";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the directory is there");
    for entry in std::fs::read_dir(from).expect("the fixture directory reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the file type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the fixture copies");
            }
        }
    }
}

struct Root {
    at: PathBuf,
    /// This repository's overlay, before a case adds its `override` block.
    overlay: String,
}

impl Root {
    /// A scratch corpus with this repository's taxonomy, and an adopter list
    /// of exactly the documents `adopter` names.
    fn new(label: &str, adopter: &[&str]) -> Root {
        let at = std::env::temp_dir().join(format!(
            "headwater-cli-surface-{}-{label}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).expect("the root is made");
        let repository = repository();
        copy(
            &repository.join(headwater_resolve::package::PACKAGES),
            &at.join(headwater_resolve::package::PACKAGES),
        );
        copy(
            &repository.join("docs/taxonomies"),
            &at.join("docs/taxonomies"),
        );
        std::fs::copy(
            repository.join(".headwater/taxonomy.yml"),
            at.join(".headwater/taxonomy.yml"),
        )
        .expect("the declaration copies");
        let overlay = std::fs::read_to_string(repository.join(".headwater/overlay.yml"))
            .expect("the overlay reads");
        assert!(
            !overlay.lines().any(|line| line == "override:"),
            "the overlay grew an `override` block, so this case must merge into it"
        );
        let root = Root { at, overlay };
        root.override_with(adopter, "");
        root
    }

    /// Write the overlay again with this `override` block, and resolve.
    fn override_with(&self, adopter: &[&str], more: &str) {
        let mut overlay = self.overlay.clone();
        overlay.push_str(&format!(
            "\noverride:\n  surface.adopter_documents: [{}]\n{more}",
            adopter.join(", ")
        ));
        std::fs::write(self.at.join(".headwater/overlay.yml"), overlay)
            .expect("the overlay writes");
        let resolved = self.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.0,
            Some(0),
            "the fixture resolves\n{}{}",
            resolved.1,
            resolved.2
        );
    }

    fn contract(&self, slug: &str, body: &str) {
        let path = self.at.join(format!("docs/interfaces/{slug}.md"));
        std::fs::create_dir_all(path.parent().expect("it has a parent"))
            .expect("the shelf directory is there");
        let mut text = format!(
            "---\n\
             id: HW-IFACE-{slug}\n\
             status: draft\n\
             status_since: 2026-08-16\n\
             summary: \"What this verb takes, what it prints, and what its exit status means.\"\n\
             last_verified: 2026-08-16\n\
             title: \"{slug}\"\n\
             governs:\n  - .githooks/pre-commit\n\
             ---\n\n\
             # {slug}\n"
        );
        for section in [
            "Synopsis",
            "Description",
            "Preconditions",
            "Options",
            "Exit status",
            "Environment",
            "Files",
            "See also",
        ] {
            text.push_str(&format!("\n## {section}\n\nNone.\n"));
        }
        text.push_str(body);
        std::fs::write(&path, text).expect("the contract writes");
    }

    fn run(&self, arguments: &[&str]) -> (Option<i32>, String, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(arguments)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).into_owned(),
            String::from_utf8_lossy(&output.stderr).into_owned(),
        )
    }

    /// The message of each finding of this rule on one document that no escape
    /// hides, read from the JSON report, which prints one member per line.
    fn findings(&self, slug: &str) -> Vec<String> {
        self.findings_of(RULE, slug)
    }

    /// The same, for any rule of the surface.
    fn findings_of(&self, rule: &str, slug: &str) -> Vec<String> {
        self.located(rule, slug)
            .into_iter()
            .map(|(_, message)| message)
            .collect()
    }

    /// The line of the page each finding of this rule stands on, beside the
    /// finding's message, so a test can tell two findings with one message
    /// apart by where they are (#1085).
    fn findings_at(&self, slug: &str) -> Vec<(usize, String)> {
        self.located(RULE, slug)
    }

    /// The 1-based line of the page's source that holds `text`.
    fn line_of(&self, slug: &str, text: &str) -> usize {
        let page = std::fs::read_to_string(self.at.join(format!("docs/interfaces/{slug}.md")))
            .expect("the contract reads");
        page.lines()
            .position(|line| line == text)
            .map(|at| at + 1)
            .expect("the page holds the line")
    }

    fn located(&self, rule: &str, slug: &str) -> Vec<(usize, String)> {
        let (_, out, _) = self.run(&["check", "--format", "json"]);
        let path = format!("\"path\": \"docs/interfaces/{slug}.md\",");
        let lines: Vec<&str> = out.lines().map(str::trim).collect();
        let mut messages = Vec::new();
        for (at, line) in lines.iter().enumerate() {
            if *line != format!("\"rule\": \"{rule}\",") {
                continue;
            }
            // One finding runs to the brace that closes it, and the last one
            // would otherwise run on into the read set, which names every
            // page. A finding the runner filtered is still in the report, with
            // its escape named, and only an unescaped one is a verdict a
            // reader meets.
            let end = lines[at + 1..]
                .iter()
                .position(|l| l.starts_with("\"rule\"") || l.starts_with('}'))
                .map_or(lines.len(), |next| at + 1 + next);
            let finding = &lines[at..end];
            if finding.contains(&path.as_str())
                && finding
                    .iter()
                    .any(|l| l.starts_with("\"escape\": \"none\""))
            {
                if let Some(message) = finding.iter().find(|l| l.starts_with("\"message\"")) {
                    let line = finding
                        .iter()
                        .find_map(|l| l.strip_prefix("\"line\": "))
                        .and_then(|n| n.trim_end_matches(',').parse().ok())
                        .unwrap_or(0);
                    messages.push((line, message.to_string()));
                }
            }
        }
        messages
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

const INSTRUCTS: &str = "\nRun the gate before you commit.\n\n```sh\nsh tools/foo.sh\n```\n";

/// The two forms the first edition missed: a path behind a shell variable, and
/// a code span inside a block quote.
const DISGUISED: &str = "\n```sh\nsh \"$ROOT/tools/var.sh\"\n```\n\n> Then run `tools/quote.sh`.\n";

const SAYS_SO: &str = "\n<!-- headwater allow=surface.local_path.instructed scope=block until=2099-12-31 reason=accepted_deviation note=how this repository does it -->\nThis repository runs `tools/foo.sh` before it commits.\n\n<!-- headwater allow=surface.local_path.instructed scope=file until=2099-12-31 reason=accepted_deviation note=the governs edge names how this repository does it -->\n";

#[test]
fn the_rule_reads_the_manifest_and_honors_a_passage_that_says_so() {
    let root = Root::new(
        "three",
        &[
            "docs/interfaces/x.md",
            "docs/interfaces/y.md",
            "docs/interfaces/w.md",
        ],
    );
    root.contract("x", INSTRUCTS);
    root.contract("y", SAYS_SO);
    root.contract("z", INSTRUCTS);
    root.contract("w", DISGUISED);

    let (_, out, err) = root.run(&["check"]);
    let x = root.findings("x");
    assert_eq!(
        x.len(),
        2,
        "the adopter page names `.githooks/pre-commit` in `governs` and `tools/foo.sh` in a \
         code block, and each is one finding\n{x:#?}\n{out}{err}"
    );
    assert!(x.iter().any(|f| f.contains("tools/foo.sh")), "{x:#?}");
    assert!(
        x.iter().any(|f| f.contains(".githooks/pre-commit")),
        "{x:#?}"
    );
    assert_eq!(
        root.findings("y"),
        Vec::<String>::new(),
        "a passage marked as how this repository does it passes"
    );
    let w = root.findings("w");
    assert!(
        w.iter().any(|f| f.contains("$ROOT/tools/var.sh"))
            && w.iter().any(|f| f.contains("tools/quote.sh")),
        "a path behind a shell variable and a code span in a block quote are both reported\n{w:#?}"
    );
    // The generation step reads the adopter list, so a page the list does not
    // name has no instance of the rule at all, rather than an instance that
    // passes without being read (#1051). Three of the four typed documents are
    // on the list.
    assert!(
        out.lines()
            .any(|line| line.trim() == format!("3 instances of {RULE}")),
        "the rule instantiates on the three listed pages and not on `z.md`\n{out}"
    );
    assert_eq!(
        root.findings("z"),
        Vec::<String>::new(),
        "a page the manifest does not list is not an adopter's page, and the rule reads the \
         manifest rather than a list of its own"
    );
}

/// A local root named bare, with no trailing `/`, in code; and the same word
/// in prose (#976).
const BARE: &str = "\nWire the hooks once.\n\n```sh\ngit config core.hooksPath .githooks\nmkdocs build\n```\n\nThe site word in prose names nothing.\n";

/// A bare local root in code is reported, and the same word in prose is not.
///
/// Edition three matched a token that opens with a declared root, `.githooks/`
/// with its slash, so the bare `.githooks` that `git config core.hooksPath`
/// sets passed. The prose sentence carries `site`, which is also a declared
/// root, and the rule reads no prose. `mkdocs` is a declared root and also the
/// program the `site_generator` integration point depends on, so its bare
/// name in a shell block is that program and not the directory.
#[test]
fn a_bare_local_root_in_code_is_reported_and_a_word_of_prose_is_not() {
    let root = Root::new("bare", &["docs/interfaces/b.md"]);
    root.contract("b", BARE);
    let (_, out, err) = root.run(&["check"]);
    let b = root.findings("b");
    assert!(
        b.iter().any(|f| f.contains("names `.githooks`,")),
        "the bare `.githooks` of a `git config` line is reported\n{b:#?}\n{out}{err}"
    );
    assert!(
        !b.iter().any(|f| f.contains("names `site`")),
        "a word of prose is not reported\n{b:#?}"
    );
    assert!(
        !b.iter().any(|f| f.contains("names `mkdocs`")),
        "a bare name that the surface declares as a program is that program\n{b:#?}"
    );
}

/// A program with a path under its own root, a bare root in a shell block,
/// the same bare root as a value in a `yaml` block, and the same bare root in
/// a fence with no info string, which is as often output as input (#1085).
const BARE_BY_PLACE: &str = "\nBuild the site.\n\n```sh\nmkdocs build\nsh mkdocs/build.sh\nrm -rf site\n```\n\nSet the output directory.\n\n```yaml\nsite_dir: site\ndocs_dir: site/docs\n```\n\nThe output of a run.\n\n```\nrm -r site\n```\n";

/// A bare root counts only where a shell runs it, and a prefixed token counts
/// everywhere (#1085).
///
/// Edition four read a bare token in every code block, so `site_dir: site` in
/// an adopter's own `mkdocs.yml` excerpt was reported as a path of this
/// repository. `mkdocs/build.sh` opens with the root `mkdocs/` and its slash,
/// so the program exclusion, which applies to a whole bare token, does not
/// reach it.
#[test]
fn a_bare_root_counts_only_in_a_shell_block_and_a_prefixed_one_counts_everywhere() {
    let root = Root::new("byplace", &["docs/interfaces/p.md"]);
    root.contract("p", BARE_BY_PLACE);
    let (_, out, err) = root.run(&["check"]);
    let p = root.findings("p");
    assert!(
        !p.iter().any(|f| f.contains("names `mkdocs`,")),
        "a bare name that the surface declares as a program is that program\n{p:#?}\n{out}{err}"
    );
    assert!(
        p.iter().any(|f| f.contains("names `mkdocs/build.sh`,")),
        "a token under the root of a program's name is still a path of this repository\n{p:#?}"
    );
    let site: Vec<usize> = root
        .findings_at("p")
        .into_iter()
        .filter(|(_, f)| f.contains("names `site`,"))
        .map(|(line, _)| line)
        .collect();
    assert_eq!(
        site,
        vec![root.line_of("p", "rm -rf site")],
        "the bare `site` of `rm -rf site` in the shell block is reported, and the `site` of \
         `site_dir: site` in the `yaml` block, at line {}, and of `rm -r site` in the fence \
         with no info string, at line {}, are not\n{p:#?}",
        root.line_of("p", "site_dir: site"),
        root.line_of("p", "rm -r site")
    );
    assert!(
        p.iter().any(|f| f.contains("names `site/docs`,")),
        "a token that opens with a root and its slash counts in a `yaml` block\n{p:#?}"
    );
}

/// A bare root in a front-matter value is the author's own word, and a path
/// under a root there still counts (#1085).
///
/// The page is named `site`, so its `title` facet is the bare word `site`, one
/// scalar with no whitespace, which is what the front-matter loop reads. Its
/// `governs` value `.githooks/pre-commit` opens with a root and its slash.
#[test]
fn a_bare_root_in_a_front_matter_value_is_not_reported_and_a_path_is() {
    let root = Root::new("frontmatter", &["docs/interfaces/site.md"]);
    root.contract("site", "\nNothing to run.\n");
    let (_, out, err) = root.run(&["check"]);
    let found = root.findings("site");
    assert!(
        found
            .iter()
            .any(|f| f.contains("the `governs` front-matter value names `.githooks/pre-commit`,")),
        "a path under a root in a front-matter value is reported\n{found:#?}\n{out}{err}"
    );
    assert!(
        !found.iter().any(|f| f.contains("names `site`,")),
        "the bare `site` of the `title` facet is not reported\n{found:#?}"
    );
}

/// The page `headwater generate` writes from the `surface` block (#1051).
const PAGE: &str = "docs/interfaces/consumer-surface.md";

#[test]
fn the_surface_block_generates_a_page_that_goes_stale_with_it() {
    let root = Root::new("page", &["docs/interfaces/x.md"]);
    root.contract("x", "");
    let generated = root.run(&["generate"]);
    let page = std::fs::read_to_string(root.at.join(PAGE)).unwrap_or_else(|_| {
        panic!(
            "`generate` writes the consumer surface page\n{}{}",
            generated.1, generated.2
        )
    });
    assert!(
        page.starts_with("<!-- headwater:generated consumer_surface"),
        "the page carries the marker on its first line\n{page}"
    );
    for member in [
        "`LICENSE`",
        "`site_generator`",
        "`mkdocs`",
        "`curl`",
        "`headwater probe`",
        "`tar`",
    ] {
        assert!(page.contains(member), "the page names {member}\n{page}");
    }
    let clean = root.run(&["generate", "--check"]);
    assert_eq!(
        clean.0,
        Some(0),
        "a tree that was just generated is not stale\n{}{}",
        clean.1,
        clean.2
    );

    // (a) A hand edit to the page.
    std::fs::write(
        root.at.join(PAGE),
        format!("{page}\nA line somebody typed.\n"),
    )
    .expect("the page writes");
    let edited = root.run(&["generate", "--check"]);
    assert_ne!(
        edited.0,
        Some(0),
        "a hand edit to the generated page is stale\n{}{}",
        edited.1,
        edited.2
    );
    std::fs::write(root.at.join(PAGE), &page).expect("the page writes back");

    // (b) An edit to the `surface` block with no regeneration.
    root.override_with(
        &["docs/interfaces/x.md"],
        "  surface.prerequisites: [git, sh, curl, tar, unzip]\n",
    );
    let moved = root.run(&["generate", "--check"]);
    assert_ne!(
        moved.0,
        Some(0),
        "a `surface` block that moved leaves the page stale\n{}{}",
        moved.1,
        moved.2
    );
    root.run(&["generate"]);
    let again = std::fs::read_to_string(root.at.join(PAGE)).expect("the page reads");
    assert!(
        again.contains("`unzip`"),
        "the page follows the block\n{again}"
    );
}

/// The command rule (#1051). A page on the list holds four blocks, and only
/// the first one is a command the surface does not declare:
///
/// - (a) an `sh` block that runs `cargo`, which `commands` does not list,
/// - (b) a `yaml` block whose first word is `name:`,
/// - (c) an untagged block of output whose first word is `wrote`,
/// - (d) an `sh` block that runs `headwater` behind a `$ ` prompt.
///
/// A rule that reads untagged blocks reports (c), a rule that ignores the info
/// string reports (b), and a rule that does not strip the prompt reports (d).
const COMMANDS: &str = "\n```sh\ncargo install headwater\n```\n\n```yaml\nname: acme\n```\n\n```\nwrote docs/x.md\n```\n\n```sh\n$ headwater check\n```\n";

#[test]
fn the_command_rule_reads_only_shell_blocks_on_a_listed_page() {
    const COMMAND: &str = "surface.command.undeclared";
    let root = Root::new("commands", &["docs/interfaces/c.md"]);
    root.override_with(
        &["docs/interfaces/c.md"],
        "  surface.commands: [headwater]\n",
    );
    root.contract("c", COMMANDS);
    root.contract("d", COMMANDS);

    let (_, out, err) = root.run(&["check"]);
    let c = root.findings_of(COMMAND, "c");
    assert_eq!(
        c.len(),
        1,
        "only the `sh` block that runs `cargo` is reported\n{c:#?}\n{out}{err}"
    );
    assert!(c[0].contains("`cargo`"), "{c:#?}");
    assert!(
        out.lines()
            .any(|line| line.trim() == format!("1 instances of {COMMAND}")),
        "the rule instantiates on the listed page and not on `d.md`\n{out}"
    );
    assert_eq!(
        root.findings_of(COMMAND, "d"),
        Vec::<String>::new(),
        "a page the manifest does not list has no instance of the rule"
    );
}

/// The two readings the first case does not reach. A `console` block is a
/// shell block, and in it a line with no `$ ` prompt is output. A program
/// named by its path is compared by the last segment of the path, so
/// `/usr/bin/cargo` is `cargo` and not a name `commands` could never hold.
const CONSOLE: &str = "\n```console\n$ /usr/bin/cargo install headwater\nwrote docs/x.md\n```\n";

#[test]
fn a_console_block_reads_its_prompted_lines_by_the_last_segment_of_the_path() {
    const COMMAND: &str = "surface.command.undeclared";
    let root = Root::new("console", &["docs/interfaces/e.md"]);
    root.override_with(
        &["docs/interfaces/e.md"],
        "  surface.commands: [headwater]\n",
    );
    root.contract("e", CONSOLE);

    let (_, out, err) = root.run(&["check"]);
    let e = root.findings_of(COMMAND, "e");
    assert_eq!(
        e.len(),
        1,
        "the prompted line is one command and the output line is none\n{e:#?}\n{out}{err}"
    );
    assert!(
        e[0].contains("runs `cargo`"),
        "the program is named by the last segment of its path\n{e:#?}"
    );
}
