# The cases of the `r2-check-counts` oracle, for the probe
# HW-PROBE-a-session-adds-a-count-of-findings-by-severity-to-the-json-report-of-the-check
# (#1874, slice 4b).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The oracle builds `headwater-cli` in every case, so it runs
# under `--cargo` alone.

needs_cargo=yes

# The sound change: a `counts` member that counts the entries of `findings`
# by severity, the shape version raised to `1.7`, and the interface page
# moved with it.
reference() {
patch -p1 --quiet <<'R2_REFERENCE'
--- a/engine/crates/adapter/src/json.rs
+++ b/engine/crates/adapter/src/json.rs
@@ -106,7 +106,10 @@
 /// `docs/interfaces/headwater-check.md` states this history for a reader who
 /// holds no clone, and `tests/interface_page.rs` fails when this constant
 /// names a version the page does not (#1451). Raise both together.
-pub const VERSION: &str = "1.6";
+///
+/// `1.7` adds `counts`: the entries of `findings`, counted by severity, as an
+/// object with `error`, `warn` and `info`.
+pub const VERSION: &str = "1.7";
 
 /// What a run carries that these bytes do not write.
 ///
@@ -391,12 +394,29 @@
     if let Some(scoped) = &run.change {
         members.push(("change", change(scoped)));
     }
+    let listed = reported(run);
+    // The entries of `findings` below, counted by severity, so a reader can
+    // gate on a count without walking the list (1.7).
+    let counted = |word: &str| {
+        number(
+            listed
+                .iter()
+                .filter(|entry| crate::severity(entry.finding.severity) == word)
+                .count(),
+        )
+    };
+    let counts = Json::object([
+        ("error", counted("error")),
+        ("warn", counted("warn")),
+        ("info", counted("info")),
+    ]);
     members.extend([
         ("coverage", coverage(&run.coverage, &run.instances)),
         ("rules", Json::Array(run.served.iter().map(rule).collect())),
+        ("counts", counts),
         (
             "findings",
-            Json::Array(reported(run).iter().map(finding).collect()),
+            Json::Array(listed.iter().map(finding).collect()),
         ),
         // The read set, in full, with a digest per input. Checks and cache asks a
         // run to report it beside the findings, and the reader that needs it is
--- a/docs/interfaces/headwater-check.md
+++ b/docs/interfaces/headwater-check.md
@@ -182,12 +182,13 @@
 | `rules[].version` | Always | The version of the check, as a number. |
 | `rules[].obligations` | Always | The identifiers of the obligations that the rule serves. The array is empty when the rule names none. |
 | `rules[].compared` | Only on the entries for `link.identifier.mismatch` and `link.fragment.unresolved` | The number of links that the rule compared, whether they pass or not. `"compared": 0` says that the rule examined nothing. It is not written on the entry of any other rule. |
+| `counts` | Always | The entries of `findings`, counted by severity, as an object with `error`, `warn` and `info`. Each count is a number, and a severity with no finding counts `0`. |
 | `findings` | Always | One entry for each finding of the report, in the shape that spec 4 declares. |
 | `read_set` | Always | Each input that the run read, with the digest of its content. An input with no digest carries an empty array in place of the digest. |
 
 ### Raising the version
 
-The current version is `1.6`. The version is a constant of the shape, and it is not the version of the engine. Two engines that write one shape write one version, so a reader does not read the shape again. The SARIF report carries `change` and `coverage` in its property bag, so this version also applies to those two members of that report.
+The current version is `1.7`. The version is a constant of the shape, and it is not the version of the engine. Two engines that write one shape write one version, so a reader does not read the shape again. The SARIF report carries `change` and `coverage` in its property bag, so this version also applies to those two members of that report.
 
 Each version added a member, and each addition moved the minor. So an absent member says a different thing at each version. At `1.1` a report with no `change` says that the run read the whole corpus. At `1.0` it says only that the engine had no such member to write.
 
@@ -198,6 +199,7 @@
 - `1.4` added `verified` and `verified_alone` to `change`. Before `1.4`, a document that only a `verified` line named was in `documents` and in no other count, so the counts did not sum ([#1398](https://github.com/headwater-ai/headwater/issues/1398)).
 - `1.5` added `compared` to the entries of `rules` for `link.identifier.mismatch` and `link.fragment.unresolved`. A rule that reports nothing is silent over links that all pass. It is also silent over a corpus with none of its links, and `compared` tells the two apart ([#1347](https://github.com/headwater-ai/headwater/issues/1347)).
 - `1.6` added `unstated` to `change`. Before `1.6`, a `verified` line whose target its document does not govern was in no member. A line whose document no row holds was in `unmatched` only as its document path ([#1631](https://github.com/headwater-ai/headwater/issues/1631)).
+- `1.7` added `counts`, the entries of `findings` counted by severity. Before `1.7`, a reader counted the findings of each severity by walking the list.
 
 ## See also
 
R2_REFERENCE
}

# The member, and no new version: the page under "Raising the version" says
# that an added member moves the minor.
wrong() {
    reference && sed -i 's/^pub const VERSION: &str = "1.7";$/pub const VERSION: \&str = "1.6";/' engine/crates/adapter/src/json.rs && grep -q '^pub const VERSION: &str = "1.6";$' engine/crates/adapter/src/json.rs
}

# The version raised, and no member written.
version_alone() {
    sed -i 's/^pub const VERSION: &str = "1.6";$/pub const VERSION: \&str = "1.7";/' engine/crates/adapter/src/json.rs && sed -i 's/^The current version is `1.6`\./The current version is `1.7`./' docs/interfaces/headwater-check.md && grep -q '^pub const VERSION: &str = "1.7";$' engine/crates/adapter/src/json.rs
}
more_wrong="version_alone"
