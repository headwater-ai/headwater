# The cases of the `r2-numbered-heading` oracle, for the probe
# HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading
# (#1874, slice 4b).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The oracle builds `headwater-cli` in every case, so it runs
# under `--cargo` alone.

needs_cargo=yes

# The sound change: a leading number of one word of digits, with or without
# a period, is not part of a section name, and the rule raises its edition
# to 2.
reference() {
patch -p1 --quiet <<'R2_REFERENCE'
--- a/engine/crates/check/src/sections.rs
+++ b/engine/crates/check/src/sections.rs
@@ -74,8 +74,11 @@
 
 impl DocumentCheck for Sections {
     const RULE: &'static str = self::RULE;
-    /// The first edition of this rule.
-    const VERSION: u32 = 1;
+    /// **Edition two, on 2026-10-30.** A heading's leading number is no longer
+    /// part of its section name, so `2. Decision` now satisfies a contract that
+    /// names `Decision`. The document, the lock and the rule are all unchanged,
+    /// so a warm cache from the previous engine would serve the old verdict.
+    const VERSION: u32 = 2;
     /// The headings are in the body, so the body is declared.
     const NEEDS_BODY: bool = true;
 
@@ -156,10 +159,26 @@
 
 /// A heading, reduced to what a contract names.
 ///
-/// Case and surrounding space are not part of a section name. Everything else
-/// is: a contract that names `Decision` is not satisfied by `Decisions`, and a
-/// rule that stemmed the word would decide for an author what their heading
+/// Case, surrounding space and a leading number are not part of a section
+/// name: `2. Decision` and `2 Decision` name `Decision`. The number is one word
+/// of digits, with or without a period after it, and only when a word follows
+/// it, so a heading that is only `2` stays `2`. Everything else is part of the
+/// name: a contract that names `Decision` is not satisfied by `Decisions`, and
+/// a rule that stemmed the word would decide for an author what their heading
 /// means.
 fn normalized(text: &str) -> String {
-    text.trim().to_lowercase()
+    let text = text.trim();
+    let unnumbered = match text.split_once(char::is_whitespace) {
+        Some((first, rest))
+            if !rest.trim().is_empty()
+                && {
+                    let digits = first.strip_suffix('.').unwrap_or(first);
+                    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
+                } =>
+        {
+            rest.trim()
+        }
+        _ => text,
+    };
+    unnumbered.to_lowercase()
 }
R2_REFERENCE
}

# The rule stated on the subsystem page, and no code moved. This case runs
# straight after the reference, and it leaves every file under
# `engine/crates` as old as the grading tree. So a build that trusts the
# times of those files serves the reference's build to it, and the case
# passes. `test-passes.sh` marks the files newer before it builds, and this
# case is what fails without that (#1874, slice 4b).
wrong() {
    printf '\n%s\n' 'The required-sections rule reads `2. Decision` as the section `Decision`.' >> docs/subsystems/checks-and-cache.md
}

# The edition raised, and the heading still read whole.
version_alone() {
    sed -i 's/^    const VERSION: u32 = 1;$/    const VERSION: u32 = 2;/' engine/crates/check/src/sections.rs && grep -q '^    const VERSION: u32 = 2;$' engine/crates/check/src/sections.rs
}

# The heading read without its number, and the edition left at 1, so a warm
# cache serves the old verdict.
number_alone() {
    reference && sed -i 's/^    const VERSION: u32 = 2;$/    const VERSION: u32 = 1;/' engine/crates/check/src/sections.rs && grep -q '^    const VERSION: u32 = 1;$' engine/crates/check/src/sections.rs
}
more_wrong="version_alone number_alone"
