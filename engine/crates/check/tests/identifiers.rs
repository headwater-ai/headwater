// SPDX-License-Identifier: Apache-2.0
//! Every identifier this repository's taxonomy can mint, recorded byte for byte.
//!
//! # What this pins, and why it is a fixture rather than an assertion
//!
//! [#207](https://github.com/headwater-ai/headwater/issues/207) rewrote 1,134
//! identifier occurrences across 274 files, so the rendered form of every scheme
//! is now quoted in commit messages, issue bodies and citations that no
//! subsequent change can reach. A later change to how a scheme is *declared*
//! must therefore leave what it *mints* alone, to the byte. An assertion written
//! after such a change proves only that its author agreed with it; a record
//! taken before it is a comparison.
//!
//! So this walks the nine schemes of the resolved taxonomy, renders each one,
//! mints under it with one fixed slug and one fixed sequence value, and reads
//! the result back. The whole surface of [`Template`] is exercised per scheme,
//! because a change that preserved `mint` and moved `sequence_of` would be a
//! change this repository's own identifiers survive and an adopter's do not.
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-check --test identifiers
//!
//! Read the diff before committing it. A line that moves here is an identifier
//! that moves.

use headwater_meta::identifier::{Needs, Template};
use std::path::{Path, PathBuf};

/// The slug and the sequence value every scheme is minted with. Constants,
/// because a record whose inputs vary records two things at once.
const SLUG: &str = "a-name";
const SEQUENCE: u64 = 42;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root")
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/identifiers.mint")
}

/// Each declared scheme, as `(name, pattern, namespace)`, from the resolved
/// taxonomy rather than from any one source. The nine are declared across three
/// files and two of the three declare no namespace at all, so a reader that
/// took a source would see a different set from the one the engine mints under.
fn schemes() -> Vec<(String, String, String)> {
    let root = repository_root();
    let repository = headwater_resolve::repository(&root).expect("this repository resolves");
    let taxonomy = &repository.resolution.taxonomy;
    let block = taxonomy
        .get("identifier_schemes")
        .and_then(|node| node.value.as_map())
        .expect("the resolved taxonomy declares identifier schemes");
    fn text(body: &headwater_yaml::Mapping, key: &str) -> String {
        body.get(key)
            .and_then(|node| node.value.as_scalar())
            .map(|scalar| scalar.text.clone())
            .unwrap_or_default()
    }
    block
        .iter()
        .map(|entry| {
            let body = entry.value.value.as_map().expect("a scheme is a mapping");
            (
                entry.key.value.clone(),
                text(body, "pattern"),
                text(body, "namespace"),
            )
        })
        .collect()
}

fn compare(actual: &str) {
    let path = fixture();
    if std::env::var_os("HEADWATER_BLESS").is_some() {
        std::fs::create_dir_all(path.parent().expect("a directory")).expect("cannot create it");
        std::fs::write(&path, actual).expect("cannot write the expectation");
        return;
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e}. Run with HEADWATER_BLESS=1 to record it.",
            path.display()
        )
    });
    assert_eq!(
        expected,
        actual,
        "\n{} is out of date. A line that moved here is an identifier that moved",
        path.display()
    );
}

#[test]
fn no_identifier_this_taxonomy_mints_changes_value() {
    let mut out = String::new();
    let schemes = schemes();
    out.push_str(&format!("{} schemes\n", schemes.len()));
    for (name, pattern, namespace) in &schemes {
        out.push_str(&format!("\n{name}\n"));
        out.push_str(&format!("  pattern    {pattern}\n"));
        out.push_str(&format!("  namespace  {namespace}\n"));
        let template = match Template::parse(pattern, namespace) {
            Ok(template) => template,
            Err(why) => {
                out.push_str(&format!("  unreadable {why}\n"));
                continue;
            }
        };
        out.push_str(&format!("  render     {}\n", template.render()));
        let needs: Vec<String> = template
            .needs()
            .into_iter()
            .map(|need| match need {
                Needs::Slug => "slug".to_string(),
                Needs::Sequence { width } => format!("seq:{width}"),
            })
            .collect();
        out.push_str(&format!("  needs      {}\n", needs.join(", ")));
        let minted = template
            .mint(Some(SLUG), Some(SEQUENCE))
            .expect("every scheme here mints from a slug and a sequence");
        out.push_str(&format!("  mint       {minted}\n"));
        out.push_str(&format!("  admits     {}\n", template.admits(&minted)));
        out.push_str(&format!(
            "  sequence   {}\n",
            match template.sequence_of(&minted) {
                Some(value) => value.to_string(),
                None => "-".to_string(),
            }
        ));
        // The refusing direction, so the record is not one a template that
        // admitted everything would also produce.
        out.push_str(&format!(
            "  refuses    {}\n",
            !template.admits(&format!("{minted}!"))
        ));
    }
    compare(&out);
}
