---
id: HW-DR-0115
status: current
status_since: 2026-11-25
summary: "An agent_rules file is one Copilot .instructions.md file for each source document with a governs edge. Its applyTo is the governs patterns, its marker is in its front matter, and its body points at the source. The projection declares a byte budget for one file and for all of them"
last_verified: 2026-11-25
title: "115 — An agent rules file is a budgeted pointer at the Copilot instructions path"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-SPEC-taxonomy-model
    - HW-SPEC-harness-support
    - HW-SPEC-vision-and-scope
    - HW-OBL-0034
---

# 115 — An agent rules file is a budgeted pointer at the Copilot instructions path

## Context

Spec 5 says that rules load when an agent opens a file that matches. A rule is a generated pointer to the standard that it derives from. It also says that a rule obeys a size budget, per file and in aggregate. The projection kind `agent_rules` had a place in the meta-schema and no emitter. Its reason in `unbuilt()` said that no document states the form of the file. No member of a projection could hold a budget, and [HW-OBL-0034](../obligations/0034-two-rules-of-taxonomy-validate-name-a-declaration.md) records that gap.

The owner ruled on 2026-11-25, in a comment on [#2026](https://github.com/headwater-ai/headwater/issues/2026): build the projection with the size budget that spec 5 requires. Spec 5 is not restated to drop the budget. The same comment states seven defaults for the build. The owner did not rule on them, and they stand unless the owner overturns them on #2026. The owner's ruling and the seven defaults are the evidence for this record.

[#2087](https://github.com/headwater-ai/headwater/issues/2087) is the first of four build issues. This record states the form, so that the emitter of [#2089](https://github.com/headwater-ai/headwater/issues/2089) implements a statement and does not make one.

## Decision

**The form is one file for each source document, at the GitHub Copilot path.** The output is `.github/instructions/{name}.instructions.md`. `{name}` is the file stem of the source document, which is its file name without `.md`. Two source documents with one stem would write one file. The emitter refuses that, and it does not overwrite. Spec 16 lists this path as the one glob-scoped surface of the three harnesses. A `CLAUDE.md` or an `AGENTS.md` in each directory waits, because adopters write those files themselves, and a generator would collide with them. The output ends in `.md`, so the census reads its marker.

**A source document is a document with a `governs` edge, of a kind that the projection's `for` admits.** The `applyTo` member of the front matter holds the `governs` patterns of that document.

**The marker is a member of the front matter.** The file opens with its front matter block, because Copilot reads `applyTo` from a block on line 1. So the `headwater:generated` marker is a top-level member of that block. A generated Markdown file with front matter already uses this form. A comment line above the block would move the block off line 1.

**The body is a pointer and not a copy.** It holds the title of the source document, its summary, its path, and the instruction "read this before you edit these files". It holds no other text of the source.

**The budget is in bytes, and the projection declares it.** The `budget` member of a projection holds two figures: `per_file`, which bounds one written file, and `aggregate`, which bounds every file of the declaration together. Both are positive whole numbers, and both are required. The engine takes `budget` on an `agent_rules` alone, and it refuses the member on every other kind. The suggested package defaults are about 600 bytes for each file and 8 KB in aggregate. An agent-facing projection with no budget fails `taxonomy validate`. That refusal lands with [#2088](https://github.com/headwater-ai/headwater/issues/2088). The findings for a file past its budget, and the pruning, land with [#2090](https://github.com/headwater-ai/headwater/issues/2090). The engine reads the budget today and enforces nothing with it.

**A rule file is a marked derived artifact with no identity of its own.** For pruning, its nuclearity is the nuclearity of its source document. This record adds no `agent_rule` kind and no `derives_from` edge.

**`generate` is the one producer.** `headwater init --harness` writes no rule file. This repository does not generate rule files for itself in this release. That needs the census to read marked files outside `docs/`, which is a separate follow-up.

**The wording follows principle 11 of spec 0.** A statement about a rule file says that it is "generated, budgeted, checked for regeneration; its effect on agents is unmeasured". No document claims that a rule file changes what an agent does until a measured arm exists, which is [#503](https://github.com/headwater-ai/headwater/issues/503).

## Consequences

The meta-schema gives the `projection` block a thirteenth member, `budget`, and a closed block `projection_budget` with the two figures. The generator's reader refuses `budget` on every kind but `agent_rules`, and `taxonomy resolve` and `taxonomy validate` run that reader before they write a lock. So an author who puts a budget on the wrong kind reads the refusal at resolve, against the source that wrote it.

The reason that `unbuilt()` gave for `agent_rules` named this record, and said that the emitter was not built. #2089 removed that reason when it built the emitter.

HW-OBL-0034 stays open. This record supplies the declaration that its size-budget half names. #2088 discharges that half, because the obligation is the refusal of a projection with no budget, and not the member that holds one. Its mapping-integrity half does not change.

The owner can overturn any of the seven defaults on #2026. A change to the form, the body or the budget figures is a change to this record before it is a change to an emitter.
