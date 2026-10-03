---
id: HW-RUN-regression-of-2026-10-03-regression-tier-present-arm-navigability
status: current
status_since: 2026-10-03
summary: "The navigability line of the sealed re-recording of the three retired regression runs, on claude-sonnet-5 over three probes: 3 sessions, 72 cents, 0 web-tool calls and 0 calls naming a probe shelf."
last_verified: 2026-10-03
tier: regression
arm: present
title: "Regression of 2026-10-03, regression tier, present arm, navigability"
---

# Regression of 2026-10-03, regression tier, present arm, navigability

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:1170ba43bc6abce3e6f97e334983a8bb64279e2e49e5647b63ccfd372e211425
lock: sha256:b768fe795c8d44a882e8604a5012c2eb5f2b4d584e7445367dfb24bb21c3585d
selection: sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76
read_set: sha256:3c8d7344cae9968817e188162a89971b634e7db5f1285a9e57264a0d132f9e2b
seed: 0
harness: 0.5.0
tier: regression
arm: present
at: 2026-10-03
cost_cents: 72
```

**This is the sealed re-recording that [HW-OBL-0200](../obligations/0200-three-graded-regression-runs-are-owed-a-fresh-recording-against-a-moved-taxonomy-lock.md) and [#1474](https://github.com/headwater-ai/headwater/issues/1474) asked for.** `tools/probe/campaign.sh` recorded it on 2026-10-03 (UTC), over the eight probes that the three retired regression runs graded, in three lines: discovery, navigability and sufficiency. This transcript is one of the three. The owner agreed the spend in writing on 2026-10-01 ([the ruling](https://github.com/headwater-ai/headwater/issues/1474#issuecomment-5935450057)), and the gate of [#1467](https://github.com/headwater-ai/headwater/issues/1467) lifted when that issue closed.

**One batch, one tree.** Every session of the three lines ran in a fresh copy of one `git archive` of commit `a46bd95e`, with the engine that commit builds. The sessions ran one at a time, in the order that seed 0 shuffled, on `claude-sonnet-5` under `claude` 2.1.288, with a cap of 80 turns and one repetition of each probe. The cap stopped no session. Each workspace lost the instrument: `docs/probes/`, `docs/probe-runs/`, `docs/probe-results/` and `.headwater/export.json` were absent from the sealed tree. It also lost every record under `docs/` that names a probe of the batch, and every file outside `docs/` that names one and is not a declared fold. The leak check (`seal.sh --leak`, with the configuration directory of a session) printed no `leak` line. All eight probes declare no leak string, so that check could find no leak of any of them, and the seal by name is the only part of the seal that held these eight.

**The confinement.** Each session ran under `bwrap` with no network of its own, under a configuration directory that held a copy of the host's credentials and nothing else, with `WebSearch` and `WebFetch` denied. Its one route out was the egress proxy of `tools/probe/probe-record.sh`, which forwards only requests to `api.anthropic.com` and reads each one first. The raw harness logs of the eight sessions hold 0 calls to `WebSearch` or `WebFetch`. The proxy refused no request that asked the provider to fetch from another host. It refused 4 connections to `api.anthropic.com:443` in each session, because it opens no tunnel. The recorded calls of the eight sessions name 0 paths on `docs/probes/`, `docs/probe-runs/` or `docs/probe-results/`. The retired recording of 2026-09-17 after the probe corrections named such a path in 5 calls. Seven sessions named 0 paths outside their workspace. The sufficiency session of the unmeasured-claim probe named 3: `/tmp/check_out.txt`, `/tmp/check_out2.txt` and `/tmp/gen_out.txt`, which it wrote itself as redirect targets. The confinement mounts `/tmp` as a tmpfs of the session's own, so no file of the host was at those paths.

**The cost of the batch.** The eight sessions spent 437 cents against a cap of 400 cents for the batch. The driver starts a session while the cents spent plus one declared session cost of 25 cents stay at or below the cap. The last session started at 341 cents spent and spent 96 cents, so the realized total passed the cap by 37 cents. The declared cost was 25 cents a session, and the realized mean was about 55 cents.

**It stands in for three retired transcripts, and it replaces one of them.** The regression runs of [2026-09-16](regression-probe-transcript-for-2026-09-16.md), [2026-09-17](regression-probe-transcript-for-2026-09-17.md) and [2026-09-17 after the probe corrections](regression-probe-transcript-for-2026-09-17-after-the-probe-corrections.md) are `deprecated`, and each one ran before the seal. The batch repeats the configuration of the third: `claude-sonnet-5` over the same eight probes. The run of 2026-09-17 graded the same eight probes on the same model, before three probe documents were corrected. Only the corrected probes exist now, so that run cannot be recorded again. The run of 2026-09-16 graded the eight probes on `claude-haiku-4-5`, and no sealed recording of that configuration exists. Whether one is owed is a spend question for the owner, and #1474 stays open for it.

This transcript's own change touches no key of `.headwater/taxonomy.lock` or `.headwater/overlay.yml`.

**This line.** It ran the three navigability probes of the eight: the adjudication probe at 21 cents, the register probe at 20 cents and the cited-ruling probe at 31 cents, 72 cents in all. The intent hook was live in all 3 sessions.

## Events

```yaml
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L2-regression-present-p1-r1"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p1-r1/docs/spec/09-open-questions.md"
      result: "sha256:5e219c743205dce8d98169409ec9891fdab2102bc9e09a4f8a40b5f369ac1309"
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p1-r1/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L2-regression-present-p2-r1"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Grep"
      argument: "{\"pattern\":\"scent\",\"output_mode\":\"files_with_matches\",\"-i\":true}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p2-r1/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p2-r1/docs/spec/09-decisions.md"
      result: "sha256:0c0c538720be0d70206ed1adb3d695531e5b1c9c975b992ad56c4c0df1896e0a"
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p2-r1/docs/spec/09-decisions.md"
      result: "sha256:0c0c538720be0d70206ed1adb3d695531e5b1c9c975b992ad56c4c0df1896e0a"
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p2-r1/docs/spec/09-decisions.md"
      result: "sha256:0c0c538720be0d70206ed1adb3d695531e5b1c9c975b992ad56c4c0df1896e0a"
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L2-regression-present-p3-r1"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls /var/tmp/hw-1474/batch/ws/L2-regression-present-p3-r1\",\"description\":\"List repository root contents\"}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"why is a count taken over the whole corpus computed at read time rather than stored in a committed file\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"which headwater 2>/dev/null; ls engine/target/*/headwater* 2>/dev/null; find . -maxdepth 2 -iname \\\"*.headwater*\\\"\",\"description\":\"Check if headwater CLI is available or built\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"./engine/target/dev-release/headwater route \\\"why is a count taken over the whole corpus computed at read time rather than stored in a committed file\\\" --root . 2>&1 | head -100\",\"description\":\"Run headwater route for the query\"}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p3-r1/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p3-r1/docs/process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md"
      result: "sha256:5e8cf3e064214715cbca7a9cafac21871bf5baf37e87d4dd0ba39b669b9c3e1f"
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L2-regression-present-p3-r1/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:cc0d9c67ecca22d8c07f6d43dcff6a9c0f0a3b3a0b2a0e729cb5e767a0868568"
    - tool: "Bash"
      argument: "{\"command\":\"./engine/target/dev-release/headwater explain docs/evaluations/why-corpus-counts-are-derived-not-stored.md --root . 2>&1 | head -80\",\"description\":\"Explain the existing evaluation document\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"./engine/target/dev-release/headwater check docs/evaluations/why-corpus-counts-are-derived-not-stored.md --root . 2>&1 | head -60\",\"description\":\"Run headwater check against the existing document\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"./engine/target/dev-release/headwater check --root . 2>&1 | grep -i \\\"why-corpus-counts\\\" -A5 -B5; echo \\\"---full tail---\\\"; ./engine/target/dev-release/headwater check --root . 2>&1 | tail -30\",\"description\":\"Run full check and look for mentions of the target document\"}"
      result: ""
  produced: []
  answer: null
```
