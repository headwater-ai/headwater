---
id: PROBE-HW-the-authoring-skill-reaches-an-agent-that-is-about-to-write-a-governed-document
status: draft
status_since: 2026-08-14
summary: Nothing makes a skill load, so its reach is a measurement, and this is the instrument that takes it.
last_verified: 2026-08-14
probe_category: discovery
expectation: opened
oracle: "none"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft
  evidence_basis: unevidenced
relations:
  examines:
    - .claude/skills/headwater-authoring/SKILL.md
  traces_to:
    - SPEC-HW-ai-integration
---

# The authoring skill reaches an agent that is about to write a governed document

## Task

Record that the retry ceiling in this system is unmeasured, so that a later reader can find it.

The task states an intent and names no verb, no kind and no file. A session that writes a Markdown file by hand has answered it, and so has a session that runs `headwater new obligation_record`. The two differ in what they read first.

## Expectation

`opened` over `.claude/skills/headwater-authoring/SKILL.md`, which is a `code_path` anchor rather than a document of this corpus.

**That target is why this probe exists and why the relation admits an anchor.** [Spec 5](../spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) rules that the reach of a skill is a measurement rather than a property: a harness reads a description and a model picks, and nothing makes a skill load. The coherence sweep cannot grade a skill description, because a sweep refuses any path that is not a classified document. The `opened` expectation over a transcript can, because a transcript records what a session read and not what the corpus contains.

**A `not_opened` arm is not the counterfactual, and the `absent` arm is.** Running this probe with the skill removed measures whether the description won against the alternative of nothing. Running it with a paraphrased description measures the description. Both are runs of this probe with the same expectation, which is what [the anti-overfitting rule](../spec/05-ai-integration.md#anti-overfitting) asks: a paraphrase varies the task statement and never the predicate.

Until a run happens, nothing about the reach of any skill in this repository is known, and no claim about the [assisted fraction](../spec/03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) rests on one.
