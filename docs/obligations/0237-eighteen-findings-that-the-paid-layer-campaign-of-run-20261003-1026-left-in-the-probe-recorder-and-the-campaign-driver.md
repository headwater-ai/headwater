---
id: HW-OBL-0237
status: current
status_since: 2026-10-09
summary: "The campaign of #1659 met gaps in tools/probe: refusals that share an exit code, a sentence a wrapper must match, ungoverned scripts and surviving mutants. None has a reader outside this repository, and #1700 meets them first."
last_verified: 2026-10-09
title: "Eighteen findings that the paid layer campaign of run 20261003-1026 left in the probe recorder and the campaign driver"
waiting_on: build
provenance:
  warrant: asserted
---

# Eighteen findings that the paid layer campaign of run 20261003-1026 left in the probe recorder and the campaign driver

## Context

The builders, verifiers and maintainers of #1474, #1472 and #1659 in run `20261003-1026` wrote these findings as intake lines. Each one is about the probe recorder, the campaign driver or the records they write. None names a reader outside this repository, so the product owner ruled each one RECORD at the end-of-run pass on 2026-10-09. The intake lines are 8 to 10, 21, 22, 30 to 43, 48, 51 and 54 to 57. Lines that state one finding twice are one item here.

Each item was checked on `66dec0f9` at the file and the symbol it names. The surviving mutants are as the verifiers reported them, and the product owner checked that the code each one names is still there. Intake line 49 is an item of [HW-OBL-0234](0234-thirteen-test-gaps-that-the-verifiers-of-run-20261002-1233-left-in-the-recorder-the-probe-tools-and-the-spec-guards.md), so this record does not repeat it. The next campaign is #1700, and it meets every item below before it spends.

## Obligation

**What the recorder writes.**

- A session that writes auto-memory files under its configuration directory has those paths listed as produced artifacts. `step_seed_produced` in `tools/probe/probe-transform.sh` reads every write call of the log and does not exclude the configuration directory. The paths carry no `findings` key, so the grader cannot grade the session. The evaluation of the campaign states that one session, `L7-documentation-absent-p3-r4`, was discarded and drawn again for this reason.
- A transcript under `docs/probe-runs/` does not say why a repetition is missing. A job that a ceiling refused and a session that ended unrecorded leave the same gap. A recorder line that lists the refused and the unrecorded jobs would explain each gap. Lines 5, 7, 14 and 18 of the campaign have such gaps.
- `tools/probe/probe-record.sh` states the count of refused provider fetches only in a sentence of `record.md`, "It refused %s of that kind". `clean` in `tools/probe/campaign-drive.sh` matches that sentence, so a change of wording holds the parallel ramp. A `provider_refusals:` field would remove the match.
- `tools/probe/probe-transform.sh` refuses a stream that does not parse with exit 4, so such a session never records. The "were not counted" sentences of `tools/probe/probe-record.sh` are then reached at status 0 only when two `jq -s` reads disagree.

**What the driver does.**

- `run_job` and the end of `tools/probe/campaign.sh` exit 7 both when a cap or a ceiling refused a job and when a recorder failed. `tools/probe/campaign-drive.sh` must tell the two apart from `slice/refused/` and `slice/started/`. A distinct exit code for a batch that ended on refusals alone would let a wrapper read the code.
- A resumed batch accepts a `--repetitions` that differs from the count its job list was made at. A batch of 354 jobs at 177 repetitions resumed with `--repetitions 30` rewrites `<out>/repetitions` to 30 and keeps 354 jobs.
- When a batch ends capped, `tools/probe/campaign-drive.sh` exits 8 and leaves `a.done` unwritten, so a relaunch enters batch A again. No verb marks a capped batch accepted. In #1659 a person wrote `a.done` by hand.
- `build_engine` in `tools/probe/campaign.sh` runs a plain `cargo build`, not `tools/hw-cargo`. A batch started without `CARGO_TARGET_DIR` builds an unthrottled target under `engine/target`.
- `campaign.sh --dry-run` prices a pooled line at the power calculation. For the regression tier of #1474 it printed 354 sessions at 88.50 dollars, while the run planned the tier's one repetition, 8 sessions in all.

**What no record states.**

- A batch may raise `--repetitions` above a tier's count only up to the count the dry run priced. Only the header of `tools/probe/campaign.sh` states this rule. It rests on the parent's ruling to merge PR #1685 and on the owner's approval of 2,142 dollars. No decision or obligation record under `docs/` states it.
- The campaign ceiling moved from 30000 to 205200 cents. Only the comment above the campaign tier in `.headwater/probe.yml` states the move and its reason. [HW-DR-0076](../decisions/0076-a-probe-budget-prices-a-run-identity-fixed-before-the-run-and-a-committed-transcript-and-a-sweep-has-neither.md) states no figure.
- Before slice 1 of #1659, `tools/probe/campaign.sh` built no tree for a component arm. The arms `no-hook`, `no-skills`, `no-claude-md` and `mcp` ran the present tree while each transcript said the arm's change was applied. No committed result is affected, because none of those arms had a committed run, and no record states the defect.
- None of the 8 regression probes declares a string under `leaks:` in `.headwater/probe.yml`. So `seal.sh --leak` prints `undeclared` for each and can find no leak in any of them.

**Tests that hold less than the code does.**

- Mutant E of `tools/probe/campaign.sh` survives. Removing the `--repetitions` check leaves `tools/probe/probe-record-fixtures.sh` green, though `0`, `010`, `-1`, `abc` and `1.5` exit 2 by hand. The same gap in `tools/probe/campaign-dry-run.sh` is inferred and was not run.
- Four mutants of `tools/probe/campaign-drive.sh` have no case. N19 ignores what was spent before on a shared cap. N16 writes the canary range high before low. N1 refuses a session at exactly the canary bound. N13 caps a retry that recorded one new session.
- `tools/probe/campaign-drive.sh` has no case against four things: the real `tools/probe/campaign.sh`, a live `record.md` with refusals, the overshoot at a bound of 300, and BSD `date`. Its header says exit 5 means only that `campaign.sh` refused the plan. `campaign.sh` also exits 5 when `--repetitions` is above the priced count.
- Mutants D2 and U1 of `tools/probe/campaign.sh` have no case. D2 points the dirty-tree check at the batch's output directory and not at the repository root, and the suite's stub answers for any directory. U1 makes `--unrecorded` accept any status but 0. A case with a status-10 session and a cost file should exit 2.

**What governs the tools.**

- No document declares `governs` onto `tools/probe/campaign-drive.sh`, `tools/probe/campaign-drive-fixtures.sh`, `tools/probe/probe-transform.sh` or `tools/engine/build-declaration-fixtures.sh`. [Spec 15](../spec/15-the-recorder-contract.md) governs `tools/probe/probe-record.sh` and `tools/probe/egress-proxy.py` only. [HW-OBL-0235](0235-eleven-documents-and-edges-that-trail-a-change-of-run-20261002-1233.md) lists `campaign.sh`, `campaign-dry-run.sh` and the other probe files of the run before, so the two records together name the ungoverned set. The adjudication of slice 1c of #1659 ruled that the edge waits on a probe how-to or on the next document that describes the driver.

## Discharge

Each item discharges alone. A recorder or driver item discharges when the change merges with a case that fails without it. A record item discharges when a decision or obligation record states the rule or the defect. The governance item discharges when a document declares `governs` onto each file it names. The record discharges when every item has. An item that gains a reader outside this repository leaves this record for an issue, and the record says where it went.
