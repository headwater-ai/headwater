---
id: HW-SPEC-harness-support
status: draft
status_since: 2026-08-25
summary: The ten capabilities a harness supplies at the four moments, stated once for every harness, and the recorded support of Claude Code, GitHub Copilot and OpenAI Codex.
last_verified: 2026-08-25
doc_type: design_spec
sequence: 16
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-fable-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-ai-integration
  governs:
    - .claude/settings.json
---

# Harness support

[Spec 5](05-ai-integration.md#the-four-moments) names the four moments at which a corpus serves an agent. [The hook contract](05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) states what a position passes, what it returns, and what it binds, which is nothing. Both are written against this engine and against no harness. What a particular harness offers at each moment is a fact about that harness, and it moves on the vendor's cadence. This part holds that record in one place, so that a change in a harness moves one table here and no sentence of spec 5.

The reader is a person who binds this engine to a harness, including a harness this repository does not run. The capability vocabulary is the half that holds still. The support table is the half that a vendor moves.

## What a harness is

A **harness** is the program that runs an agent against a checkout. It brokers three things, and every capability below is a claim about one of them. It decides what context the model reads without asking. It hands control to configured scripts at fixed positions in its loop, and takes it back. It keeps the register of what a person, or the model, invokes by name.

A harness is not one program per vendor. "Copilot" names an IDE integration, a terminal program, a hosted coding agent, a code reviewer and a completion engine. Each of the five supports a different subset of the table. A support claim therefore names a surface, and a claim that names only a vendor compresses real variance. The table below carries one column per vendor and states, in the notes, where the surfaces of one vendor disagree.

## The ten capabilities

Ten capabilities cover the integration this engine asks for. Three carry context to the model, three intercept the loop, three are invoked by name, and one is a protocol rather than a harness feature. Each entry states which moment of spec 5 it serves, and the terms the harness must meet. The terms are the hook contract's vocabulary: what the harness passes, and what it does with what comes back.

### Context: standing, scoped, injected

**C1 — standing context.** A file the harness reads whole into every session, with no action from the model. It carries the conventions and the one directive that [names a skill before any description competes](05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does). The terms: the file lives in the checkout, and the harness reads it at session start. It costs its size on every request, which is why spec 5 prices it against the subagent.

**C2 — scoped context.** Instructions attached only when work touches a matching path, which is [read-time rule loading](05-ai-integration.md#read-time-rule-loading). The terms: the checkout declares which file covers which paths, and the harness attaches on match. A harness offers this as a glob a rule file declares, or as a file per directory. A projection follows whichever shape the harness reads. Either way the attached files are generated projections of canonical documents under a declared size budget, which spec 5 requires. None of them is a second authored copy of a rule.

**C3 — injected context.** A position that runs when a prompt is submitted, whose output joins the model's context before the model acts. It serves [intent-time routing](05-ai-integration.md#intent-time-routing). The terms: the harness passes the prompt as text, and it returns standard output to the model verbatim. Silence is a result that costs nothing, because a wrong pointer costs more than a missing one.

### Interception: refusal, advisory, gate

**C4 — write refusal.** A position before a tool call that can deny the call with a reason the agent reads. It serves backfill at the [write moment](05-ai-integration.md#write-time-hooks): a raw write of a new document is refused, and the refusal names `headwater new`. The terms: the harness passes the tool name and the one path, and a denial carries prose. The harness gives that prose to the agent rather than to a log.

**C5 — write advisory.** A position after a tool call that adds context and blocks nothing. It serves impact detection. The posture is the substance: spec 5 keeps it advisory, because a blocking gate trains the reflex answer that destroys the signal.

**C6 — turn gate.** A position at the end of a turn that can refuse the end and hand the agent a report. It serves the [review moment](05-ai-integration.md#review-time-checks). A binding calls the commit gate itself, so what stops a turn and what stops a commit stay one file. The terms: a blocking verdict that carries the report, and a flag that says the gate already blocked this turn. A gate with no such flag is a loop.

### Invocation: commands, skills, agents

**C7 — commands.** A procedure a person invokes by name. The workflow commands of this repository are three: the build-order iteration, the stacked run, and the board review. The terms: a file per command in the checkout, and the harness offers it under the name the file carries.

**C8 — skills.** An instruction package the model selects by description, which spec 5 grades as [scent](05-ai-integration.md#scent-is-the-thing-being-engineered) and measures rather than trusts. The terms: a directory per skill in the checkout, a description the harness serves to the model, and loading on selection rather than always.

**C9 — isolated agents.** A scoped instruction set that runs in its own context and reports back, which is what keeps the always-on prompt small. The terms: a file per agent in the checkout, and an isolated context per run.

### The protocol surface

**C10 — tools.** The [MCP server](05-ai-integration.md#agent-surfaces) is the one surface that is not harness work at all, because the protocol is the point of the protocol. What varies per harness is registration. Either the checkout declares the server, so that every session of that repository gets it, or only the operator's personal configuration can. A checkout that cannot register its own server has the tools only where every operator has done the same work once each.

## What every binding owes, whatever the harness

Four terms hold for a binding to any harness, and the first three restate the hook contract at this altitude.

- **A binding calls a verb that ships, and carries no rule of its own.** Two entry points to one answer are two answers as soon as one drifts.
- **A binding fails open, even where the harness fails closed.** Copilot's documentation makes an erroring pre-tool hook deny the call. On such a harness the binding catches every failure of its own. A missing engine, or an unparseable input, must never deny an edit that the positions below already hold.
- **A binding binds nothing, and the position under it does.** The commit gate and the CI job hold every change, no cell of the table below reaches either, and a column prices earliness alone.
- **A fixture suite drives every position over recorded input, including every refusal.** `.claude/hooks/fixtures.sh` is the reference: a hook that has refused nothing is a hook that nobody has seen work.

## The support table

The Claude Code column is bound in `.claude/`. Two suites hold it: `.claude/hooks/fixtures.sh` for the positions, and `.claude/skills/fixtures.sh` for the skills. The Codex and Copilot rows for C3 through C6 bind the same way, in `.codex/hooks.json` and `.github/hooks/*.json`, and `.claude/hooks/fixtures.sh` holds their cases too. `.claude/hooks/fixtures-live.sh` reruns the live confirmation against a real install of each harness, and nothing gates on it, the same posture as `headwater probe`. The remaining cells of both columns are read from vendor documentation dated 2026-08-25, and no fixture holds them.

| Capability | Claude Code | GitHub Copilot | OpenAI Codex |
|---|---|---|---|
| C1 standing context | `CLAUDE.md` | `.github/copilot-instructions.md`, and it reads `AGENTS.md` | `AGENTS.md` |
| C2 scoped context | a `CLAUDE.md` in the directory it covers | `.github/instructions/*.instructions.md`, with an `applyTo` glob | an `AGENTS.md` in the directory it covers |
| C3 injected context | `UserPromptSubmit` hook | `userPromptSubmitted` hook | `UserPromptSubmit` hook |
| C4 write refusal | `PreToolUse`, deny with a reason | `preToolUse`, deny with a reason, and an erroring hook denies | `PreToolUse`, exit 2 with the reason on standard error |
| C5 write advisory | `PostToolUse`, added context | `postToolUse` | `PostToolUse` |
| C6 turn gate | `Stop`, exit 2 blocks the turn | `agentStop`, `decision: block` on the CLI and the cloud agent, and the IDE's `Stop` cannot block | `Stop` is documented, and whether it blocks is not |
| C7 commands | `.claude/commands/*.md` | `.github/prompts/*.prompt.md` | `~/.codex/prompts/*.md`, personal rather than repository configuration |
| C8 skills | `.claude/skills/*/SKILL.md` | `.github/skills/`, and it reads `.claude/skills/` and `.agents/skills/` | `.agents/skills/`, selected by name in the composer |
| C9 isolated agents | `.claude/agents/*.md` | `.github/agents/*.agent.md` | subagents in the CLI |
| C10 tool registration | `.mcp.json` in the checkout | a workspace file in the IDE, repository settings for the cloud agent | `[mcp_servers]` in `config.toml`, operator configuration |
| position registration | `.claude/settings.json` | `.github/hooks/*.json`, and the IDE also reads `.claude/settings.json` | `.codex/hooks.json`, or `~/.codex/hooks.json`, behind a feature flag that ships on |

Every Claude Code cell except C2 is bound. The Codex and Copilot rows for C3 through C6 are bound the same way, each held by `.claude/hooks/fixtures.sh`. `.claude/hooks/fixtures-live.sh` is the live confirmation behind that, and it reruns against a real install rather than resting on one session. Every other cell of both columns is a file this repository ships or a claim vendor documentation makes, and no fixture holds either kind.

**The shape converged, and it is the shape this repository already ships.** All three harnesses read a `SKILL.md` under a per-skill directory. All three name the same four positions, and all three read a standing file from the checkout. Two of them read this repository's own files: Copilot's IDE discovers hooks in `.claude/settings.json`, and its skill loader reads `.claude/skills/`. So part of the Claude Code binding is loaded by a second harness through that harness's own choice, which nothing here has measured. A binding for either other column is registration of the same verbs, not a port of any logic, because the scripts carry none.

**The posture inversion is the one hazard the vocabulary has to name.** Every position of this repository fails open, and one harness documents a pre-tool position that fails closed on error. The second term of the binding contract above exists for that cell.

**A second inversion held on the turn gate, and a live run is what found it.** Copilot's `agentStop` does not fail closed on exit 2. An exit-2 hook there is logged, and the turn ends anyway. The block that works is a `decision: block` object on standard output at exit 0. `.claude/hooks/review.sh` branches on the `COPILOT_CLI` environment variable for that one difference, and Claude Code and Codex keep the exit-2 path the hook contract states.

**The IDE differs from the CLI under one vendor name.** Copilot's completion surface reads none of this table, and its IDE cannot block a turn where its CLI can. Its code reviewer reads instructions from the base branch rather than the feature branch. The column records the most capable surface, and a person binding one surface reads the vendor's own reference for that surface.

**No column moves enforcement.** The table prices where a finding arrives, and the [hook contract](05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) already prices what that is worth. A refusal at write time costs one retry, and the same refusal at review time costs a rewrite. Whether any cell earns its context cost is a [probe](05-ai-integration.md#measuring-whether-any-of-this-works) question, no probe has run, and no cell of this table rests on one.
