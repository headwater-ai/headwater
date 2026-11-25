// SPDX-License-Identifier: Apache-2.0
//! The decisive fixture for #2016's first slice: `headwater check --fix`
//! writes `verified_revision` onto a `governs` entry whose target is a list.
//!
//! Before it, `relation.target.suspect` offered no patch on a list entry,
//! because the splice matched an entry only by one scalar `to`. A list entry
//! then stayed unstamped for good: 28 of the 389 `governs` entries in this
//! repository's `docs/` were lists, and none of them carried a digest. The
//! check crate cannot apply a fix, so the case lives here, where the check
//! layer runs and its patch is composed and written by the same code the CLI
//! calls.
//!
//! Each row writes one list in one form, under a change that re-verifies the
//! document, and holds three things. The first `--fix` writes the digest
//! `tree_revision` reaches onto the entry, in the mapping form, with the
//! members in the author's order. A second run over the same change offers no
//! patch and leaves the bytes alone. And after a member moves, a `verified`
//! line that names the edge re-stamps the same entry, so the list holds one
//! entry and not two.

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
const TODAY: &str = "2026-09-24";
const YESTERDAY: &str = "2026-09-23";
const DOCUMENT: &str = "governs-suspect/hooks.md";

/// A directory under the temporary directory that is removed when this value
/// is dropped, keyed on the row and the pid, because cargo runs the cases of
/// one target as threads of one process.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(label: &str, members: &[&str]) -> Scratch {
    let root = Scratch(std::env::temp_dir().join(format!(
        "headwater-list-stamp-{label}-{}",
        std::process::id()
    )));
    let _ = std::fs::remove_dir_all(&root.0);
    for member in members {
        write(&root.0, member, &format!("#!/bin/sh\necho {member}\n"));
    }
    root
}

fn write(root: &Path, path: &str, bytes: &str) {
    let at = root.join(path);
    std::fs::create_dir_all(at.parent().expect("a parent")).expect("the directory creates");
    std::fs::write(at, bytes).expect("the file writes");
}

/// The fixture taxonomy of the check crate's suspect cases, which declares
/// `verified_revision` on `governs` and a `source-tree` anchor kind.
fn taxonomy() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../check/fixtures/governs-suspect.taxonomy.yml");
    std::fs::read_to_string(at).expect("the fixture taxonomy")
}

/// A governing document, with `entry` written verbatim under `governs:`.
fn text_of(last_verified: &str, entry: &str) -> String {
    format!(
        "---\nid: GS-FIX-hooks\nstatus: current\nstatus_since: 2026-01-05\nlast_verified: \
         {last_verified}\nsummary: governs the hooks a commit runs\nrelations:\n  governs:\n\
         {entry}\n---\n\n# Governs the hooks\n\nThe hooks a commit runs.\n"
    )
}

/// A change built from manifest lines, each prior version handed over as
/// bytes so that no file stands for it.
fn change(lines: &[String], priors: &[(String, String)]) -> Change {
    let mut manifest = String::from("headwater change 1\n");
    for line in lines {
        manifest.push_str(line);
        manifest.push('\n');
    }
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

fn run(root: &Path, change: Change) -> Run {
    let source = taxonomy();
    let value = headwater_yaml::load(&source)
        .expect("the fixture taxonomy loads")
        .value
        .as_map()
        .expect("a mapping")
        .clone();
    let taxonomy = Taxonomy::read(&value).expect("the taxonomy reads");
    let declarations = Declarations::read(&value).expect("the declarations read");
    let register = Register::read(&value).expect("the register reads");
    let shape = Shape::read(&value).expect("the shape reads");
    let corpus = Corpus::new(root, "governs-suspect");
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
            lock: "sha256:list-stamp-fixture",
            taxonomy: &taxonomy,
            shape: &shape,
            relations: &declarations,
            config: &config,
            register: &register,
            adoption: None,
            observations: &Observations::empty(),
            pin: None,
            lock_stale: None,
            agent_budgets: &[],
            harvests: &[],
            imports: &[],
            source: "engine/crates/check/fixtures/governs-suspect.taxonomy.yml",
        },
        &headwater_check::claim::Claims::empty(),
        &Context::at(Date::parse(TODAY).expect("the date parses")).scoped_to(change),
        &mut Cache::disabled(),
    )
}

/// The patches of this rule that a run offers.
fn offered(run: &Run) -> Vec<Patch> {
    run.findings
        .iter()
        .filter(|finding| finding.rule == RULE)
        .filter_map(|finding| finding.patch.clone())
        .collect()
}

/// What `headwater check --fix` does with the patches: compose, refuse
/// nothing, write.
fn fix(root: &Path, patches: &[Patch]) {
    let composed = headwater_scaffold::fix::compose(root, patches);
    assert!(composed.refused.is_empty(), "{:?}", composed.refused);
    headwater_scaffold::fix::apply(root, &composed.files).expect("the fix writes");
}

fn digest(root: &Path, members: &[&str]) -> String {
    let owned: Vec<String> = members.iter().map(|member| member.to_string()).collect();
    headwater_graph::anchors::tree_revision(root, &owned).expect("the members carry a digest")
}

/// Each row: a label, the entry as the author wrote it, the members in the
/// author's order, and the line `--fix` writes the target as.
type Row<'a> = (&'a str, &'a str, &'a [&'a str], &'a str);

#[test]
fn fix_stamps_a_list_entry_once_and_re_stamps_the_same_entry() {
    let rows: [Row<'_>; 3] = [
        (
            "bare",
            "    - [b.sh, a.sh]",
            &["b.sh", "a.sh"],
            "    - to: [b.sh, a.sh]",
        ),
        (
            "mapping",
            "    - to: [b.sh, a.sh]",
            &["b.sh", "a.sh"],
            "    - to: [b.sh, a.sh]",
        ),
        // A member that holds a comma is written quoted, or the list would
        // read back as three members.
        (
            "quoted",
            "    - [\"x, y.sh\", a.sh]",
            &["x, y.sh", "a.sh"],
            "    - to: [\"x, y.sh\", a.sh]",
        ),
    ];
    for (label, entry, members, written) in rows {
        let root = scratch(label, members);
        let root = &root.0;
        write(root, DOCUMENT, &text_of(TODAY, entry));
        // The change moved `last_verified`, so it re-verified the document,
        // and the unrecorded entry may be given its first digest.
        let re_verified = || {
            change(
                &[format!("prior\t{DOCUMENT}\tprior/{DOCUMENT}")],
                &[(format!("prior/{DOCUMENT}"), text_of(YESTERDAY, entry))],
            )
        };

        let first = offered(&run(root, re_verified()));
        assert_eq!(first.len(), 1, "{label}: {first:?}");
        fix(root, &first);
        let now = digest(root, members);
        let after = std::fs::read_to_string(root.join(DOCUMENT)).expect("the document reads");
        let stamped = format!("{written}\n      verified_revision: {now}\n");
        assert!(after.contains(&stamped), "{label}:\n{after}");
        assert!(
            !after.contains(entry.trim_end()) || entry == written,
            "{label}:\n{after}"
        );

        // A second run over the same change offers nothing and writes nothing.
        let again = offered(&run(root, re_verified()));
        assert!(again.is_empty(), "{label}: {again:?}");
        fix(root, &again);
        assert_eq!(
            std::fs::read_to_string(root.join(DOCUMENT)).expect("the document reads"),
            after,
            "{label}"
        );

        // One member moves, and a `verified` line names the edge by it.
        write(root, members[1], "#!/bin/sh\necho moved\n");
        let moved = digest(root, members);
        assert_ne!(moved, now, "{label}: the member moved");
        let line = change(&[format!("verified\t{DOCUMENT}\t{}", members[1])], &[]);
        let restamp = offered(&run(root, line));
        assert_eq!(restamp.len(), 1, "{label}: {restamp:?}");
        fix(root, &restamp);
        let last = std::fs::read_to_string(root.join(DOCUMENT)).expect("the document reads");
        assert!(
            last.contains(&format!("{written}\n      verified_revision: {moved}\n")),
            "{label}:\n{last}"
        );
        assert_eq!(
            last.matches("verified_revision").count(),
            1,
            "{label}:\n{last}"
        );
        assert!(!last.contains(&now), "{label}:\n{last}");
    }
}
