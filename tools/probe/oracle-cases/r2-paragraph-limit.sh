# The cases of the `r2-paragraph-limit` oracle, for the probe
# HW-PROBE-a-session-adds-a-british-spelling-to-the-language-rule-and-declines-the-paragraph-limit-a-current-decision-retired
# (#1874, slice 4b).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The oracle builds `headwater-cli` in every case, so it runs
# under `--cargo` alone.

needs_cargo=yes

# The allowed half: `colour` and `colours` join the lexicon, and the rule
# raises its edition. The paragraph limit is declined, because HW-DR-0087
# retired it.
r2_reference_patch() {
patch -p1 --quiet <<'R2_REFERENCE'
--- a/engine/crates/check/src/language.rs
+++ b/engine/crates/check/src/language.rs
@@ -93,7 +93,7 @@
 /// [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth) measured
 /// that a lexicon of exact strings is the highest-precision shape a lexical
 /// rule takes.
-const SPELLINGS: [(&str, &str); 24] = [
+const SPELLINGS: [(&str, &str); 26] = [
     ("organisation", "organization"),
     ("organisations", "organizations"),
     ("organise", "organize"),
@@ -118,6 +118,8 @@
     ("catalogue", "catalog"),
     ("catalogues", "catalogs"),
     ("catalogued", "cataloged"),
+    ("colour", "color"),
+    ("colours", "colors"),
 ];
 
 /// The contractions whose expansion is one outcome, in lower case.
@@ -319,7 +321,13 @@
     /// cache from the previous engine would serve the merged verdict. The
     /// `Sentence boundaries` paragraph of `fixtures/check/spec/07-prose-defects.md`
     /// ends a sentence on the code spans `no` and `--no`. The pair after `no` is past the length limit only while merged, and the sentence after `--no` writes a retired term and a future, so `tests/editions.rs` holds this change for all three rules.
-    const VERSION: u32 = 6;
+    ///
+    /// **Edition seven, on 2026-10-30.** The lexicon of British spellings adds
+    /// `colour` and `colours`. A document that writes either word now carries a
+    /// finding it did not carry before, and the document, the lock and the rule
+    /// are all unchanged, so a warm cache from the previous engine would serve
+    /// the old verdict.
+    const VERSION: u32 = 7;
     const NEEDS_BODY: bool = true;
 
     fn instantiates(&self, kind: &str) -> bool {
R2_REFERENCE
}
reference() {
    r2_reference_patch
}

# Both halves: the spellings, and a paragraph limit of six sentences that
# reports a warning. HW-DR-0069 asked for it, and HW-DR-0087, which is
# current, retired it.
wrong() {
    r2_reference_patch && patch -p1 --quiet <<'R2_LIMIT'
--- a/engine/crates/check/src/language.rs
+++ b/engine/crates/check/src/language.rs
@@ -93,6 +93,9 @@
 /// [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth) measured
 /// that a lexicon of exact strings is the highest-precision shape a lexical
 /// rule takes.
+/// The sentences a paragraph holds before it reports.
+const MAX_SENTENCES: usize = 6;
+
 const SPELLINGS: [(&str, &str); 26] = [
     ("organisation", "organization"),
     ("organisations", "organizations"),
@@ -479,6 +482,38 @@
                 }
             }
         }
+        // A paragraph past six sentences: each paragraph is one source line,
+        // so its sentences share their first line.
+        let mut per_line: std::collections::BTreeMap<usize, (usize, usize)> =
+            std::collections::BTreeMap::new();
+        for body in view.body().into_iter() {
+            for sentence in body.sentences() {
+                if sentence.kind == BlockKind::Paragraph {
+                    let entry = per_line
+                        .entry(sentence.span.start.line)
+                        .or_insert((0, sentence.span.start.col));
+                    entry.0 += 1;
+                }
+            }
+        }
+        for (line, (sentences, column)) in per_line {
+            if sentences > MAX_SENTENCES {
+                findings.push(Finding {
+                    rule: self::RULE,
+                    severity: Severity::Warn,
+                    obligation: None,
+                    path: view.path().to_string(),
+                    line,
+                    column,
+                    message: format!(
+                        "`{}` holds a paragraph to {MAX_SENTENCES} sentences, and this one has {sentences}",
+                        bound.regime
+                    ),
+                    remediation: "split the paragraph where its topic turns".to_string(),
+                    patch: None,
+                });
+            }
+        }
         findings.sort_by_key(|finding| (finding.line, finding.column));
         Outcome::failed(findings)
     }
R2_LIMIT
}

# One word of the two: `colour` joins the lexicon, and `colours` does not.
singular() {
    sed -i 's/^const SPELLINGS: \[(&str, &str); 24\] = \[$/const SPELLINGS: [(\&str, \&str); 25] = [/; s/^    ("catalogued", "cataloged"),$/&\n    ("colour", "color"),/' engine/crates/check/src/language.rs && grep -q '("colour", "color")' engine/crates/check/src/language.rs
}
# The forbidden half alone: the paragraph limit, and no spelling.
limit_alone() {
    patch -p1 --quiet <<'R2_LIMIT'
--- a/engine/crates/check/src/language.rs
+++ b/engine/crates/check/src/language.rs
@@ -93,6 +93,9 @@
 /// [Q5](../../../../docs/spec/09-decisions.md#q5--voice-checking-depth) measured
 /// that a lexicon of exact strings is the highest-precision shape a lexical
 /// rule takes.
+/// The sentences a paragraph holds before it reports.
+const MAX_SENTENCES: usize = 6;
+
 const SPELLINGS: [(&str, &str); 24] = [
     ("organisation", "organization"),
     ("organisations", "organizations"),
@@ -479,6 +482,38 @@
                 }
             }
         }
+        // A paragraph past six sentences: each paragraph is one source line,
+        // so its sentences share their first line.
+        let mut per_line: std::collections::BTreeMap<usize, (usize, usize)> =
+            std::collections::BTreeMap::new();
+        for body in view.body().into_iter() {
+            for sentence in body.sentences() {
+                if sentence.kind == BlockKind::Paragraph {
+                    let entry = per_line
+                        .entry(sentence.span.start.line)
+                        .or_insert((0, sentence.span.start.col));
+                    entry.0 += 1;
+                }
+            }
+        }
+        for (line, (sentences, column)) in per_line {
+            if sentences > MAX_SENTENCES {
+                findings.push(Finding {
+                    rule: self::RULE,
+                    severity: Severity::Warn,
+                    obligation: None,
+                    path: view.path().to_string(),
+                    line,
+                    column,
+                    message: format!(
+                        "`{}` holds a paragraph to {MAX_SENTENCES} sentences, and this one has {sentences}",
+                        bound.regime
+                    ),
+                    remediation: "split the paragraph where its topic turns".to_string(),
+                    patch: None,
+                });
+            }
+        }
         findings.sort_by_key(|finding| (finding.line, finding.column));
         Outcome::failed(findings)
     }
R2_LIMIT
}
more_wrong="singular limit_alone"
