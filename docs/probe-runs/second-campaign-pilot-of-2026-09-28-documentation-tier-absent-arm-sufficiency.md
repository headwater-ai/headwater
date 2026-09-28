---
id: HW-RUN-second-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency
status: deprecated
status_since: 2026-09-28
summary: "The absent arm of the documentation tier over the sufficiency selection: 4 sessions, 55 cents, the intent hook live in 0. The recording is retired as a pilot, and it is graded again since #1292."
last_verified: 2026-09-28
tier: documentation
arm: absent
title: "Second campaign pilot of 2026-09-28, documentation tier, absent arm, sufficiency"
---

# Second campaign pilot of 2026-09-28, documentation tier, absent arm, sufficiency

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0
lock: sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8
selection: sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d
read_set: sha256:7a76c0b28551210b534705cf2b69216c68b6543dba250fc82fb24861e4949f46
seed: 0
harness: 0.4.0
tier: documentation
arm: absent
at: 2026-09-28
cost_cents: 55
```

**This is a second pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at one repetition per probe, to test the three changes the owner ruled on [#980](https://github.com/headwater-ai/headwater/issues/980) after the first pilot. First, every workspace of both arms carries the built `headwater`, so the intent hook can run in a present-arm session. Second, a closed-set word alone on the final line is the answer. Third, the seal removes HW-OBL-0198, the answer key of the `patched` probe. No comparison pools it with the full run, and no obligation is restated on it.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `ed9e61a7`, with the engine that commit builds, in one order that seed 981 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names a probe of the batch, and the answer keys of those probes. Each absent workspace also lost its tier's ablation.

**This transcript stands at `deprecated`, and the reason is the lock it pins.** The digest above is the lock of the tree that these pilot sessions met. The taxonomy change that declared the `subsystem_spec` kind on 2026-09-28 moved that lock, and the first of the five confirmations refused this recording until #1292. Since #1292, that confirmation reads a transcript with a moved lock when the read set of its probes did not move. The read set of this selection did not move, so the result derived from it carries verdicts again. The transcript stays at `deprecated` because it is a pilot. No comparison pools it with the full run, and a reader may take no rate from it as a measurement. The full campaign batch records its own sessions.

## Events

```yaml
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L3-documentation-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1/docs -type f | head -200\",\"description\":\"List files under docs directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1/\",\"description\":\"List working directory contents\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1 -type d -iname \\\"docs*\\\"; echo \\\"---\\\"; find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1 -iname \\\"*tombstone*\\\"; echo \\\"---grep amber---\\\"; grep -ril \\\"amber\\\" /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1 2>/dev/null\",\"description\":\"Search for docs directories, tombstone files, and amber references\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1/engine/crates/cli/fixtures/answered-export/verify.py"
      result: "sha256:d51d579a794cfb58d8b642093d892bc374ff02c6110f46a752b6fc65ffa615f5"
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1/mkdocs -maxdepth 3; echo \\\"---site---\\\"; find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p1-r1/site -maxdepth 2\",\"description\":\"Inspect mkdocs and site directory structure\"}"
      result: ""
  produced: []
  answer: "absent"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L3-documentation-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -type f | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" . 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" --include=*.rs . 2>/dev/null | xargs grep -n \\\"accepted_by\\\" 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p2-r1/engine/crates/query/src/mcp.rs"
      result: "sha256:4f2630fc853b02b40f23ae98415ddbb7c9f27228edf06490a79888a0f790e09d"
    - tool: "Bash"
      argument: "{\"command\":\"find . -path ./engine/target -prune -o -type d -iname \\\"spec\\\" -print -o -type d -iname \\\"docs\\\" -print 2>/dev/null; echo \\\"---\\\"; find . -iname \\\"03-authoring*\\\" 2>/dev/null; echo \\\"---docs dir---\\\"; ls -la docs 2>/dev/null; find . -maxdepth 1 -type d\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"warrant\\\" engine/crates/check/src/*.rs 2>/dev/null | head -60\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p2-r1/engine/crates/check/src/promotion.rs"
      result: "sha256:e694ff623defba242adcc5c136b78dfb0741a47779ad0b4a648fa638022b3179"
  produced: []
  answer: "stamp"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L3-documentation-absent-p3-r1"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L3-documentation-absent-p4-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p4-r1/ 2>&1\",\"description\":\"List contents of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p4-r1/CLAUDE.md 2>&1 | head -100; echo \\\"---FIND---\\\"; find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L3-documentation-absent-p4-r1 -maxdepth 2 -iname \\\"*todo*\\\" -o -maxdepth 2 -iname \\\"*backlog*\\\" -o -maxdepth 2 -iname \\\"*debt*\\\" -o -maxdepth 2 -iname \\\"*known*issue*\\\" 2>&1\",\"description\":\"Check CLAUDE.md and search for debt/todo tracking files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la AGENTS.md; echo \\\"---\\\"; find . -iname \\\"*todo*\\\" -o -iname \\\"*backlog*\\\" -o -iname \\\"*debt*\\\" 2>&1 | grep -v -E '^\\\\./(site|\\\\.git)' | head -50\",\"description\":\"Check AGENTS.md symlink target and search whole repo for debt/todo files\"}"
      result: ""
  produced: []
  answer: null
```
