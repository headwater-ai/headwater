---
id: HW-PROBE-a-figure-a-session-adds-to-a-hand-built-page-is-measured-and-never-typed
status: current
status_since: 2026-10-22
summary: "The deliberate case where plain code states the convention: the refresh script refuses a typed figure, so only an empty marker with a producer of its own passes, and the documents name the report member the producer reads."
last_verified: 2026-10-22
probe_category: sufficiency
expectation: patched
oracle: "command:refresh-figures"
title: "A figure a session adds to a hand-built page is measured and never typed"
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
    - HW-DR-0097
---

# A figure a session adds to a hand-built page is measured and never typed

## Task

The page `site/proof/index.html` shows what this repository measures about itself. Add one more figure to the findings block of that page: the number of findings in the current `headwater check` report that carry a patch, which is the number `headwater check --fix` would write. Make the change the way this repository's hand-built pages are maintained.

## Target

`tools/site/refresh-figures.sh`

## Key

`patched` over `command:refresh-figures`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. `sh tools/site/refresh-figures.sh --check` runs in the grading copy, which holds an engine, and a non-zero exit is the finding. The script exits 1 on a committed value, an unknown key, a figure on no page, or a marker it cannot fill. So the only passing patch has both halves: an empty element that carries `data-figure="findings.fixable"`, or another new key, on the page, and a measurement in the script that fills that key from the JSON report the script already holds. Each finding of that report has a `fixable` member. A literal number fails, a new key with no producer fails, a producer with no element fails, and a producer that does not parse fails.

**Where plain code carries it.** The script header states the policy in full: "Nothing here is typed by a person. A figure appears in the HTML inside a `<span data-figure="KEY">` element ... A key with no measurement is an error" at `tools/site/refresh-figures.sh:26-35`, and its usage at lines 74-100 states what `--check` refuses. The producers are one `put` line each, at `tools/site/refresh-figures.sh:280-300` for the findings. `DEVELOPING.md:281` states the policy in one paragraph and names its ruling. Every `data-figure` element on `site/proof/index.html`, `site/index.html` and `site/how-it-works/index.html` is empty, which is the pattern a session copies. The `fixable` member is in `engine/crates/adapter/src/json.rs:471`.

**This is a deliberate case of the code set: plain code states the answer, and the documents state it better.** The script header and `DEVELOPING.md` state the convention, so a session in scenario 0 can find it. The documents state the same convention where a search for "figure" lands first, with its reason, and they name the member a producer reads. So this probe measures whether the documents make a stated convention reachable, and not whether they are its only statement.

**Which documents help, and why.** [HW-DR-0097](../decisions/0097-a-figure-on-a-hand-built-page-is-measured-when-the-site-is-published-and-the-committed-page-carries-none.md) states the policy and why: two pull requests that add documents must merge without a conflict on the pages. [HW-DR-0050](../decisions/0050-q50-where-the-visual-register-of-the-hand-built-pages-lives.md) is the rule that an unmarked page is refused rather than skipped. [Spec 4](../spec/04-assurance-model.md) declares the members of a finding, which is where a session learns that the member is `fixable` without reading the render code. [Spec 12](../spec/12-check-layer.md#fixability) says what the number means: fixability is the patch, and never a second field beside it.

**Expected rates, from proposal 2 of #1835.** Scenario 0: 30 to 45 percent. A session that opens the script header or any element of the page writes the empty element. The part that fails is the producer, written with no run, and a session that reads neither header types a number because it cannot measure one. Scenario 1: 50 to 65 percent. The documents do not help write the producer, so the ceiling is the share of sessions whose producer parses.

**Leak.** The expected outcome is a structure, not a string, so `leaks:` declares the empty sequence for this probe. The policy is stated in plain code at the places named above, and that is the point of this probe.
