// SPDX-License-Identifier: Apache-2.0
//! `headwater site`: a built site held against the corpus it was built from.
//!
//! # The defect this target exists for
//!
//! [#978](https://github.com/headwater-ai/headwater/issues/978). Nothing in
//! the binary read a built site, so a page the navigation names and the build
//! never wrote, a page left from a document the corpus no longer holds, and a
//! link to a page that is not there all reached a reader.
//!
//! # The table
//!
//! One scratch corpus: the standard package, a `site_nav` projection, and two
//! decisions on one shelf. One scratch site of hand-written HTML in the layout
//! MkDocs writes with directory URLs, so no generator has to be installed. The
//! clean site exits 0 with no finding. Each of the four cases after it is one
//! edit away from the clean site and asserts exactly one finding, with its
//! path. **The stale page is the case the verb exists for**: no check of the
//! source can see it, because the source no longer holds the file.
//!
//! Each case is its own corpus. Cargo runs the cases of one target as threads
//! of one process, so a directory keyed on the process alone is one case
//! removes while another reads it.

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root resolves")
}

const OVERLAY_WITH_NAV: &str = "add:\n  identifier_schemes.decision_id.namespace: ACME\nadd_to:\n  projections:\n    - {kind: site_nav, output: .headwater/nav.yml}\n";
const OVERLAY_WITHOUT_NAV: &str = "add:\n  identifier_schemes.decision_id.namespace: ACME\n";

struct Outcome {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl std::fmt::Debug for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "exit {:?}\n--- stdout\n{}--- stderr\n{}",
            self.code, self.stdout, self.stderr
        )
    }
}

struct Root {
    at: PathBuf,
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.at);
    }
}

impl Root {
    fn new(label: &str, overlay: &str) -> Root {
        let at =
            std::env::temp_dir().join(format!("headwater-cli-site-{}-{label}", std::process::id()));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(at.join("docs")).expect("the corpus directory is made");
        let root = Root { at };
        copy(
            &repository().join(".headwater/packages/headwater-standard"),
            &root.at.join(".headwater/packages/headwater-standard"),
        );
        // `init` proposes a corpus root from a directory holding Markdown, so
        // one file is there for it and gone after.
        std::fs::write(root.at.join("docs/seed.md"), "# seed\n").expect("the seed writes");
        let init = root.run(&["init"]);
        assert_eq!(init.code, Some(0), "the corpus initializes\n{init:?}");
        std::fs::remove_file(root.at.join("docs/seed.md")).expect("the seed goes");
        std::fs::write(root.at.join(".headwater/overlay.yml"), overlay)
            .expect("the overlay writes");
        let resolved = root.run(&["taxonomy", "resolve"]);
        assert_eq!(
            resolved.code,
            Some(0),
            "the taxonomy resolves\n{resolved:?}"
        );
        for title in ["Alpha choice", "Beta choice"] {
            let made = root.run(&["new", "decision", "--title", title]);
            assert_eq!(made.code, Some(0), "the decision scaffolds\n{made:?}");
        }
        let generated = root.run(&["generate"]);
        assert_eq!(
            generated.code,
            Some(0),
            "the projections write\n{generated:?}"
        );
        root.clean_site();
        root
    }

    /// The site a build of this corpus writes with directory URLs: the shelf
    /// index, the two decisions, a home page and a 404 page outside every
    /// shelf. Each decision links to the other with a fragment the target
    /// holds, and to the shelf index with no fragment.
    fn clean_site(&self) {
        self.page(
            "index.html",
            "<a href=\"decisions/\">Decisions</a><a href=\"https://example.org/\">out</a>",
        );
        // A link that opens with `/` depends on where the site is served, so
        // it is not read. This one would resolve to nothing in the directory.
        self.page("404.html", "<a href=\"/headwater/\">home</a>");
        self.page(
            "decisions/index.html",
            "<h1 id=\"decision-records\">Decision records</h1><a href=\"0001-alpha-choice/\">1</a><a href=\"0002-beta-choice/\">2</a>",
        );
        self.page(
            "decisions/0001-alpha-choice/index.html",
            // `#caf%C3%A9` names the `id` `café`: a generator writes an `id`
            // decoded and a link percent-encoded, and the two must meet.
            "<h1 id=\"alpha\">Alpha</h1><h2 id=\"caf\u{e9}\">Caf\u{e9}</h2><a href=\"../0002-beta-choice/#beta\">beta</a><a href=\"../\">up</a><a href=\"#alpha\">here</a><a href=\"#caf%C3%A9\">cafe</a>",
        );
        self.page(
            "decisions/0002-beta-choice/index.html",
            "<h1 id=\"beta\">Beta</h1><a href=\"../0001-alpha-choice/#alpha\">alpha</a><a href=\"../\">up</a>",
        );
    }

    fn page(&self, relative: &str, body: &str) {
        let path = self.at.join("site").join(relative);
        std::fs::create_dir_all(path.parent().expect("a page has a parent"))
            .expect("the page directory is made");
        std::fs::write(
            &path,
            format!("<!doctype html><html><body>{body}</body></html>\n"),
        )
        .expect("the page writes");
    }

    fn run(&self, verb: &[&str]) -> Outcome {
        let output = Command::new(env!("CARGO_BIN_EXE_headwater"))
            .args(verb)
            .arg("--root")
            .arg(&self.at)
            .output()
            .expect("the binary runs");
        Outcome {
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    fn site(&self) -> Outcome {
        let dir = self.at.join("site");
        self.run(&["site", dir.to_str().expect("a UTF-8 path")])
    }
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("the target directory is made");
    for entry in std::fs::read_dir(from).expect("the source reads") {
        let entry = entry.expect("the entry reads");
        let target = to.join(entry.file_name());
        match entry.file_type().expect("the type reads").is_dir() {
            true => copy(&entry.path(), &target),
            false => {
                std::fs::copy(entry.path(), &target).expect("the file copies");
            }
        }
    }
}

/// The finding lines of one run: every line that opens with a class.
fn findings(outcome: &Outcome) -> Vec<String> {
    outcome
        .stdout
        .lines()
        .filter(|line| line.starts_with("site."))
        .map(str::to_string)
        .collect()
}

/// Exactly one finding, of `rule`, about `at`.
fn one(outcome: &Outcome, rule: &str, at: &str) {
    let found = findings(outcome);
    assert_eq!(outcome.code, Some(1), "a finding exits 1\n{outcome:?}");
    assert_eq!(found.len(), 1, "exactly one finding\n{outcome:?}");
    assert!(
        found[0].starts_with(&format!("{rule}  {at}:")),
        "the finding is {rule} at {at}\n{outcome:?}"
    );
}

#[test]
fn the_clean_site_exits_0_with_no_finding() {
    let root = Root::new("clean", OVERLAY_WITH_NAV);
    let outcome = root.site();
    assert_eq!(outcome.code, Some(0), "{outcome:?}");
    assert!(findings(&outcome).is_empty(), "{outcome:?}");
    assert!(
        outcome
            .stdout
            .contains("0 findings over 3 navigation entries, 5 pages"),
        "the summary counts what was read\n{outcome:?}"
    );
}

#[test]
fn a_page_the_navigation_names_and_the_site_lacks_is_missing() {
    let root = Root::new("missing", OVERLAY_WITH_NAV);
    std::fs::remove_file(root.at.join("site/decisions/0002-beta-choice/index.html"))
        .expect("the page goes");
    // The two links onto the page would also die. Point them elsewhere, so
    // that the one finding is the one this case is about.
    root.page(
        "decisions/0001-alpha-choice/index.html",
        "<h1 id=\"alpha\">Alpha</h1><a href=\"../\">up</a>",
    );
    root.page(
        "decisions/index.html",
        "<a href=\"0001-alpha-choice/\">1</a>",
    );
    one(
        &root.site(),
        "site.page.missing",
        "decisions/0002-beta-choice.md",
    );
}

/// A shelf index is a page the navigation names too (#528). Its source is a
/// `README.md`, so its page has one form rather than two, and the deletion
/// also kills every link onto it, which the case allows and does not count.
#[test]
fn a_shelf_index_the_site_lacks_is_missing() {
    let root = Root::new("missing-index", OVERLAY_WITH_NAV);
    std::fs::remove_file(root.at.join("site/decisions/index.html")).expect("the index goes");
    let outcome = root.site();
    assert_eq!(outcome.code, Some(1), "{outcome:?}");
    let missing: Vec<String> = findings(&outcome)
        .into_iter()
        .filter(|line| line.starts_with("site.page.missing"))
        .collect();
    assert_eq!(missing.len(), 1, "one missing page\n{outcome:?}");
    assert!(
        missing[0].starts_with("site.page.missing  decisions/README.md:"),
        "the missing page is the shelf index\n{outcome:?}"
    );
    assert!(
        findings(&outcome)
            .iter()
            .all(|line| line.starts_with("site.page.missing") || line.starts_with("site.link.dead")),
        "the deletion gives a missing page and dead links, nothing else\n{outcome:?}"
    );
}

/// The case the verb exists for. A page under a shelf for a document the
/// corpus does not hold: deleted, moved, or never typed.
#[test]
fn a_page_under_a_shelf_that_no_document_answers_to_is_stale() {
    let root = Root::new("stale", OVERLAY_WITH_NAV);
    root.page("decisions/gone/index.html", "<h1 id=\"gone\">Gone</h1>");
    one(&root.site(), "site.page.stale", "decisions/gone/index.html");
}

/// The fragment-less case #350 folded in: a link to a directory with no
/// index answers 404, and a checker that reads only fragments passes it.
#[test]
fn a_link_to_a_file_the_site_lacks_is_dead() {
    let root = Root::new("link", OVERLAY_WITH_NAV);
    root.page(
        "decisions/0002-beta-choice/index.html",
        "<h1 id=\"beta\">Beta</h1><a href=\"../0001-alpha-choice/#alpha\">alpha</a><a href=\"../\">up</a><a href=\"../other/\">other</a>",
    );
    one(
        &root.site(),
        "site.link.dead",
        "decisions/0002-beta-choice/index.html",
    );
}

#[test]
fn a_fragment_that_names_no_id_on_its_page_is_dead() {
    let root = Root::new("fragment", OVERLAY_WITH_NAV);
    root.page(
        "decisions/0001-alpha-choice/index.html",
        "<h1 id=\"alpha\">Alpha</h1><a href=\"../0002-beta-choice/#nope\">beta</a><a href=\"../\">up</a>",
    );
    one(
        &root.site(),
        "site.fragment.dead",
        "decisions/0001-alpha-choice/index.html",
    );
}

/// A page outside every shelf's directory is not the corpus's to account
/// for: a home page, a 404 page, search, and a theme's assets.
#[test]
fn a_page_outside_every_shelf_is_not_read_as_stale() {
    let root = Root::new("outside", OVERLAY_WITH_NAV);
    root.page("search/index.html", "<p>search</p>");
    let outcome = root.site();
    assert_eq!(outcome.code, Some(0), "{outcome:?}");
}

#[test]
fn an_empty_directory_is_refused() {
    let root = Root::new("empty", OVERLAY_WITH_NAV);
    let _ = std::fs::remove_dir_all(root.at.join("site"));
    std::fs::create_dir_all(root.at.join("site")).expect("the empty site is made");
    let outcome = root.site();
    assert_eq!(outcome.code, Some(1), "{outcome:?}");
    assert!(
        outcome.stderr.contains("holds no `.html` file"),
        "{outcome:?}"
    );
    assert!(
        outcome.stdout.is_empty(),
        "a refusal prints no report\n{outcome:?}"
    );
}

#[test]
fn a_directory_that_is_not_there_is_refused() {
    let root = Root::new("absent", OVERLAY_WITH_NAV);
    let _ = std::fs::remove_dir_all(root.at.join("site"));
    let outcome = root.site();
    assert_eq!(outcome.code, Some(1), "{outcome:?}");
    assert!(outcome.stderr.contains("is not a directory"), "{outcome:?}");
}

#[test]
fn a_taxonomy_with_no_site_nav_is_refused() {
    let root = Root::new("no-nav", OVERLAY_WITHOUT_NAV);
    let outcome = root.site();
    assert_eq!(outcome.code, Some(1), "{outcome:?}");
    assert!(
        outcome.stderr.contains("declares no `site_nav` projection"),
        "{outcome:?}"
    );
}
