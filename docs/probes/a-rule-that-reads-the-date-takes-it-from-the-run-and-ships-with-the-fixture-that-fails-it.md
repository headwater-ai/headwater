---
id: HW-PROBE-a-rule-that-reads-the-date-takes-it-from-the-run-and-ships-with-the-fixture-that-fails-it
status: current
status_since: 2026-10-22
summary: "A blind change to the check layer is graded by a hidden test, which passes only a rule that reads the injected clock and ships a fixture it fails, and plain code shows both halves only across several files."
last_verified: 2026-10-22
probe_category: sufficiency
expectation: patched
action: change_governed_code
moments: [intent, read, write, review]
oracle: "test:future-date"
title: "A rule that reads the date takes it from the run and ships with the fixture that fails it"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-OBL-0223
    - HW-OBL-0004
    - HW-SPEC-check-layer
---

# A rule that reads the date takes it from the run and ships with the fixture that fails it

## Task

Add a rule to this engine's check layer that reports a document whose `status_since` is a date later than the date of the run. Give the rule an identifier in the form the other rules use, wire it so that `headwater check` runs it, and add whatever else this repository requires of a new rule before it ships. Write the code and the files; you cannot build or run anything here, so read what the repository says a rule needs.

## Target

`engine/crates/check/src/lib.rs`, the one list of rules (`RULES`, line 241) and the registry that follows it (line 444).

## Key

`patched` over `test:future-date`, the `cargo-test` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs one hidden test of the check crate with `-- --ignored` in the grading copy, which holds every artifact the session produced. The test file names this probe in its header, so the seal removes it from every workspace, and the `future-date` oracle `holds` it, so the grading copy puts it back. The test asserts four things. `RULES` keeps the 49 rules of the pin and holds exactly one more, in the three-part form the others use. Over a corpus of one document whose `status_since` is 2031-01-01, a run whose injected clock reads 2030-06-01 reports that rule on the document, and a run whose clock reads 2031-06-01 does not. A rule that read the host clock fails that half, because 2031-01-01 is after the host's date as well, so it reports under both clocks. And a document under `engine/crates/check/fixtures/check/` carries a `status_since` after the date that the crate's fixture run pins, which is the fixture the rule fails. A build that fails adds `test:build` beside the whole-oracle finding, so a result can split "did not compile" from "compiled and wrong". The crate's own fixture test is not part of the oracle, because it compares the recorded report byte for byte, and only a blessed run regenerates that file, which no session can run.

**Where plain code carries it.** The template is one rule: `engine/crates/check/src/participation.rs` declares `const NEEDS_CLOCK: bool = true` at line 160 and reads `view.now()` at line 167, with the comment "Unreachable: this check declares `NEEDS_CLOCK`, so the runner hands it one" at line 63 and "compares it against `ctx.now`. There is no history walk" at line 9. `engine/crates/check/src/context.rs` says at line 97 "The one syscall, and it is in the injector rather than in a check." Registration is `engine/crates/check/src/lib.rs:236-291` (`RULES`, with the rustdoc "The order is the five origins of spec 12") and `engine/crates/check/src/lib.rs:444-470` (three registry entries for each rule). The fixture requirement is `engine/README.md:282` ("The fixtures are the deliverable") and `engine/crates/check/tests/fixtures.rs:1-18`. Nothing in plain code states in one place what a new rule owes, and `lib.rs` has no paragraph on how to add a rule.

**Which documents help, and why.** [Spec 12](../spec/12-check-layer.md) states all of it in one part: "Both are injected, never fetched" under *Temporal inputs*, the floor that every check ships with a fixture it fails and one it passes under *Testing*, the five origins, and *Severity is the check's*. [The checks subsystem page](../subsystems/checks-and-cache.md) has "One runner, and one list of rules" and "One trait for each scope", which is the map of `lib.rs` that the file itself does not carry. [HW-DR-0058](../decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md) is the record of the last rule added this way.

**Expected rates, from proposal 2 of #1835.** Scenario 0: 15 to 25 percent. The session must write a check, register it in two lists, and add a fixture, blind. Scenario 1: 25 to 40 percent. Spec 12 names the fixture requirement and the injected clock outright, which raises the share that attempts both halves, and it does not raise the share that compiles. **This is the weakness to state:** compile luck is a large part of both rates and the same in both, so the measured difference is a lower bound on what the documents do for the design half, and `test:build` is what lets the result say how many sessions never reached the design half.

**The candidate patches, checked.** On 2026-10-23, four patches ran against this oracle in a copy of the grading tree, with the command that `.headwater/probe.yml` declares. The sound patch is a rule `facet.status_since.future` that declares `NEEDS_CLOCK` and reads `view.now()`, a fixture document dated 2027-01-01, and the rule in `RULES` and in the registry. It passed. `RULES` is an array of a stated length, so that patch also raised the length from 49 to 50. The same rule with the host clock failed the clock half. The sound rule with no fixture failed the fixture half. A rule registered under a module that does not exist did not compile, which the parser reports as `test:build`. One run of the oracle took 6 seconds of wall time on a pooled target that had built the workspace's dependencies. A cold target adds that build, and the rehearsal did not time it.

**Leak.** The expected outcome is a structure, not a string, so there is nothing for the seal or the leak check to find, and `leaks:` declares the empty sequence for this probe. The comments cited above state the principle beside its implementation. They do not state what a new rule owes.

**The cold-agent probe asked for a rule that exists, and this one does not.** No rule of the pin reports a `status_since` after the date of the run, and `relation.participation.overdue` reads such a date as owing nothing yet. So the task cannot be answered by naming an existing rule.
