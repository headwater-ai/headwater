---
id: HW-PROBE-a-session-repairs-a-change-narration-finding-in-a-sentence-about-another-tool-and-raises-no-other-finding
status: current
status_since: 2026-10-30
summary: "A session rewrites the sentence of HW-DR-0012 on a suppression that matches nothing so the change-narration finding clears, and raises no finding the pin did not carry."
last_verified: 2026-10-30
probe_category: sufficiency
expectation: patched
oracle: "command:r9-eslint"
title: "A session repairs a change-narration finding in a sentence about another tool and raises no other finding"
action: repair_check
moments: [review]
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: unevidenced
relations:
  traces_to:
    - HW-DR-0012
---

# A session repairs a change-narration finding in a sentence about another tool and raises no other finding

## Task

### Phrasing 1

`headwater check` reports change narration in the decision on the migration path for an existing corpus, in a sentence about ESLint. Repair the sentence so the finding clears, and raise no other finding.

### Phrasing 2

One sentence of the decision about adopting Headwater on an existing corpus writes "no longer" about an ESLint suppression, and the voice rule reports it. Rewrite it so that the check is clean of it and of anything new.

## Target

[HW-DR-0012](../decisions/0012-migration-path-for-an-existing-corpus.md), whose paragraph "Every run reports how many pairs remain" holds the sentence.

## Key

`patched` over `command:r9-eslint`, the `exit` oracle that `.headwater/probe.yml` declares under `oracles:`. Its command runs `tools/probe/oracles/nothing-else-broken.sh --check-clean` with a held file under `tools/probe/oracle-args/` that names the baseline. The baseline names this probe, scopes the oracle to the target's file, and holds the findings of that file in the sealed grading tree less the one the task asks to repair. The oracle passes when the file carries no finding, of any severity, that the baseline does not, and no strict error. So the tree as it stands fails on the one finding the baseline lacks.

**The reference change and the wrong change, checked.** `tools/probe/oracle-cases/r9-eslint.sh` holds both, and `tools/probe/probe-oracle-fixtures.sh` runs them in a copy of the sealed grading tree. The reference change rewrites "a suppression which no longer matches" as "a suppression that matches nothing". It clears the finding, raises none, and passes. The wrong change rewrites the clause into one sentence of more than 25 words. The change-narration finding clears and a sentence-length finding is raised, so it fails. The grading tree as it stands fails.

**The axes.** No phrasing names a file, so the probe does not carry `names_file`. No superseded or contradicted record stands in the way, so the probe does not carry `superseded_target`.

**The baseline moves with its file.** The baseline holds the findings of one file in the sealed grading tree. A change to that file on `main`, or a seal that removes a document it links, moves them, and `tools/probe/probe-oracle-fixtures.sh` then fails this probe's reference change. Re-record it with `sh tools/probe/oracles/nothing-else-broken.sh --record <baseline>` in that tree, and delete the line of the finding the task repairs.
