---
id: HW-RUN-regression-probe-transcript-for-2026-09-11
status: deprecated
status_since: 2026-09-11
summary: "Four regression sessions ran against the lock of 2026-09-11, and the recorder observed every tool call each one made. The recording is retired, because a later change moved the lock it pins."
last_verified: 2026-09-11
tier: regression
arm: present
title: "Regression probe transcript for 2026-09-11"
---

# Regression probe transcript for 2026-09-11

## Run identity

```yaml
model: claude-haiku-4-5
served_version: claude-haiku-4-5-20251001
tree: sha256:57e3ccceda009a3f910ddb9a5b01a81079c399ffdd474719df8fcc7bf9bf8cb4
lock: sha256:52e80828f5e9333f24256b9163ee363e3e52069d6853a50431feabc286de5ff1
selection: sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a
read_set: sha256:4552fe4c8345b84c96dfb90fcaf048fcdd9284642c4771077b49077369b2ec17
seed: 0
harness: 0.1.2
tier: regression
arm: present
at: 2026-09-11
cost_cents: 43
```

`tools/probe/probe-record.sh` drove four sessions, one for each probe `headwater probe plan --tier regression` selected, and each one ran in its own copy of this corpus outside this checkout. Claude Code 2.1.268 wrote each stream to the recorder's standard-output pipe. The harness wrote this log and the model did not. The identity above is the identity the plan fixed before the first session started, and the four members that belong to the run are the model, the served version, the date and the realized cost. The cost is the sum over the four sessions, which spent 28, 7, 4 and 4 cents against a declared 25 cents each.

**This transcript stands at `deprecated`, and the reason is the lock it pins.** The digest above is the lock of the tree these four sessions met. A later change to the taxonomy moved that lock, so the first of the five confirmations refuses this recording and the result derived from it carries no verdict. Nobody has recorded four fresh sessions against the tree that stands now. A reader may not take any rate from this file as a measurement of this corpus, and the record of what the sessions did is kept here for whoever records the next four.

It stood at `current` until that lock moved, which is what held every taxonomy change to a fresh recording. [HW-DR-0062](../decisions/0062-a-refused-recording-is-held-by-the-reliance-its-state-claims-and-not-by-promotion.md) is the ruling that made this retirement reachable, and [spec 15](../spec/15-the-recorder-contract.md) states what a run now does about a refusal. A refused recording at this state is reported by `headwater generate` and does not fail the run.

## Events

```yaml
- probe: "HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor"
  session: "regression-20260911-cold-current"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":1}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{\"name\":\"empty-summary-rule\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la\",\"description\":\"Check current directory and its contents\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git init\",\"description\":\"Initialize git repository\"}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{\"name\":\"empty-summary-rule\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git status\",\"description\":\"Check git status to verify repo initialization\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git add .gitignore && git commit -m \\\"Initial commit\\\" --allow-empty\",\"description\":\"Create initial commit for worktree support\"}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{\"name\":\"empty-summary-rule\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/\",\"description\":\"List contents of .headwater directory\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.yml"
      result: "sha256:3e2ddc9cba2aa02c51b31ec95bf4d05f90ab84c9e7d3ea780d054a119adb61e8"
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/overlay.yml"
      result: "sha256:aaf39bfa96d69cfdd4d7c18188318782e031a116f2ace0df401d213f089648c0"
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/overlay.yml"
      result: "sha256:aaf39bfa96d69cfdd4d7c18188318782e031a116f2ace0df401d213f089648c0"
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/overlay.yml"
      result: "sha256:aaf39bfa96d69cfdd4d7c18188318782e031a116f2ace0df401d213f089648c0"
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine -type f -name \\\"*.rs\\\" | head -20\",\"description\":\"Find Rust source files in the engine\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -r \\\"summary\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine --include=\\\"*.rs\\\" | head -20\",\"description\":\"Search for summary-related code in engine\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine/crates/sweep/\",\"description\":\"List sweep crate directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine/crates/sweep/src/\",\"description\":\"List sweep source directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"empty\\\\|summary\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine/crates/sweep/src/intake.rs | head -30\",\"description\":\"Search for empty and summary patterns in intake\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock"
      result: "sha256:ea0db96299f4ffe3f6d593bc7e30ec66cef1c98fe9692aafe9562a04c19651cd"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"^  rules:\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock | head -5\",\"description\":\"Find where rules are defined in taxonomy.lock\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"rules:\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock | head -20\",\"description\":\"Find all rules sections in taxonomy.lock\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-170-probes.wlPHN4/cold-workspace/packages -name \\\"*.yml\\\" -o -name \\\"*.yaml\\\" | head -20\",\"description\":\"Find YAML files in packages directory\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/packages/headwater-standard/conformance.yml"
      result: "sha256:9ff0735e955bea021067c9949782bb01b5bd2ea46ef7be87c929d3fe9094f0db"
    - tool: "Bash"
      argument: "{\"command\":\"grep -r \\\"section\\\\|facet\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/packages/headwater-standard/taxonomy.yml | head -30\",\"description\":\"Search for section/facet rules in base taxonomy\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/packages/headwater-standard/taxonomy.yml"
      result: "sha256:f8404c347cbf6760c27b52fd5d14678186f42285451f83acfc36ad851abcdb65"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"checks:\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/packages/headwater-standard/taxonomy.yml\",\"description\":\"Find checks section in base taxonomy\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"tail -500 /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock | head -200\",\"description\":\"Examine end of taxonomy.lock for check definitions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"facet\\\\|FACET\\\\|summary\\\\|SUMMARY\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock | head -30\",\"description\":\"Search for facet and summary in taxonomy.lock\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock"
      result: "sha256:ea0db96299f4ffe3f6d593bc7e30ec66cef1c98fe9692aafe9562a04c19651cd"
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock"
      result: "sha256:ea0db96299f4ffe3f6d593bc7e30ec66cef1c98fe9692aafe9562a04c19651cd"
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine -type f -name \\\"*.rs\\\" -exec grep -l \\\"facet.*required.*missing\\\\|facet.required\\\" {} \\\\;\",\"description\":\"Find where facet checks are implemented\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/cold-workspace/engine/crates/check/src/facet_blank.rs"
      result: "sha256:1ebf373da656429c364f31a2b3168a5b4b849ca308ed11c9079ebaabff5f8869"
    - tool: "Bash"
      argument: "{\"command\":\"grep -A 10 \\\"summary:\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/packages/headwater-standard/taxonomy.yml | head -20\",\"description\":\"Find summary facet definition in base taxonomy\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"OB-FACET\\\\|CT-FACET\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock\",\"description\":\"Find all facet-related obligations and controls\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"facet.value.blank\\\\|facet.blank\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/.headwater/taxonomy.lock\",\"description\":\"Search for facet.value.blank in taxonomy.lock\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"facet_blank\\\\|facet.value.blank\\\" /tmp/headwater-170-probes.wlPHN4/cold-workspace/engine/crates/check/src/lib.rs\",\"description\":\"Check if facet_blank is wired in the check crate\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "regression-20260911-tombstone-current"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-170-probes.wlPHN4/tombstone-workspace -type f -name \\\"*.md\\\" -o -name \\\"*.txt\\\" -o -name \\\"*.json\\\" | head -20\",\"description\":\"List files in the tombstone workspace to understand corpus structure\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/tombstone-workspace"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-170-probes.wlPHN4/tombstone-workspace -type f \\\\( -name \\\"*tombstone*\\\" -o -name \\\"*recovery*\\\" \\\\) 2>/dev/null\",\"description\":\"Find tombstone and recovery related files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -r \\\"amber\\\" /tmp/headwater-170-probes.wlPHN4/tombstone-workspace 2>/dev/null | head -10\",\"description\":\"Search for the word \\\"amber\\\" in the corpus\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/tombstone-workspace/docs/probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md"
      result: "sha256:053809c0132f430d157812cc03a0b5ebbacbfd7077e529315253f030ee1be9cf"
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/tombstone-workspace/.headwater/corpus.json"
      result: "sha256:d7f8c109f4bbc5ab4bf3f17c15665017dba79e91149f191cf7986412d0ca6b4e"
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-170-probes.wlPHN4/tombstone-workspace/docs -name \\\"*recovery*\\\" -type f\",\"description\":\"Find recovery-related files in the corpus\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /tmp/headwater-170-probes.wlPHN4/tombstone-workspace/docs/answers/ 2>/dev/null || echo \\\"No answers directory\\\"\",\"description\":\"Check for answers directory structure\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/tombstone-workspace/engine/crates/cli/fixtures/answered-export/docs/answers/recovery.md"
      result: "sha256:004433da618be44f09c72b05b3e590a6188020df229c6af7e28def1b63f6640d"
    - tool: "Bash"
      argument: "{\"command\":\"grep -r \\\"tombstone\\\" /tmp/headwater-170-probes.wlPHN4/tombstone-workspace/docs --include=\\\"*.md\\\" -l\",\"description\":\"Find documents mentioning tombstones\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/tombstone-workspace/docs/obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md"
      result: "sha256:4119ea0107a1fd3c5f815e0f78b20a67f0474a6abac8954c9a03cf3aca6e9d40"
    - tool: "Bash"
      argument: "{\"command\":\"grep -r \\\"tombstone\\\" /tmp/headwater-170-probes.wlPHN4/tombstone-workspace/.headwater --include=\\\"*.json\\\" --include=\\\"*.md\\\" 2>/dev/null | head -20\",\"description\":\"Search .headwater directory for tombstone references\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/tombstone-workspace/.headwater/corpus.json"
      result: "sha256:d7f8c109f4bbc5ab4bf3f17c15665017dba79e91149f191cf7986412d0ca6b4e"
    - tool: "Bash"
      argument: "{\"command\":\"grep -E \\\"filtered|profile|tombstone\\\" /tmp/headwater-170-probes.wlPHN4/tombstone-workspace/.headwater/corpus.json\",\"description\":\"Check for profile and tombstone configuration\"}"
      result: ""
  produced: []
  answer: "present"
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "regression-20260911-adjudication-current"
  calls:
    - tool: "Read"
      argument: "/tmp/headwater-170-probes.wlPHN4/adjudication-workspace/docs/spec/09-open-questions.md"
      result: "sha256:e2196f7a29315e6f0e5dd51493a44520f59069b5721f4a62074f954d7535a397"
  produced: []
  answer: null
- probe: "HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document"
  session: "regression-20260911-authoring-clean"
  calls:
    - tool: "Write"
      argument: "/home/james/.claude/projects/-tmp-headwater-170-probes-wlPHN4-authoring-workspace/memory/retry-ceiling-unmeasured.md"
      result: "sha256:257c53bc0ab412fb270bb87358134c66ea723853522a5ae998ba204c4da685f0"
    - tool: "Read"
      argument: "/home/james/.claude/projects/-tmp-headwater-170-probes-wlPHN4-authoring-workspace/memory/MEMORY.md"
      result: "sha256:3e47960ac6624cfdb8ffca9883affbd5a14893bad91b8ff99d8a4dede73011cb"
    - tool: "Write"
      argument: "/home/james/.claude/projects/-tmp-headwater-170-probes-wlPHN4-authoring-workspace/memory/MEMORY.md"
      result: "sha256:3e47960ac6624cfdb8ffca9883affbd5a14893bad91b8ff99d8a4dede73011cb"
  produced: []
  answer: null
```

## What the recorder observes, and what it observed here

The `answer` of the `HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer` event is the word the session put on the harness `result` line, and `tools/probe/probe-record.sh` derives it there. It derived nothing for the other three, which is correct: an `opened` probe declares no answer set, and the final text of such a session is prose that a transcript does not hold.

The first transform of these four logs wrote `answer: null` for all four, because the driver called the transform with no `--answer` at all. Spec 15 rules that `answer: null` says the recorder watched and saw no final answer, so that transcript said this session ended with nothing when it ended with `present`, and a quarter of the result beside it graded the recorder. [#803](https://github.com/headwater-ai/headwater/issues/803) carries the defect. The events above come from the same four logs, re-read by the corrected driver: the recording is the recording of 2026-09-11 and only the derivation moved.
