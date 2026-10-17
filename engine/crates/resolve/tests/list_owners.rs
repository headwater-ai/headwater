// SPDX-License-Identifier: Apache-2.0
//! Which source wrote each item of a resolved list (#1758).
//!
//! `taxonomy resolve` refuses a projection the generator refuses and names the
//! source that wrote it. The generator counts the merged list, so each case
//! below holds one operation's effect on the owner of each position, and the
//! cases the replay cannot answer hold that it says so rather than guessing.

use headwater_resolve::{list_owners, list_writers, operation, Operation, Role, Source};
use headwater_yaml::Mapping;

const BASE: &str = "projections:\n  - {kind: shelf_index, output: a.md}\n  - {kind: shelf_index, output: b.md}\n";

fn base(text: &str) -> Mapping {
    Source::from_text("base.yml", Role::Taxonomy, text)
        .expect("the base loads")
        .root
        .value
        .as_map()
        .cloned()
        .expect("the base is a mapping")
}

/// The operations of each overlay, numbered from 1 in the order given, which
/// is the order the resolver applies them in.
fn operations(overlays: &[&str]) -> Vec<Operation> {
    overlays
        .iter()
        .enumerate()
        .flat_map(|(index, text)| {
            let source = Source::from_text("overlay.yml", Role::Overlay, text)
                .expect("the overlay loads");
            operation::read(index + 1, &source.name, &source.root).expect("the operations read")
        })
        .collect()
}

fn path() -> Vec<String> {
    vec!["projections".to_string()]
}

#[test]
fn the_base_items_come_first_and_each_add_to_appends_its_own() {
    let ops = operations(&[
        "add_to:\n  projections:\n    - {kind: shelf_index, output: c.md}\n",
        "add_to:\n  projections:\n    - {kind: shelf_index, output: d.md}\n    - {kind: shelf_index, output: e.md}\n",
    ]);
    assert_eq!(
        list_owners(&base(BASE), &ops, &path()),
        Some(vec![0, 0, 1, 2, 2])
    );
    assert_eq!(list_writers(&base(BASE), &ops, &path()), vec![0, 1, 2]);
}

#[test]
fn a_base_with_no_list_starts_from_nothing() {
    let ops = operations(&["add_to:\n  kinds.x.y: [z]\n"]);
    assert_eq!(
        list_owners(&base("kinds: {}\n"), &ops, &path()),
        Some(vec![])
    );
    assert_eq!(list_writers(&base("kinds: {}\n"), &ops, &path()), Vec::<usize>::new());
}

#[test]
fn remove_from_takes_out_the_item_that_is_the_same_and_shifts_the_rest() {
    let ops = operations(&[
        "add_to:\n  projections:\n    - {kind: shelf_index, output: c.md}\n",
        "remove_from:\n  projections:\n    - {kind: shelf_index, output: a.md}\n",
    ]);
    assert_eq!(list_owners(&base(BASE), &ops, &path()), Some(vec![0, 1]));
}

#[test]
fn remove_from_an_item_the_list_does_not_hold_cannot_be_answered() {
    let ops = operations(&["remove_from:\n  projections:\n    - {kind: shelf_index, output: z.md}\n"]);
    assert_eq!(list_owners(&base(BASE), &ops, &path()), None);
}

#[test]
fn override_replaces_every_owner() {
    let ops = operations(&[
        "add_to:\n  projections:\n    - {kind: shelf_index, output: c.md}\n",
        "override:\n  projections:\n    - {kind: shelf_index, output: d.md}\n",
    ]);
    assert_eq!(list_owners(&base(BASE), &ops, &path()), Some(vec![2]));
}

#[test]
fn remove_empties_the_list() {
    let ops = operations(&["remove:\n  - projections\n"]);
    assert_eq!(list_owners(&base(BASE), &ops, &path()), Some(vec![]));
}

#[test]
fn an_operation_above_or_below_the_list_cannot_be_answered() {
    let below = operations(&["override:\n  projections.x: y\n"]);
    assert_eq!(list_owners(&base(BASE), &below, &path()), None);
    assert_eq!(list_writers(&base(BASE), &below, &path()), vec![0, 1]);
    let nested = vec!["a".to_string(), "b".to_string()];
    let above = operations(&["override:\n  a: {b: [x]}\n"]);
    assert_eq!(list_owners(&base("a: {b: [w]}\n"), &above, &nested), None);
}

#[test]
fn a_list_written_as_something_else_cannot_be_answered() {
    let ops = operations(&["override:\n  projections: none\n"]);
    assert_eq!(list_owners(&base(BASE), &ops, &path()), None);
}
