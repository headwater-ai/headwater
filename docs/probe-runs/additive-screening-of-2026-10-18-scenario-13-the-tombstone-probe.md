---
id: HW-RUN-additive-screening-of-2026-10-18-scenario-13-the-tombstone-probe
status: current
status_since: 2026-10-19
summary: "The scenario-13 arm of the additive screening round, line 15 of the batch: 18 sessions over 1 probe, 327 cents, the intent hook live in 18, 0 stopped at the turn cap, 0 stopped at the session budget, 0 paths outside the workspace named, 0 sessions uncounted, 0 web-tool calls."
last_verified: 2026-10-19
tier: additive
arm: scenario-13
title: "Additive screening of 2026-10-18, scenario 13, the tombstone probe"
relations:
  records:
    - HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded
provenance:
  warrant: asserted
---

# Additive screening of 2026-10-18, scenario 13, the tombstone probe

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:7d84079327a6496ad99d1047ec2c9d0f82a810c3765b0ae22a4ae3ea97be60e8
lock: sha256:1fb804541b7441a6588f57cc93c893118896dbf705119c83b1f9f77072386124
selection: sha256:d4a51542343398c4c62fd22e5b7e4efd1a2f39ff54e81fed513de756e487f98e
read_set: sha256:f2bed904e8aa89a1c71328e44e2d90ffeadc9aacd7a1c1e1237a8158ee5b8923
seed: 0
harness: 0.6.0
tier: additive
arm: scenario-13
at: 2026-10-18
cost_cents: 327
```

**This line is one of the screening round of the additive campaign that [#1775](https://github.com/headwater-ai/headwater/issues/1775) asked for.** `tools/probe/campaign.sh` recorded it as line 15 of the plan of the batch, a plan that draws its 15 lines from `tools/probe/additive-screening.spec`. It is the `scenario-13` arm of the additive tier over 1 probe of the tuning set, at 18 repetitions per probe, on `claude-sonnet-5` under Claude Code 2.1.288 with a cap of 80 turns for each session.

**One tree, one pin, one attempt.** Every session ran in a fresh copy of a `git archive` of the pinned tree, and the engine of that tree. The pin was `2d212a26` for the whole batch, and the batch did not restart. The `tree` digest in the identity above is the digest the assembly recorded.

**What the recorder counted.** The line holds 18 sessions and cost 327 cents: the intent hook live in 18, 0 stopped at the turn cap, 0 stopped at the session budget, 0 paths outside the workspace named, 0 sessions uncounted, 0 web-tool calls. A session the recorder could not count adds nothing to either count, so each count is a floor.

**The contract of this batch.** The sessions ran under the recorder contract of spec 15: a `bwrap` sandbox with no network of its own, a configuration directory that holds a copy of the credentials and nothing else, the permission mode `dontAsk`, and `WebSearch` and `WebFetch` denied. The round approved $700 over every attempt, and the driver ran with a cap of 70000 cents and a session budget of 300 cents.

**Where the raw recordings are.** The raw session logs are outside git, under `/mnt/build/hw-1775/screening/a/sessions/L15-*/raw.jsonl` on the host that ran the batch. Nobody deletes them until the report of [#1779](https://github.com/headwater-ai/headwater/issues/1779) is published.

**Where the figures are.** The rates, the intervals and the comparisons of this line are in the generated result that reads this transcript, and this document states none of them.

## Events

```yaml
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r1"
  calls:
    - tool: "Grep"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r1/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r1/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r1/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r10"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r10/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r10/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r11"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r11/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r11/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r12"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r12/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r12/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r12/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r13"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r13/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r13/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r14"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"paragraph limit counts sentences under the language rule and never words\"}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r14/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r14/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r15"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"69\\\" -A 20 \\\"/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r15/docs/spec/09-open-questions.md\\\" | head -60\"}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r15/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r15/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r16"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r16/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Grep"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r16/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Grep"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r16/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r16/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r17"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r17/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r17/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r18"
  calls:
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r18/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r2"
  calls:
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r2/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r2/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r3"
  calls:
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"paragraph limit sentence count language rule open question 69\"}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r3/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r3/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r4"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r4/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r4/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r4/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r5"
  calls:
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r5/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r5/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r6"
  calls:
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r6/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r6/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r7"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r7/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r7/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r7/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r8"
  calls:
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r8/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r8/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r8/docs/decisions/0087-no-paragraph-limit-joins-the-language-rule-because-most-paragraphs-past-six-sentences-hold-one-topic.md"
      result: "sha256:fb83eb7fd0c0c778d8743eea38fbf729bc0b675ee37b82099ad43a0501ac03fd"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-when-the-decision-a-tombstone-links-is-superseded"
  session: "L15-additive-scenario-13-p1-r9"
  calls:
    - tool: "Grep"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r9/docs/spec/09-open-questions.md"
      result: "sha256:e0eb42b40e1ebf1364562b3e8a3b6f3f6c1f44cc5f81c2c951f1f07d845499f6"
    - tool: "Read"
      argument: "/mnt/build/hw-1775/screening/a/ws/L15-additive-scenario-13-p1-r9/docs/decisions/0069-a-paragraph-limit-counts-sentences-under-the-language-rule-and-never-words.md"
      result: "sha256:3accae5c8185e53e67b62ca6e107fe6cba75ca820626b8d914350b7487921c1c"
  produced: []
  answer: null
```
