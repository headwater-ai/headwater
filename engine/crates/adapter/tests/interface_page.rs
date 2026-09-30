// SPDX-License-Identifier: Apache-2.0
//! The interface page for `headwater check`, held to the JSON shape this crate
//! writes (#1451).
//!
//! A consumer of `headwater check --format json` holds no clone of this
//! repository, so the page is the only statement of the shape it gets. Before
//! #1451 the only record of what each version added was the doc comment on
//! [`headwater_adapter::json::VERSION`], which that consumer never reads. The
//! two cases here fail when the constant or the members move and the page does
//! not.
//!
//! The members come from `tests/fixture.json` and `tests/fixture.scoped.json`,
//! which `fixtures.rs` re-records from a real run, rather than from a second run
//! here. Between them the two reach every member the table names: the scoped
//! one writes `change`, and the full one writes `compared` on the two rules that
//! carry it.

use std::path::PathBuf;

/// The heading on the interface page that the member table sits under.
const MEMBER_TABLE: &str = "### The JSON report, member by member";

/// The page this suite holds.
const PAGE: &str = "docs/interfaces/headwater-check.md";

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn page() -> String {
    let path = repository().join(PAGE);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The version the page states as current, from the sentence that states it.
fn page_version(page: &str) -> Option<String> {
    let (_, after) = page.split_once("The current version is `")?;
    let (version, _) = after.split_once('`')?;
    Some(version.to_string())
}

/// The member column of each row of the table under [`MEMBER_TABLE`].
fn page_members(page: &str) -> Vec<String> {
    let mut rows = Vec::new();
    for line in page
        .lines()
        .skip_while(|line| *line != MEMBER_TABLE)
        .skip(1)
        .take_while(|line| !line.starts_with('#'))
    {
        let Some(row) = line.strip_prefix("| `") else {
            continue;
        };
        let Some((member, _)) = row.split_once("` |") else {
            continue;
        };
        rows.push(member.to_string());
    }
    rows
}

/// The members of one recorded report that the table accounts for: every
/// top-level member, and every member of `taxonomy`, of `change` and of an
/// entry of `rules`. The table names `coverage`, `findings` and `read_set` as
/// top-level members and sends the reader elsewhere for their insides.
fn written_members(fixture: &str, into: &mut Vec<String>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join(fixture);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let value = headwater_yaml::load(&text)
        .unwrap_or_else(|errors| panic!("{} does not parse: {errors:?}", path.display()))
        .value;
    let top = value.as_map().expect("the report is an object");
    let mut push = |member: String| {
        if !into.contains(&member) {
            into.push(member);
        }
    };
    for entry in top.entries() {
        let key = entry.key.value.clone();
        match key.as_str() {
            "taxonomy" | "change" => {
                let inner = entry.value.value.as_map().expect("an object");
                for member in inner.entries() {
                    push(format!("{key}.{}", member.key.value));
                }
            }
            "rules" => {
                let entries = entry.value.value.as_seq().expect("an array");
                for rule in entries {
                    let rule = rule.value.as_map().expect("a rules entry is an object");
                    for member in rule.entries() {
                        push(format!("rules[].{}", member.key.value));
                    }
                }
            }
            _ => {}
        }
        push(key);
    }
}

/// The page states the version the engine writes, and names every version of
/// the shape from `1.0` to it. Bump [`headwater_adapter::json::VERSION`] and
/// leave the page, and this fails naming the new version.
#[test]
fn the_check_interface_page_names_every_version_of_the_json_shape() {
    let page = page();
    let stated = page_version(&page).unwrap_or_else(|| panic!("{PAGE} states no current version"));
    let current = headwater_adapter::json::VERSION;
    assert_eq!(
        stated, current,
        "the engine writes JSON shape {current} and {PAGE} states {stated} as current"
    );
    let minor: u32 = current
        .strip_prefix("1.")
        .and_then(|minor| minor.parse().ok())
        .unwrap_or_else(|| panic!("json::VERSION {current} is not `1.<minor>`: the page test reads only a first major"));
    let missing: Vec<String> = (0..=minor)
        .map(|k| format!("`1.{k}`"))
        .filter(|version| !page.contains(version.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "{PAGE} does not name these versions of the JSON shape: {missing:?}"
    );
}

/// Every member of a `rules` entry that a run writes is a row of the member
/// table, and every row is a member some run writes, so a stale row fails too.
/// The same holds for the top-level members and those of `taxonomy` and
/// `change`.
#[test]
fn every_member_of_a_rules_entry_is_named_on_the_check_page() {
    let mut written = Vec::new();
    written_members("fixture.json", &mut written);
    written_members("fixture.scoped.json", &mut written);
    assert!(
        written.iter().any(|member| member == "rules[].compared"),
        "neither fixture writes `compared`, so this case no longer reaches the conditional member"
    );

    let page = page();
    let named = page_members(&page);
    let unnamed: Vec<&String> = written
        .iter()
        .filter(|member| !named.contains(member))
        .collect();
    assert!(
        unnamed.is_empty(),
        "the JSON report writes members that {PAGE} does not name under `{MEMBER_TABLE}`: {unnamed:?}"
    );
    let unseen: Vec<&String> = named
        .iter()
        .filter(|member| !written.contains(member))
        .collect();
    assert!(
        unseen.is_empty(),
        "{PAGE} names members under `{MEMBER_TABLE}` that no recorded report writes: {unseen:?}"
    );
}
