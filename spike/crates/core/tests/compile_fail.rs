// SPDX-License-Identifier: Apache-2.0
//! **Item 2 of the spike**, and the one that can falsify the Q1 decision.
//!
//! The claim under test, from `docs/evaluations/language-choice.md`:
//!
//! > A test that fails to compile when a `Document`-scoped check reaches a
//! > sibling. A test that compiles proves the claim false.
//!
//! The positive controls run first and matter as much as the failures. Without
//! them, a crate that does not build at all would look like a perfect score.

#[test]
fn scope_enforcement() {
    let t = trybuild::TestCases::new();

    // ---- positive controls: legitimate checks must compile ----
    t.pass("tests/ui/pass_document_check.rs");
    t.pass("tests/ui/pass_corpus_check.rs");
    t.pass("tests/ui/pass_edge_check.rs");

    // ---- the leaks ----
    t.compile_fail("tests/ui/leak_sibling.rs");
    t.compile_fail("tests/ui/leak_construct_view.rs");
    t.compile_fail("tests/ui/leak_escape_lifetime.rs");
    t.compile_fail("tests/ui/leak_downcast.rs");
    t.compile_fail("tests/ui/leak_mutate_self.rs");
    t.compile_fail("tests/ui/leak_interior_mutability.rs");
}
