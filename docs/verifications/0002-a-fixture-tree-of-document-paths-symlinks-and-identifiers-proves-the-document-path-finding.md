---
id: HW-VER-0002
status: current
status_since: 2026-10-02
summary: "Seven tests over one fixture tree prove HW-AC-0003: one note per case, two symlinks, and a resolver outside the tree, each read against relation.target.unresolved."
last_verified: 2026-10-02
title: "A fixture tree of document paths, symlinks and identifiers proves the document path finding"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  proves:
    - HW-AC-0003
  governs:
    - engine/crates/check/tests/document_path_target.rs
    - engine/crates/check/fixtures/document-path-target/**
    - engine/crates/check/fixtures/document-path-target.taxonomy.yml
  cited_in:
    - engine/crates/check/tests/document_path_target.rs
---

# A fixture tree of document paths, symlinks and identifiers proves the document path finding

## Approach

The fixture tree at `engine/crates/check/fixtures/document-path-target/` holds one note for each case of [HW-AC-0003](../acceptance-criteria/0003-check-reports-a-document-path-written-as-a-relation-target-once-in-every-spelling-and-through-a-symlink.md). `engine/crates/check/fixtures/document-path-target.taxonomy.yml` declares the kinds and relations the notes use. `NOTE-FIX-b` in `notes/b.md` is the target document that every path case names.

The cases, and the file that carries each one:

- **The decisive case.** `notes/a.md` writes the path of `NOTE-FIX-b` under `traces_to`.
- **Spelling.** `notes/a2.md` writes the path with `./` and a `..` segment. `notes/k.md` writes it in two spellings in one entry.
- **Symlinks.** `src/alias.md` is a symlink onto `notes/b.md`, and `linked/` is a symlink onto `notes/`. `notes/l.md` and `notes/l2.md` write a path through each one.
- **Path and identifier together.** `notes/m.md` writes the path and then `NOTE-FIX-b`.
- **The identifier form.** `notes/c.md` writes `NOTE-FIX-b`.
- **Paths that name no document.** `notes/d.md` traces to `src/lib.rs`, and `notes/n.md` traces to `src/lib-link.rs`, a symlink onto it. `notes/e.md` writes a wildcard, `notes/g.md` names a file with no identifier, and `notes/j.md` names a file with no kind. `notes/h.md` writes the path under `cites`, whose resolver is a fixture snapshot outside the tree. `notes/f.md` writes the path under `governs`.

`engine/crates/check/tests/document_path_target.rs` builds the census, the graph and one check run over the tree, with a pinned clock. Each of its seven tests reads the findings of `relation.target.unresolved` on one or more notes, and the graph's target for each entry.

    cargo test -p headwater-check --test document_path_target --manifest-path engine/Cargo.toml --locked

The two source files under `src/` hold no code that the engine runs. They exist so that a path to a source file, and a symlink onto one, resolve to a real entry on the tree.

## What this does not cover

The tree proves the rule over one taxonomy, and that taxonomy declares two relations that admit a document and an anchor. It does not prove every relation of every package.

The test does not run `headwater check` from the command line. It calls the check library, so the CLI's output format for the finding is outside this test.

This document governs the test file and the fixture tree. A change to either one is a change to the test design, and a reader then checks this document against the change. The comment that cites `HW-VER-0002` in the test file binds the `cited_in` edge.
