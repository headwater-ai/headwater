---
id: HW-RUN-regression-of-2026-10-03-regression-tier-present-arm-sufficiency
status: current
status_since: 2026-10-03
summary: "The sufficiency line of the sealed re-recording of the three retired regression runs, on claude-sonnet-5 over two probes: 2 sessions, 184 cents, 0 web-tool calls and 0 calls naming a probe shelf."
last_verified: 2026-10-03
tier: regression
arm: present
title: "Regression of 2026-10-03, regression tier, present arm, sufficiency"
---

# Regression of 2026-10-03, regression tier, present arm, sufficiency

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:1170ba43bc6abce3e6f97e334983a8bb64279e2e49e5647b63ccfd372e211425
lock: sha256:b768fe795c8d44a882e8604a5012c2eb5f2b4d584e7445367dfb24bb21c3585d
selection: sha256:d0fddc9f0c2dd6ec76cbaa0a7a483989929218b5457cef93846a007b0420940b
read_set: sha256:4b05d45fef40465836aad67df91f018e9bc23f0fea8aeb72ac5cf9ea9764b7d3
seed: 0
harness: 0.5.0
tier: regression
arm: present
at: 2026-10-03
cost_cents: 184
```

**This is the sealed re-recording that [HW-OBL-0200](../obligations/0200-three-graded-regression-runs-are-owed-a-fresh-recording-against-a-moved-taxonomy-lock.md) and [#1474](https://github.com/headwater-ai/headwater/issues/1474) asked for.** `tools/probe/campaign.sh` recorded it on 2026-10-03 (UTC), over the eight probes that the three retired regression runs graded, in three lines: discovery, navigability and sufficiency. This transcript is one of the three. The owner agreed the spend in writing on 2026-10-01 ([the ruling](https://github.com/headwater-ai/headwater/issues/1474#issuecomment-5935450057)), and the gate of [#1467](https://github.com/headwater-ai/headwater/issues/1467) lifted when that issue closed.

**One batch, one tree.** Every session of the three lines ran in a fresh copy of one `git archive` of commit `a46bd95e`, with the engine that commit builds. The sessions ran one at a time, in the order that seed 0 shuffled, on `claude-sonnet-5` under `claude` 2.1.288. Each probe ran once, with a cap of 80 turns. The cap stopped no session. Each workspace lost the instrument: `docs/probes/`, `docs/probe-runs/`, `docs/probe-results/` and `.headwater/export.json` were absent from the sealed tree. It also lost every record under `docs/` that names a probe of the batch. Outside `docs/`, it lost every file that names one and is not a declared fold. The leak check (`seal.sh --leak`, with the configuration directory of a session) printed no `leak` line. All eight probes declare no leak string, so that check could find no leak of any of them. Only the seal by name held these eight probes.

**The confinement.** Each session ran under `bwrap` with no network of its own, and with `WebSearch` and `WebFetch` denied. Its configuration directory held a copy of the host's credentials and nothing else. Its one route out was the egress proxy of `tools/probe/probe-record.sh`, which forwards only requests to `api.anthropic.com` and reads each one first. The raw harness logs of the eight sessions hold 0 calls to `WebSearch` or `WebFetch`. The proxy refused no request that asked the provider to fetch from another host. It refused 4 connections to `api.anthropic.com:443` in each session, because it opens no tunnel. The recorded calls of the eight sessions name 0 paths on `docs/probes/`, `docs/probe-runs/` or `docs/probe-results/`. The retired recording of 2026-09-17 after the probe corrections named such a path in 5 calls. Seven sessions named 0 paths outside their workspace. The sufficiency session of the unmeasured-claim probe named 3: `/tmp/check_out.txt`, `/tmp/check_out2.txt` and `/tmp/gen_out.txt`, which it wrote itself as redirect targets. The confinement mounts `/tmp` as a tmpfs of the session's own, so no file of the host was at those paths.

**The cost of the batch.** The eight sessions spent 437 cents against a cap of 400 cents for the batch. The driver starts a session while the cents spent plus one declared session cost of 25 cents stay at or below the cap. The last session started at 341 cents spent and spent 96 cents, so the realized total passed the cap by 37 cents. The declared cost was 25 cents a session, and the realized mean was about 55 cents.

**It stands in for three retired transcripts, and it replaces one of them.** The regression runs of [2026-09-16](regression-probe-transcript-for-2026-09-16.md), [2026-09-17](regression-probe-transcript-for-2026-09-17.md) and [2026-09-17 after the probe corrections](regression-probe-transcript-for-2026-09-17-after-the-probe-corrections.md) are `deprecated`, and each one ran before the seal. The batch repeats the configuration of the third: `claude-sonnet-5` over the same eight probes. The run of 2026-09-17 graded the same eight probes on the same model, before three probe documents were corrected. Only the corrected probes exist now, so that run cannot be recorded again. The run of 2026-09-16 graded the eight probes on `claude-haiku-4-5`, and no sealed recording of that configuration exists. Whether one is owed is a spend question for the owner, and #1474 stays open for it.

This transcript's own change touches no key of `.headwater/taxonomy.lock` or `.headwater/overlay.yml`.

**This line.** It ran the two sufficiency probes of the eight, for 184 cents in all. The tombstone probe spent 22 cents and the unmeasured-claim probe 162 cents. The intent hook was live in both sessions. The unmeasured-claim session is the one that named three paths under its own `/tmp`.

## Events

```yaml
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L3-regression-present-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs -maxdepth 2 -type d | sort\",\"description\":\"List top-level docs directory structure\"}"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs/how-to/diagnose-an-isolation-failure.md"
      result: "sha256:2b8928f55d654f08c5096dfcd6e69c06a2e101f5d597fe6a7158669b783d1ac4"
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs/decisions/0100-a-counted-tombstone-lists-a-digest-of-each-withheld-identifier-and-a-sealed-one-lists-nothing.md"
      result: "sha256:eef84b6008bc8560a240b6d5069b689123d763db63a396f1a522c48c691949c8"
    - tool: "Bash"
      argument: "{\"command\":\"find /var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs -iname \\\"*recovery*\\\"\",\"description\":\"Search docs for files with recovery in the name\"}"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p1-r1/docs/interfaces/headwater-export.md"
      result: "sha256:952caaca5dc5cdfe922c8bbb2f6cf7ba813803745b8e5a27be7df3d995fa1881"
  produced: []
  answer: "absent"
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L3-regression-present-p2-r1"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la && git status 2>&1 | head -20\",\"description\":\"Check working directory contents and git status\"}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"session budget consumed by standing instructions before work starts; where this repository tracks what it still owes / debt / obligations\"}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-engine\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/engine && test -x target/dev-release/headwater && echo \\\"already built\\\" || echo \\\"need build\\\"\",\"description\":\"Check if headwater CLI already built\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1 && engine/target/dev-release/headwater route \\\"how much of a session's budget do the standing instructions of this repository consume before any work starts\\\" --root . 2>&1 | head -80\",\"description\":\"Route the task to find relevant governing documents\"}"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/obligations/README.md"
      result: "sha256:cb2292fec7109073d1627dbd6f46f9998d00e36ec1387bae57a2e8df66901971"
    - tool: "Bash"
      argument: "{\"command\":\"ls /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/\",\"description\":\"List process obligations shelf\"}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/README.md"
      result: "sha256:d6a88e0638e050cf66d307aba52d6b5425c3d0a75d500c988eff720b4e8cf8d6"
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0207-the-plan-usage-of-one-issue-is-known-only-as-the-average-of-its-run.md"
      result: "sha256:ff3238ef34ae5212a09771241fa369b5eb679d2f0dc16fa32ddefb1363fd8c11"
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md"
      result: "sha256:7bd8023284f56efbb21dda88e0d9544363be9ec24bcfaac813e2b97f08e01377"
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/evaluations/the-build-order-as-a-multi-agent-system.md"
      result: "sha256:6a3e40cd97a941fdeadc8d500557711c3d418b5c26444bf04618fca58e6c7f33"
    - tool: "Bash"
      argument: "{\"command\":\"ls /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/.headwater/ids/ 2>&1; echo \\\"---\\\"; find /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/.headwater/ids -iname \\\"*obl*\\\" 2>&1 | head -5; echo \\\"---count---\\\"; ls /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/.headwater/ids/*/ 2>&1 | head -30\",\"description\":\"Inspect identifier claim store for obligation scheme\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/.headwater/ids/obligation_record_id/ | sed 's/HW-OBL-//' | sort -n | tail -5\",\"description\":\"Find highest minted obligation record identifier\"}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-authoring\",\"args\":\"Create a process obligation record on docs/process/obligations/ stating that nothing in this repository measures how much of a session's token budget the standing instructions (CLAUDE.md, the UserPromptSubmit route hook output, the skill and agent listings) consume before any task work starts, for an ordinary session (not just the per-dispatch build-order figure already recorded in docs/process/evaluations/the-build-order-as-a-multi-agent-system.md).\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater new --help 2>&1 | head -60\",\"description\":\"Show headwater new help to list available kinds\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater new 2>&1 | head -60\",\"description\":\"List valid document kinds via refusal message\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater new bogus-kind-xyz 2>&1 | head -60\",\"description\":\"Trigger refusal listing every declared kind\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater new bogus-kind-xyz --title \\\"Test\\\" --root . 2>&1 | head -60\",\"description\":\"Trigger kind-not-found refusal to list declared kinds\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater new process_obligation --title \\\"Nothing sums what a session's standing instructions cost before its first turn\\\" --summary \\\"CLAUDE.md, the route hook's printed pointers, and the skill and agent listings all load before any task work starts, and only the build-order per-dispatch figure for CLAUDE.md has ever been priced.\\\" --root . 2>&1\",\"description\":\"Scaffold the new process obligation document\"}"
      result: ""
    - tool: "Grep"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/.headwater"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"sed -n '1,10p' /var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/obligations/0201-no-coverage-or-mutation-measurement-has-ever-been-taken-over-the-engine-s-test-suite.md\",\"description\":\"Check frontmatter of a similar measurement-gap obligation\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater new process_obligation --title \\\"Nothing sums what a session's standing instructions cost before its first turn\\\" --summary \\\"CLAUDE.md, the route hook's printed pointers, and the skill and agent listings all load before any task work starts, and only the build-order per-dispatch figure for CLAUDE.md alone has ever been priced.\\\" --facet waiting_on=measurement --root . 2>&1\",\"description\":\"Scaffold the new process obligation document\"}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Skill"
      argument: "{\"skill\":\"hw-corpus\"}"
      result: ""
    - tool: "Edit"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Edit"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater generate --root . 2>&1 | tail -20\",\"description\":\"Regenerate derived artifacts including shelf index\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"0233\\\" docs/process/obligations/README.md\",\"description\":\"Confirm the new record appears in the regenerated shelf index\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater check --strict --root . 2>&1 | tail -40; echo \\\"EXIT:$?\\\"\",\"description\":\"Run strict check over the corpus\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater check --strict --root . > /tmp/check_out.txt 2>&1; echo \\\"EXIT:$?\\\"; grep -n \\\"0233\\\" /tmp/check_out.txt; echo \\\"---errors/findings summary---\\\"; tail -5 /tmp/check_out.txt\",\"description\":\"Run strict check, capture real exit code, and filter for the new document\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"✗\\\\|error\\\\b\\\" /tmp/check_out.txt | grep -v \\\"sha256\\\" | head -30; echo \\\"---counts---\\\"; grep -oP '\\\\d+ (error|warn)[a-z]*' /tmp/check_out.txt | sort | uniq -c | head; echo \\\"---near our doc lines with context---\\\"; sed -n '2505,2530p' /tmp/check_out.txt\",\"description\":\"Look for error-severity findings and summary counts\"}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Edit"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Edit"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Edit"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Edit"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
    - tool: "Bash"
      argument: "{\"command\":\"engine/target/dev-release/headwater generate --root . > /tmp/gen_out.txt 2>&1; echo \\\"GEN_EXIT:$?\\\"; engine/target/dev-release/headwater check --strict --root . > /tmp/check_out2.txt 2>&1; echo \\\"CHECK_EXIT:$?\\\"; grep -n \\\"0233\\\" /tmp/check_out2.txt\",\"description\":\"Regenerate derived artifacts and re-run strict check\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -c \\\"✗ error\\\" /tmp/check_out2.txt; grep -c \\\"▲ warn\\\" /tmp/check_out2.txt\",\"description\":\"Confirm error/warn counts unchanged by our addition\"}"
      result: ""
    - tool: "Read"
      argument: "/var/tmp/hw-1474/batch/ws/L3-regression-present-p2-r1/docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
  produced:
    - path: "docs/process/obligations/0233-nothing-sums-what-a-session-s-standing-instructions-cost-before-its-first-turn.md"
      result: "sha256:3cd1df0dbf0b242e97f79c46a1242ade6c1f5773ac60dbe528c2e6671f69be77"
      cites:
        - "HW-OBL-0233"
        - "HW-PD-0001"
      findings:
        - "identifier.claim.missing"
  answer: null
```
