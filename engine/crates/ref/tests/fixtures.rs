// SPDX-License-Identifier: Apache-2.0
//! The sublanguage's fixture corpus.
//!
//! Each case file under `fixtures/` holds one input per line, and the `.record`
//! beside it holds what the grammar makes of every one of them. A blank line
//! and a `#` line are skipped. `~` on its own is the empty string, which no
//! other notation in a line-per-case file can write.
//!
//! Accepted and refused inputs share one file, because the point of a grammar
//! is the boundary rather than either side of it. Splitting them would put the
//! two halves of one rule in two places.
//!
//! To re-record after a deliberate change:
//!
//!     HEADWATER_BLESS=1 cargo test -p headwater-ref --test fixtures
//!
//! Read the diff before committing it. A blessed fixture is the change.

use headwater_ref::{classify, Address, Reference, Scalar};
use std::path::{Path, PathBuf};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn blessing() -> bool {
    std::env::var_os("HEADWATER_BLESS").is_some()
}

/// Every case in `name`, in file order, with `~` read as the empty string.
fn cases(name: &str) -> Vec<String> {
    let path = fixtures_dir().join(format!("{name}.cases"));
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let found: Vec<String> = source
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| if line == "~" { "" } else { line }.to_string())
        .collect();
    assert!(!found.is_empty(), "no cases in {}", path.display());
    found
}

fn compare(name: &str, actual: &str) {
    let path = fixtures_dir().join(format!("{name}.record"));
    if blessing() {
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
        expected, actual,
        "\n{name}.cases does not match {name}.record"
    );
}

/// The input as the record writes it, so an empty case is visible.
fn shown(input: &str) -> &str {
    if input.is_empty() {
        "~"
    } else {
        input
    }
}

/// The parse rather than the input. A record that reprinted the address would
/// show nothing, and the one field with no visible symptom when it is wrong is
/// where the parser put each boundary.
fn split(address: &Address) -> String {
    address.segments().join(" / ")
}

#[test]
fn addresses_parse_to_the_recorded_segments() {
    let mut out = String::new();
    for input in cases("addresses") {
        let line = match Address::parse(&input) {
            Ok(address) => split(&address),
            Err(error) => format!("error at {}: {error}", error.at),
        };
        out.push_str(&format!("{} => {line}\n", shown(&input)));
    }
    compare("addresses", &out);
}

#[test]
fn references_parse_to_the_recorded_root_and_address() {
    let mut out = String::new();
    for input in cases("references") {
        let line = match Reference::parse(&input) {
            Ok(reference) => format!(
                "root {}, address {}",
                reference.root().name(),
                split(reference.address())
            ),
            Err(error) => format!("error at {}: {error}", error.at),
        };
        out.push_str(&format!("{} => {line}\n", shown(&input)));
    }
    compare("references", &out);
}

#[test]
fn scalars_classify_to_the_recorded_reading() {
    let mut out = String::new();
    for input in cases("scalars") {
        let line = match classify(&input) {
            Ok(Scalar::Reference(reference)) => format!("reference {reference}"),
            Ok(Scalar::Literal(literal)) => format!("literal {literal:?}"),
            Err(error) => format!("error at {}: {error}", error.at),
        };
        out.push_str(&format!("{} => {line}\n", shown(&input)));
    }
    compare("scalars", &out);
}

#[test]
fn address_pairs_record_which_subtrees_overlap() {
    let mut out = String::new();
    for input in cases("disjoint") {
        let (left, right) = input.split_once(' ').expect("two addresses on a line");
        let left = Address::parse(left).expect("the left address");
        let right = Address::parse(right).expect("the right address");
        let verdict = if left.is_disjoint_from(&right) {
            "disjoint"
        } else {
            "overlapping"
        };
        out.push_str(&format!("{left} vs {right} => {verdict}\n"));
    }
    compare("disjoint", &out);
}

/// Every address this repository's own sources already write.
///
/// The loader crate keeps the same test for the same reason. A constructed
/// fixture cannot fail the way a file somebody wrote for another purpose can,
/// and this grammar was settled after both of these files were written. A
/// grammar that refused one of them would be a finding against a committed
/// overlay rather than a rule.
#[test]
fn this_repositorys_own_overlay_addresses_parse() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root");
    let sources = [
        root.join(".headwater/overlay.yml"),
        root.join("docs/taxonomies/design-spec/bundle.yml"),
    ];
    let mut seen = 0;
    for path in sources {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let loaded = headwater_yaml::load(&source)
            .unwrap_or_else(|_| panic!("{} does not load", path.display()));
        let map = loaded.as_map().expect("a mapping at the root");
        let operation = map.get("add").expect("an `add` block");
        for entry in operation.as_map().expect("`add` holds a mapping") {
            let key = &entry.key.value;
            Address::parse(key)
                .unwrap_or_else(|e| panic!("{}: `{key}` is not an address: {e}", path.display()));
            seen += 1;
        }
    }
    // The count is here so that a source which stops holding an `add` block
    // makes this test fail rather than pass over nothing.
    assert_eq!(seen, 31, "the two sources declare 31 add operations");
}
