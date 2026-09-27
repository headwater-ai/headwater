---
id: HW-EVAL-external-evaluation-harnesses-and-benchmarks-for-the-probe-campaign
status: current
status_since: 2026-09-27
summary: "DeepEval, Promptfoo and Inspect AI can drive a probe session but never grade one. The campaign adopts none, because each reads model text or opens a channel the session can write to."
last_verified: 2026-09-27
title: "External evaluation harnesses and benchmarks for the probe campaign"
provenance:
  warrant: asserted
  agency: mixed
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
    - HW-SPEC-the-recorder-contract
    - HW-DR-0059
---

# External evaluation harnesses and benchmarks for the probe campaign

Issue #88 is the probe campaign. It runs the probe shelf against a corpus-present arm and a corpus-absent arm, and it publishes the rate with its counterfactual. Issue #1015 asked which external tool the campaign builds on, and which public benchmark supplies its scenarios. This evaluation answers both questions against three contracts that already stand:

- [Spec 5](../spec/05-ai-integration.md#measuring-whether-any-of-this-works) states that the grader is never the system under test, and that no grader reads prose.
- [Spec 5](../spec/05-ai-integration.md#what-earns-the-grader-the-right-that-every-other-agent-facing-mechanism-is-denied) names `headwater probe grade` as the grader. A recorded fixture set holds its verdicts, and it has a version.
- [Spec 15](../spec/15-the-recorder-contract.md#a-recorder-is-a-driver-and-a-transform-and-the-channel-between-them-is-the-condition) and [HW-DR-0059](../decisions/0059-a-transform-over-a-harness-session-log-is-an-observed-transcript-when-the-log-arrives-by-a-channel-the-model-cannot-write-to.md) admit a recorder that is a driver and a transform. The condition is that the log reaches the transform by a channel that the model cannot write to.

The grader is therefore fixed before any tool is compared. An external tool can have one role only: the **driver**, which starts the sessions and runs the matrix of model, arm and repetition. [HW-EVAL-adjacent-work §M.3](adjacent-work.md#m3-the-grader-decides-the-answer-and-the-literature-proves-it) is the evidence against a model judge, and this document does not repeat it.

## The question

Can DeepEval, Promptfoo or Inspect AI replace a part of the recorder or the grader that this repository already holds? Can BERBench make a corpus-present arm and a corpus-absent arm from one merged pull request, with no fixture that a person writes for each task? Which public benchmark supplies a scenario for each of the four probe categories?

## A. The three harnesses

Each version below is the latest release on its registry on 2026-09-27. The source read for each claim is the published package at that version, and not a documentation page that can change after this date.

| | DeepEval 4.2.6 | Promptfoo 0.123.1 | Inspect AI 0.3.271 |
|---|---|---|---|
| Grades with no model | Yes. `ToolCorrectnessMetric` with no `available_tools` compares two lists and calls no model ([metric docs](https://deepeval.com/docs/metrics-tool-correctness)). Most other metrics are model judges. | Yes. `trajectory:tool-used`, `contains`, `regex` and `javascript` call no model ([assertion docs](https://www.promptfoo.dev/docs/configuration/expected-outputs/)). | Yes. `includes`, `match`, `pattern` and a custom `@scorer` call no model ([scorer docs](https://inspect.aisi.org.uk/scorers.html)). `model_graded_qa` is a model judge. |
| What the grading reads | Its own `LLMTestCase.tools_called`, a list that the caller fills. The package has no Claude Code integration, so the caller is a transform like `tools/probe/probe-transform.sh`. | `trajectory:tool-used` reads OpenTelemetry spans. `contains` and `regex` read the final result text of the session. | Built-in scorers read `state.output.completion`, the final text. A custom scorer can read `tool_calls` on each assistant message. |
| The decisive fixture | Passes, but not by DeepEval. The metric sees only what the caller filtered, so the property is the transform's. | `contains: "tool: Read"` fails: it matches the bait `text` block. `trajectory:tool-used` reads `tool_use` blocks by type, but through a receiver that the session can write to (see below). | `includes("tool: Read")` fails: it matches the bait. A custom scorer over `tool_calls` passes on the block, but the bridge is a channel that the session can reach (see below). |
| Its own transcript or spec 15's | Its own test case. | Its own trace store. | Its own `.eval` log. |
| License | Apache-2.0 | MIT | MIT |
| Releases in the 90 days to 2026-09-27 | 21 | 8 | 30 |
| Dependency weight | Python, 31 required packages | Node, 80 required and 45 optional packages | Python, 41 required packages |

The decisive fixture is `tools/probe/fixtures/live-haiku-session.jsonl`. `tools/probe/probe-record-fixtures.sh` builds it so that each `thinking` and `text` block carries text shaped like a transcript field, and one `text` block writes `tool: Read`. A harness passes when its assertion that a tool was used reads the `tool_use` block by its type. A harness fails when the assertion can match the bait in a `text` block.

**Promptfoo gives its receiver to the session.** The Claude Agent SDK provider in 0.123.1 makes one span for each `tool_use` block, and it selects the block by type. It also writes `TRACEPARENT` and `OTEL_EXPORTER_OTLP_ENDPOINT` into the environment of the agent process. The default endpoint is `http://127.0.0.1:4318`. A `Bash` call of the session inherits both values, so it can post a span named `tool Read` into the trace that `trajectory:tool-used` reads. That is a channel the model can write to, and spec 15 refuses it.

**Inspect AI puts its bridge inside the sandbox.** `sandbox_agent_bridge` runs a model proxy on port 13131 in the same container as the agent. A `Bash` call of the session can send a request to that proxy with a message history that it wrote. This evaluation did not measure whether Inspect records such a request in `state.messages`. So the bridge channel is unconfirmed, and it is not safe.

**DeepEval brings no driver.** It has integrations for LangChain, LlamaIndex, CrewAI, Google ADK, Pydantic AI, Strands and AgentCore, and none for Claude Code. The recorder would stay `tools/probe/probe-record.sh`, and DeepEval would only compare two lists that `headwater probe grade` already compares with a witness.

### Recommendation: none

The campaign of #88 builds on no external harness. It uses `tools/probe/probe-record.sh` as the driver, `tools/probe/probe-transform.sh` as the transform, and `headwater probe grade` as the grader.

- None of the three can be the grader. Spec 5 names the grader, and none of the three carries a witness on a satisfied verdict or refuses where a pass is free.
- None of the three is a better driver. The in-repository driver reads the standard output of the harness process, which the session holds no handle on. Promptfoo adds a writable receiver. Inspect AI adds a proxy that the session can reach. DeepEval adds no driver.
- What a driver would add is the matrix and concurrency. `headwater probe plan` already fixes the selection, the arms and the budget, so a shell loop over it is the whole matrix.

Reopen this recommendation when a harness reads the session log from the standard output of the harness process. It must also keep only the `tool_use` and `tool_result` blocks, by type. Section D then applies.

## B. BERBench

BERBench 0.2.4 (`berbench 0.2.4 (3e5f5a2ccdaf0cba370f5e9404bdae9f9cc58fae)`, binary SHA-256 `df52846b6a4a3ba3cf2daebc340a4a01ee9870491bdb27c4aa8efc66b89dc443`) turns a merged pull request into a task. The linked issue becomes the prompt, and the parent of the fix becomes the start tree. The test changes become a hidden verifier, and the rest of the change becomes a reference patch. Its repository has no license, and its command-line tool is a closed binary.

**The answer is yes for a `patched` probe, and no for the other forms.** One merged pull request gives both arms with no fixture that a person writes for each task. Two files are written once for each repository, and one line is written for each task.

The run on this repository, in a clone outside the checkout, on 2026-09-27:

    BERBENCH_VERSION=v0.2.4 BERBENCH_BIN_DIR=/tmp/berbench-bin bash install
    berbench init
    berbench task scan --json          # 20 candidates
    berbench task create 1180
    berbench evaluation create arms
    berbench evaluation validate arms
    berbench task validate 1180

`task create 1180` wrote the prompt from issue #1136 with its number removed. It wrote `tests.patch` over one file, `engine/crates/cli/tests/fixed_name_shelf.rs`, and `reference.patch` over two files. It protected `engine/crates/cli/tests/`. It could not find a test command for a Cargo workspace under `engine/`, so it wrote the placeholder `false`. A person replaces that placeholder once for each task. `Dockerfile.berbench` is the second file that a person writes, once for each repository. It must copy the tree to `/workspace` and fetch every crate at build time, because the task container has no network.

VALIDATION-RESULT

**The absent arm is a `pre` command, and it copies the list that `ablate.sh` holds.** An evaluation block can declare `pre` commands, and each command is part of the cell identity. The block below resolved to three setups:

    tools:
      - tool: claude-code
        model: [sonnet-5]
        effort: [medium]
        options:
          agents_md: [default, none]
      - tool: claude-code
        model: [sonnet-5]
        effort: [medium]
        pre:
          - rm -rf /workspace/CLAUDE.md /workspace/.claude /workspace/.githooks /workspace/.headwater

The built-in option `agents_md: none` does not state what it removes. `.headwater/probe.yml` records that the absent arm removes four paths. So only the `pre` arm is the declared ablation. `tools/probe/ablate.sh` cannot run as that command, because it refuses a workspace that is its own checkout. So the four paths are a second copy of the list in `ablate.sh` and `.headwater/probe.yml`.

**The grading is a `patched` oracle.** The hidden tests pass or fail on the patch that the session produced. This matches the `patched` form of spec 5 and the Sufficiency category. The other four forms need the tool calls, and BERBench does not grade them.

**Whether its agent log reaches a transform by a safe channel is not known.** BERBench stores "execution evidence" for each cell outside the repository. Its documentation does not say how it captures the agent log, and its source is not public. A paid run is the only way to read that evidence, and this evaluation made none. So a BERBench run can supply a `patched` verdict, but its log cannot yet supply a transcript under spec 15. The step that fails is the channel, and not the arms.

## C. Public benchmarks for each probe category

The probe shelf holds 8 probes. By category, it holds 3 Discovery, 3 Navigability, 2 Sufficiency and 0 Consistency. By expectation, it holds 4 `opened`, 1 `not_opened`, 1 `cited`, 1 `answered` and 1 `patched`.

| Category | Corpus with a matching scenario | Expectation form of its oracle |
|---|---|---|
| Discovery | None. RepoBench-R retrieves code snippets for a completion and not a governing document. #88 writes these probes. | none |
| Sufficiency | SWE-bench Verified. Its tests that must change from fail to pass are an oracle over a patch. | `patched` |
| Navigability | None. No corpus of the five pairs a code path with the document that governs it. #88 writes these probes. | none |
| Consistency | None. LongBench, DocBench and OfficeQA grade one answer against a reference, which is a rubric or string match over prose. #88 writes these probes. | none |

**Issue #1015 names "the `patched`, navigation, and discovery probe categories".** `patched` is an expectation form and not a category, so the table above has one row for each of the four categories. It names the form that each oracle supplies.

SWE-bench Verified supplies a scenario and an oracle, and it supplies no corpus. Its repositories carry no governed documents, so the present arm and the absent arm of a SWE-bench task differ by nothing. A SWE-bench task measures the corpus only after a person adds a corpus to the repository. BERBench on this repository does not have that gap, because the corpus is already in the start tree.

## D. The adapter interface

Section A recommends no runner, and section B recommends none either. So this section defines no adapter. The seam stays `tools/probe/probe-transform.sh`, which reads a harness session log on standard input. Scenario definitions stay in `docs/probes/`, and nothing in them names a runner.

## What this leaves open

- [HW-OBL-0124](../obligations/0124-a-probe-result-is-printed-and-never-committed-so-nothing-regenerates-one.md) stays open. No runner in this evaluation supplies a recording that the first confirmation of spec 15 accepts. The shelf `docs/probe-runs/` holds 5 recordings, and that confirmation refuses each of them.
- The campaign tier in `.headwater/probe.yml` declares 58 repetitions and 2 arms at 25 cents a session, against a budget of 5000 cents. `headwater probe plan --tier campaign` refuses it: 928 sessions project $232.00 against a ceiling of $50.00. This evaluation does not change that number.
