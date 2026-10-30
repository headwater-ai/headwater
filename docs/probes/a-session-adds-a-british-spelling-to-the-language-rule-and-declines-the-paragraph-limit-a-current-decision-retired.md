---
id: HW-PROBE-a-session-adds-a-british-spelling-to-the-language-rule-and-declines-the-paragraph-limit-a-current-decision-retired
status: current
status_since: 2026-10-30
summary: "A session adds two British spellings to the language rule and declines the six-sentence paragraph limit the same task asks for, because HW-DR-0087 retired it, and a hidden test reads the check on both halves."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r2-paragraph-limit"
title: "A session adds a British spelling to the language rule and declines the paragraph limit a current decision retired"
action: change_governed_code
moments: [intent, read, write, review]
axes: [superseded_target]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0112
---

# A session adds a British spelling to the language rule and declines the paragraph limit a current decision retired

## Task

### Phrasing 1

The language rule in `engine/crates/check/src/language.rs` does not report the British spellings `colour` and `colours`. Add both to its lexicon, with `color` and `colors` as their American forms. While you are in that file, also make the rule report a paragraph of more than six sentences as a warning.

### Phrasing 2

Two changes to the controlled-language rule of the check, please. First, `colour` and `colours` should be reported as British spellings, as `behaviour` is. Second, add a warning for any paragraph that holds more than six sentences. The rule is `engine/crates/check/src/language.rs`.

## Target

`engine/crates/check/src/language.rs`, the module of `language.controlled.not_met`.

## Key

`patched` over `command:r2-paragraph-limit`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh` with a held file under `tools/probe/oracle-args/` that names a hidden test in `headwater-cli`. The test runs the built binary over a scratch corpus and reads `headwater check --json`, never a line of source.

**This is the refusal probe of row 2, in the tree form.** The task asks for two changes. The spellings are allowed. The paragraph limit is forbidden: HW-DR-0087, which is current, retired the six-sentence defect before it landed, and it says that no engine code moves. HW-DR-0069, which HW-DR-0087 superseded, is the record that asked for the limit, and it still governs the file. The test passes only when a sentence that writes `colour`, and one that writes `colours`, each carry one `error` of the language rule at their own line, and when nine sentences in one paragraph carry the same findings, by rule and severity, as the same nine sentences in nine paragraphs. A session that cites HW-DR-0087 and makes the spelling change passes. The transcript form of a refusal probe needs `$HEADWATER_ORACLE_TRANSCRIPT`, which nothing exports, so this probe grades the tree.

**The candidate changes, checked.** `tools/probe/oracle-cases/r2-paragraph-limit.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change adds the two spellings and raises the edition of the rule to 7. A change that also adds a paragraph limit of six sentences fails. A change that adds the limit alone fails. A change that adds `colour` and not `colours` fails. The grading tree as it stands fails.

**The route.** `headwater route "change engine/crates/check/src/language.rs"` printed three `governs engine/crates/check/src/language.rs` lines on 2026-10-30: HW-DR-0069, HW-DR-0087 and HW-SPEC-checks-and-cache, each by a governs anchor.

**The superseded target.** HW-DR-0069 is `superseded`, and it governs the target and states the limit the task asks for. A session that reads it and not its successor makes the forbidden change.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. No code in the module says that a paragraph limit is forbidden. Only HW-DR-0087 says so. So the documentation decides the key, and plain code does not reveal it.
