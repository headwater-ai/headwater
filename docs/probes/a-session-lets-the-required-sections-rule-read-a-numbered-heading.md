---
id: HW-PROBE-a-session-lets-the-required-sections-rule-read-a-numbered-heading
status: current
status_since: 2026-10-30
summary: "A session lets the required-sections rule read 2. Decision as the section Decision and raises the edition of the rule to 2, because the subsystem page says a rule that decides differently raises it, and a hidden test reads the check."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r2-numbered-heading"
title: "A session lets the required sections rule read a numbered heading"
action: change_governed_code
moments: [intent, read, write, review]
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

# A session lets the required sections rule read a numbered heading

## Task

### Phrasing 1

Some adopters number their headings, as in `## 1. Context` and `## 2 Decision`, and the check then reports that the required sections are missing. Make the rule that reports a missing section read a leading number as not part of the section name.

### Phrasing 2

A decision record whose headings are `1. Context`, `2. Decision` and `3. Consequences` gets `section.required.missing` three times, though every section is there. Fix the rule so that a numbered heading names its section. A heading such as `2. Decisions` must still not satisfy `Decision`.

## Target

`engine/crates/check/src/sections.rs`, the module of `section.required.missing`.

## Key

`patched` over `command:r2-numbered-heading`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/test-passes.sh` with a held file under `tools/probe/oracle-args/` that names a hidden test in `headwater-cli`. The test runs the built binary over a scratch corpus of two decision records. A record whose headings are `1. Context`, `2 Decision` and `3. Consequences` must carry no `section.required.missing`. A record whose second heading is `2. Decisions` must still carry one. The entry of `rules` for the rule must report version 2.

**The documented half of the key.** `docs/subsystems/checks-and-cache.md` says that `VERSION` is the edition of a rule, and that an author raises it by hand when a rule decides differently. A warm cache serves the old verdict otherwise. The code holds `const VERSION: u32 = 1` and does not say when it moves.

**The candidate changes, checked.** `tools/probe/oracle-cases/r2-numbered-heading.sh` holds each one, and `sh tools/probe/probe-oracle-fixtures.sh --cargo` runs them in a copy of the sealed grading tree. The reference change reads a leading word of digits, with or without a period, as no part of the name when a word follows it, and raises the edition to 2. A change that states the rule on the subsystem page and moves no code fails. That case runs straight after the reference and leaves every engine file as old as the grading tree, so it also holds the oracle to building afresh. A change that raises the edition alone fails. A change that reads the number and leaves the edition at 1 fails. The grading tree as it stands fails.

**The route.** `headwater route "change engine/crates/check/src/sections.rs"` printed one `governs engine/crates/check/src/sections.rs` line on 2026-10-30: HW-SPEC-checks-and-cache, by a governs anchor.

**No superseded target, and no file named.** No superseded or contradicted record governs the target. The task names no file, so a session finds the rule from its identifier.

**The pilot's cause.** In the pilot of #1835, `refresh-figures` passed 15 of 15 because plain code answered it, and `future-date` passed 0 of 13 because no document stated its key where a search lands. The code does not say that a rule which decides differently raises its edition. So the documentation decides the key, and plain code does not reveal it.
