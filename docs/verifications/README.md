<!-- headwater:generated shelf_index. `headwater generate` writes this file, and `headwater generate --check` holds it. Edit the corpus, not this file. -->

# Verifications

The documents on this shelf, in the reading order this corpus derives.

- [The section-heading rule has no case table, and a golden fixture report is what proves it runs](0001-the-section-heading-rule-has-no-case-table-and-a-golden-fixture-report-is-what-proves-it-runs.md) — A golden fixture report in engine/crates/check/tests/fixtures.rs is what proves HW-AC-0002, not a per-shape case table. (asserted, and no human has accepted it)
- [A fixture tree of document paths, symlinks and identifiers proves the document path finding](0002-a-fixture-tree-of-document-paths-symlinks-and-identifiers-proves-the-document-path-finding.md) — Seven tests over one fixture tree prove HW-AC-0003: one note per case, two symlinks, and a resolver outside the tree, each read against relation.target.unresolved. (asserted, and no human has accepted it)
- [A fixture tree of three list anchors proves that the export carries list members](0003-a-fixture-tree-of-three-list-anchors-proves-that-the-export-carries-list-members.md) — One fixture decision governs a single path and three lists, one of them holding a member named with the separator, and one test reads the export back against HW-AC-0004. (asserted, and no human has accepted it)
- [The ingest stubs of the query fixture tree prove what governs a path](0004-the-ingest-stubs-of-the-query-fixture-tree-prove-what-governs-a-path.md) — Two stub files that a pattern and a list both reach, read by a recorded report and an MCP spelling test, prove HW-AC-0005. (asserted, and no human has accepted it)
