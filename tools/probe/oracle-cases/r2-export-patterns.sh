# The cases of the `r2-export-patterns` oracle, for the probe
# HW-PROBE-a-session-lets-the-graph-export-list-the-patterns-of-every-anchor
# (#1874, slice 4b).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The oracle builds `headwater-cli` in every case, so it runs
# under `--cargo` alone.

needs_cargo=yes

# The sound change: `patterns` for every anchor, the export version raised
# to `1.4`, and the interface page moved with it.
reference() {
patch -p1 --quiet <<'R2_REFERENCE'
--- a/engine/crates/generate/src/export.rs
+++ b/engine/crates/generate/src/export.rs
@@ -72,8 +72,9 @@
 /// `1.2` added `patterns` ([#1247](https://github.com/headwater-ai/headwater/issues/1247),
 /// see [`list_members`]), and `1.2` to `1.3` added `identifiers` to each
 /// `counted` tombstone ([#1309](https://github.com/headwater-ai/headwater/issues/1309),
-/// see [HW-DR-0100](../../../../docs/decisions/0100-a-counted-tombstone-lists-a-digest-of-each-withheld-identifier-and-a-sealed-one-lists-nothing.md)).
-pub const VERSION: &str = "1.3";
+/// see [HW-DR-0100](../../../../docs/decisions/0100-a-counted-tombstone-lists-a-digest-of-each-withheld-identifier-and-a-sealed-one-lists-nothing.md)),
+/// and `1.3` to `1.4` wrote `patterns` for a single-pattern anchor too.
+pub const VERSION: &str = "1.4";
 
 /// What a loss is about.
 ///
@@ -606,12 +607,12 @@
 /// A list anchor's `id` is the length-prefixed identity of HW-DR-0074, which
 /// is a key and not a path. So a consumer that wants the files an anchor
 /// governs reads `patterns`, the sorted normalized members, and never decodes
-/// `id` (#1247). A single-pattern anchor writes no `patterns`, because its
-/// `id` is already its one pattern: a consumer reads `patterns` when it is
-/// there and `id` as the one pattern when it is not. This member moved the
-/// export version from `1.1` to `1.2`.
+/// `id` (#1247). A single-pattern anchor writes `patterns` too, a list of the
+/// one pattern that is its `id`, so a consumer reads one member for every
+/// anchor. This member moved the export version from `1.1` to `1.2`, and
+/// writing it for every anchor moved it from `1.3` to `1.4`.
 fn list_members(patterns: &[String]) -> Option<Json> {
-    (patterns.len() > 1).then(|| Json::Array(patterns.iter().map(Json::string).collect()))
+    (!patterns.is_empty()).then(|| Json::Array(patterns.iter().map(Json::string).collect()))
 }
 
 fn of_value_map(map: &headwater_yaml::Mapping) -> Json {
--- a/docs/interfaces/headwater-export.md
+++ b/docs/interfaces/headwater-export.md
@@ -135,7 +135,7 @@
 | `graph.anchors[].anchor_kind` | Always | The anchor kind. |
 | `graph.anchors[].id` | Always | The normalized identity of the anchor. For an anchor that holds more than one pattern, it is a key and not a path. |
 | `graph.anchors[].resolver` | Always | The resolver that bound the anchor. |
-| `graph.anchors[].patterns` | When the anchor holds more than one pattern | The sorted, normalized patterns. Read this member for the files of a list anchor, and do not decode `id`. |
+| `graph.anchors[].patterns` | Always, from `1.4`. Before `1.4`, only when the anchor holds more than one pattern | The sorted, normalized patterns. A single-pattern anchor lists its one pattern, which is its `id`. Read this member for the files of an anchor, and do not decode `id`. |
 | `graph.anchors[].excluded_by` | When a corpus exclusion claims a single literal pattern | The exclusion pattern that claims the anchor. |
 | `graph.edges` | Always | One element for each edge that the profile carries. |
 | `graph.edges[].source` | Always | The path of the source document. |
@@ -148,7 +148,7 @@
 | `graph.edges[].target.kind` | When `bound` is `document` | The kind of the target document. |
 | `graph.edges[].target.anchor_kind` | When `bound` is `anchor` or `withheld` | The anchor kind. |
 | `graph.edges[].target.resolver` | When `bound` is `anchor` | The resolver that bound the anchor. |
-| `graph.edges[].target.patterns` | When `bound` is `anchor` and the anchor holds more than one pattern | The sorted, normalized patterns, as on the anchor. |
+| `graph.edges[].target.patterns` | When `bound` is `anchor`. Before `1.4`, only when the anchor also holds more than one pattern | The sorted, normalized patterns, as on the anchor. |
 | `graph.edges[].target.rule` | When `bound` is `withheld` | The export profile whose filter withheld the target. |
 | `graph.edges[].target.reason` | When `bound` is `nothing` | Why nothing bound the target, as one English sentence. Do not parse it. |
 | `graph.edges[].attributes` | When the edge has instance attributes | The attributes as the author wrote them. The keys come from the corpus, so this table does not name them. |
@@ -158,13 +158,13 @@
 | `bound` | Members of `target` | What it means |
 |---|---|---|
 | `document` | `id`, `path`, `kind` | A typed document is the target. |
-| `anchor` | `anchor_kind`, `id`, `resolver`, and `patterns` for more than one pattern | An anchor is the target. |
+| `anchor` | `anchor_kind`, `id`, `resolver` and `patterns` | An anchor is the target. |
 | `withheld` | `anchor_kind`, `rule` | The source withheld the target under a declared export filter. This is not a defect. |
 | `nothing` | `reason` | Nothing bound the target. The edge travels, so that a reader does not conclude that every relation resolved. |
 
 ### Raising the version
 
-The current version is `1.3`. `version` and `export_version` carry this one value, from one constant in the engine. Read `version`. `export_version` stays because its removal is a major change, and that change costs every reader more than the duplicate costs.
+The current version is `1.4`. `version` and `export_version` carry this one value, from one constant in the engine. Read `version`. `export_version` stays because its removal is a major change, and that change costs every reader more than the duplicate costs.
 
 These rules decide how a change to the members moves the version. This section is the one statement of them.
 
@@ -172,7 +172,7 @@
 - **A member removed or renamed moves the major.** A change to the type or the meaning of a member also moves the major. So does a member that becomes conditional where it was always present, because a reader that relied on it then fails.
 - **A reader that meets a major above the one it understands stops with a message.** A reader that meets a different minor warns and continues.
 
-The history: `1.0` to `1.1` added `version` ([#343](https://github.com/headwater-ai/headwater/issues/343)). `1.1` to `1.2` added `patterns` to a list anchor and to an edge onto one ([#1247](https://github.com/headwater-ai/headwater/issues/1247)). `1.2` to `1.3` added `identifiers` to each `counted` tombstone ([#1309](https://github.com/headwater-ai/headwater/issues/1309)).
+The history: `1.0` to `1.1` added `version` ([#343](https://github.com/headwater-ai/headwater/issues/343)). `1.1` to `1.2` added `patterns` to a list anchor and to an edge onto one ([#1247](https://github.com/headwater-ai/headwater/issues/1247)). `1.2` to `1.3` added `identifiers` to each `counted` tombstone ([#1309](https://github.com/headwater-ai/headwater/issues/1309)). `1.3` to `1.4` wrote `patterns` for a single-pattern anchor too, so a reader reads one member for every anchor.
 
 ## See also
 
R2_REFERENCE
}

# Every anchor lists its patterns, and the version stays at `1.3`.
wrong() {
    reference && sed -i 's/^pub const VERSION: &str = "1.4";$/pub const VERSION: \&str = "1.3";/' engine/crates/generate/src/export.rs && grep -q '^pub const VERSION: &str = "1.3";$' engine/crates/generate/src/export.rs
}

# The version raised, and a single-pattern anchor still writes none.
version_alone() {
    sed -i 's/^pub const VERSION: &str = "1.3";$/pub const VERSION: \&str = "1.4";/' engine/crates/generate/src/export.rs && grep -q '^pub const VERSION: &str = "1.4";$' engine/crates/generate/src/export.rs
}
more_wrong="version_alone"
