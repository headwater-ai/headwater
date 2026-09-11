---
id: HW-RUN-regression-probe-transcript-for-2026-09-09
title: Regression probe transcript for 2026-09-09
status: draft
status_since: 2026-09-09
summary: "Four regression sessions recorded the calls that a Claude Haiku session made while it answered the corpus probes."
last_verified: 2026-09-09
tier: regression
arm: present
---

# Regression probe transcript for 2026-09-09

## Run identity

```yaml
model: claude-haiku-4-5
served_version: claude-haiku-4-5-20251001
tree: sha256:73f19c035efb9259f56750d65d14ca049e3dd31b3749f75be03a4ec359e33aad
lock: sha256:de4da2c26f7744de974e46575a21d8fcdbeee813ba5dd4f23b71993e99527650
selection: sha256:2ae2482aea0cf2c00e83fac793d72d09c287514680ef632d535f1607d12a913a
read_set: sha256:78d36a3aecbfac469d19049a6f1d0286b97bbdf93793b27b216ebf5b95d76e84
seed: 0
harness: 0.1.1
tier: regression
arm: present
at: 2026-09-09
cost_cents: 28
```

The recorder ran `tools/probe-record.sh` from Headwater at `c18fa0df`. Claude Code 2.1.266 wrote its stream to the recorder's standard-output pipe. The harness, and not the model, wrote this log.

## Events

```yaml
- probe: "HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor"
  session: "regression-20260909-cold-current"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-724-probes.Ufy2y3/cold-workspace -name \\\"*.md\\\" -path \\\"*/docs/decisions/*\\\" | head -10\",\"description\":\"Find decision documents to understand rule patterns\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace/docs/decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md"
      result: "sha256:12b684170c99791eb4055003524b0ec031649ec1ef7d08e69a458c8940254e41"
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-724-probes.Ufy2y3/cold-workspace -name \\\".headwater\\\" -type d\",\"description\":\"Find the .headwater directory which contains taxonomy configuration\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater/\",\"description\":\"List files in .headwater directory\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater/taxonomy.yml"
      result: "sha256:533c52c13324be1eb94e097a95a205a0f26122dd68f6a361f4ef2d5e617e2b4c"
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-724-probes.Ufy2y3/cold-workspace/engine -name \\\"*.rs\\\" -path \\\"*/check/*\\\" | head -10\",\"description\":\"Find Rust files in the check layer\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"summary\\\" /tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater/overlay.yml | head -20\",\"description\":\"Search for summary-related rules in overlay\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -i \\\"summary\\\" /tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater/taxonomy.lock | head -10\",\"description\":\"Search for summary in taxonomy lock\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater/taxonomy.lock"
      result: "sha256:2f034ab5ac6f89d32da5368478d21a7e08d1f69e78bdc8fbaea53e9511b7c3ee"
    - tool: "Bash"
      argument: "{\"command\":\"grep -A 20 \\\"summary:\\\" /tmp/headwater-724-probes.Ufy2y3/cold-workspace/.headwater/taxonomy.lock | head -30\",\"description\":\"Find the summary facet definition in taxonomy lock\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /tmp/headwater-724-probes.Ufy2y3/cold-workspace/engine/crates -name \\\"*.rs\\\" | xargs grep -l \\\"facet.value.blank\\\" | head -5\",\"description\":\"Find implementation of the facet.value.blank rule\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace/engine/crates/check/src/facet_blank.rs"
      result: "sha256:1ebf373da656429c364f31a2b3168a5b4b849ca308ed11c9079ebaabff5f8869"
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/cold-workspace/engine/crates/cli/tests/blank_facet.rs"
      result: "sha256:928530614225e83227e1d26526bc3f153b036f377b44c2ba915e1fc0dab620d5"
  produced: []
  answer: null
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "regression-20260909-tombstone-current"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -type f \\\\( -name \\\"*.md\\\" -o -name \\\"*.txt\\\" \\\\) | head -20\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -r \\\"amber\\\" --include=\\\"*.md\\\" . 2>/dev/null | head -20\"}"
      result: ""
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/tombstone-workspace/engine/crates/cli/fixtures/answered-export/docs/answers/recovery.md"
      result: "sha256:004433da618be44f09c72b05b3e590a6188020df229c6af7e28def1b63f6640d"
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/tombstone-workspace/docs/probes/a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer.md"
      result: "sha256:053809c0132f430d157812cc03a0b5ebbacbfd7077e529315253f030ee1be9cf"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "regression-20260909-adjudication-current"
  calls:
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/adjudication-workspace/docs/spec/09-open-questions.md"
      result: "sha256:378925885531edac69ebc8c31b5c8939bbd273b67a7915d4158f648ed1303a7b"
    - tool: "Read"
      argument: "/tmp/headwater-724-probes.Ufy2y3/adjudication-workspace/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document"
  session: "regression-20260909-authoring-clean"
  calls:
    - tool: "Write"
      argument: "/home/james/.claude/projects/-home-james-projects-headwater/memory/retry-ceiling-unmeasured.md"
      result: "sha256:37b9900a96ed43effd5b850fd08943772989b8d0c576732755e19e649776dd76"
    - tool: "Read"
      argument: "/home/james/.claude/projects/-home-james-projects-headwater/memory/MEMORY.md"
      result: "sha256:9043eee03faabd9b6ce0869a80eff70bb074f552130b616fae7e4eef446c4c1e"
    - tool: "Edit"
      argument: "/home/james/.claude/projects/-home-james-projects-headwater/memory/MEMORY.md"
      result: "sha256:9043eee03faabd9b6ce0869a80eff70bb074f552130b616fae7e4eef446c4c1e"
  produced: []
  answer: null
```
