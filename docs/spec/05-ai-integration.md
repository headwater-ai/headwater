---
id: HW-SPEC-ai-integration
status: current
status_since: 2026-08-01
last_verified: 2026-08-11
summary: How the corpus serves an agent at four moments, and how a probe measures whether any of it works.
doc_type: design_spec
sequence: 5
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: [claude-fable-5, claude-opus-5]
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  cites_evidence:
    - HW-EVAL-adjacent-work
    - HW-EVAL-the-measurement-layer
    - HW-EVAL-the-serving-boundary
    - HW-EVAL-theoretical-foundations
    - HW-EVAL-warrant-and-adjudication
  governs:
    - .claude/hooks/lib.sh
    - .claude/hooks/intent.sh
    - .claude/hooks/write.sh
    - .claude/hooks/review.sh
    - .claude/skills/fixtures.sh
    - .claude/skills/headwater-authoring/SKILL.md
    - .claude/skills/headwater-taxonomy/SKILL.md
    - .claude/agents/headwater-maintainer.md
---

# 5 — AI integration

Agents now read documentation more often than people do. The failure mode of an agent is not that it skims. When context is missing, an agent invents, and it invents with confidence. The corpus is designed for that reader.

## The four moments

An assistant touches documentation at four different moments. Each moment needs a different mechanism. Most "AI-ready docs" efforts do not separate the moments, and that is why they give less than they promise.

| Moment | Question | Mechanism |
|---|---|---|
| **Intent** | "What governs the thing I am about to do?" | Routing — resolve a task description to governing documents *before* the agent reads any file |
| **Read** | "What rules apply to this file?" | Path-scoped rule loading, started when the file opens |
| **Write** | "Did what I just changed invalidate a document?" | Impact detection on edit, and metadata backfill on creation |
| **Review** | "Is this change consistent with the corpus?" | Change-scoped checks, and agent review against the governing set |

### Intent-time routing

This is the highest-value moment, and it is the one most often missed. An agent that reads the governing standard *before* it writes code produces different code. An agent that finds the standard afterwards produces an apology.

Routing matches the **declared purpose** of each kind against the intent of the task before it matches any text. "why is it like this?" resolves to kinds that serve `rationale`, and "what does it do?" resolves to kinds that serve `behavior`. That is a search over the corpus's intentional structure, not over its prose. It is cheaper and more precise than lexical ranking, and it degrades gracefully because purposes are a small closed set. Lexical ranking then orders results *within* the matched purpose, and derived reading precedence breaks ties ([spec 2](02-taxonomy-model.md#reading-precedence-is-derived)). Where two linked documents both match, routing offers a nucleus before its satellite. It offers a successor before the document that it superseded, and a governing document before the one that it governs.

In all other respects, routing is a deterministic projection over the graph — summaries, facets, relations, and code-path anchors — not a semantic search. A task description resolves to a ranked set of **pointers**: a path, the name the document declares, a one-line summary, and never content. The budget caps the ranked list. It does not cap the documents that govern a path the task names, because a route names those and does not rank them. A route reports how many ranked pointers the budget withheld, so a reader can tell three answers from three of fifteen. Pointers keep the corpus as the single source of truth and keep the context cost near zero.

Routing is **confidence-gated and fails open**: below the threshold it says nothing. A wrong pointer costs more than a missing one, because an agent will follow it.

Silence for weak scent and silence for a withheld document are different facts, and routing keeps them apart. Where a route runs over a filtered [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter), a document that the filter removed is reported at the profile's declared tombstone grain. It never falls under the confidence gate, because nothing about it is uncertain.

A third case joins those two. A pointer to a document with the `asserted` [warrant](01-conceptual-model.md#warrant) states that warrant beside the summary. Nobody accepted the document, and an agent that follows the pointer has to know that before it reads. To offer such a pointer silently is the failure that [Q15](09-decisions.md#q15--a-synthesized-content-tier) exists to prevent, reached through our own routing surface.

#### Scent is the thing being engineered

Information-foraging theory names what routing actually trades in: **scent** — the proximal cue that predicts distal value. A reader or an agent follows scent, and abandons a patch when the scent weakens. Thus scent quality, not corpus quality, decides whether anything is found. A perfect document with a vague summary is invisible.

Scent has two surfaces, because a reader arrives at a document in two ways. A routing result answers a task description, and no referring edge exists, so the cue has to sit on the node. That cue is the `summary` facet. A reader who follows a relation meets the referring text first, and the target's summary is distal from there. That cue is an optional **cue** attribute on the relation instance, which the referring document writes ([spec 2](02-taxonomy-model.md#instance-attributes-and-which-end-owns-each-one)).

One rule grades both, and it comes from the theory rather than from convenience.

> **Grade a cue against the alternatives that it competes with at the moment a reader reads it.**

Scent is never absolute in foraging theory. It is a comparison over the options at the point of decision ([HW-EVAL-theoretical-foundations §E.1](../evaluations/theoretical-foundations.md#e1-information-foraging--routing-has-a-theory)). So each measure below names its comparison set, and the set is what changes between the two placements.

| Measure | Comparison set | Catches |
|---|---|---|
| **Distinctiveness** (summary) | the shelf siblings, as a static proxy for the pointer list | A summary that shares no discriminating term with its siblings cannot separate them |
| **Distinctiveness** (cue) | the other outbound cues of the same document | A cue that does not separate this link from the others that the document offers |
| **Non-restatement** (summary) | the document's own title | A summary that only rephrases the title carries no information that the path did not |
| **Non-restatement** (cue) | the target's summary and title | A cue that rephrases the fallback carries nothing that the fallback carried |
| **Length band** | the declared band | Too short to discriminate, or too long to scan |
| **Routing precision** | the pointer list that the query returned | Of the pointers offered, how often the agent opened the top one, and it sufficed |
| **Abandonment** | the same list | Pointers offered and never opened, which is scent that promised and did not pay |
| **Traversal precision** | the edges reachable from an opened document | Of the edges available, how often the agent followed one whose target sufficed |
| **Traversal abandonment** | the same edge set | Edges offered and never followed |

The static measures run as advisory checks, and they stay advisory permanently, because the remedy for a weak cue is a rewrite ([spec 4](04-assurance-model.md#where-promotion-cannot-finish)). Distinctiveness over cues is `Document`-scoped and non-restatement is `Edge`-scoped ([spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on)). The four behavioral measures come from probe transcripts. A transcript is the only place where the corpus observes foraging behavior rather than infers it.

A shelf is a proxy and the pointer list is the thing. A static check reads the proxy, because a query-dependent set is not available at check time. A probe measures the list itself. The two measure one property at two fidelities, and the specification does not pretend otherwise.

The confidence gate is a scent threshold. The engine stays silent when the strongest available cue is weak, because a cue that misleads is worse than a missing cue. That is the same asymmetry stated in foraging terms, and it is why the gate errs toward silence.

#### What a cue may do, and where it is served

The cue is optional and the summary stays required, so a corpus that declares no cue behaves as it does today ([Q20](09-decisions.md#q20--where-scent-lives)). Five rules govern it, and four of the five follow from rulings that already exist.

- **`related` and `explain` serve the cue where one exists, and the target's summary otherwise.** Routing never serves a cue, because a routing result has no referring edge.
- **A cue states the warrant of its target.** A cue that points at a document with the `asserted` [warrant](01-conceptual-model.md#warrant) says so beside the cue, for the reason that a pointer does.
- **The confidence gate does not reach a cue.** The gate is a threshold over a score, and an author writes a cue rather than the engine scoring it. Silence is not available on a traversal either, because the reader already holds the document and can see the edge. So the fallback is the summary and never nothing.
- **A withheld target takes its cues with it.** A cue is an edge attribute, and an [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter) filters attributes by class under default-deny. A tombstone carries a rule identifier and never a cue.
- **Nothing makes a cue mandatory.** A required prose field on every edge is the capture cost that [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) says kills a corpus. A hand-authored cue also counts as hand entry, so the assisted fraction reports the tax if there is one.

```
headwater route "add rate limiting to the ingest API"
  docs/standards/api-design.md      — API surface conventions, versioning, error shapes
  docs/specifications/ingest/...    — ingest service behavior and SLOs
  docs/decisions/dr-0031.md         — why throttling is applied at the edge, not per-service
```

### Read-time rule loading

Rules load additively when an agent opens a file that matches. They are **generated projections of canonical documents**, not a parallel copy. A rule file is a short pointer to the standard that it derives from. It carries only enough content to make the agent stop and read the source. CI checks the regeneration, so a standard and its rule cannot disagree.

Rules obey a **size budget** declared on the kind or projection that produces them. The engine meters always-loaded context on every request, so it enforces budgets per file and in aggregate. An over-budget rule is a finding that names the layer that must trim it. The budget is not optional: an agent-facing projection with no applicable budget fails `taxonomy validate` ([spec 2](02-taxonomy-model.md#the-meta-schema)).

When a budget binds, the engine **drops satellites before nuclei**. A generated rule is a satellite of the standard that it derives from. To drop the standard and keep its teaser is exactly backwards, and without declared nuclearity the engine has no basis for the choice. Relation nuclearity makes context pruning a structural operation rather than a heuristic one.

### Write-time hooks

- **Backfill** — at creation time, when it is cheap, the engine completes the front matter, identifier, and required sections of a new document against its kind. `headwater new` is that completion. It derives the shelf, the facets, the identifier and the sections from the committed lock, and it refuses rather than guessing. So the hook at this moment refuses a raw write of a document that does not exist yet, and names the verb. It refuses creation alone, because an edit to a document that already carries front matter is what [`check --fix`](12-check-layer.md#fixability) and the commit gate hold.
- **Impact detection** — a document can declare that it governs code. An edit to that code raises an advisory prompt that names the specific documents at risk. The prompt names each document by its path and by the declared name, which for an `interface_contract` is the command a caller types. The prompt is advisory on purpose: a blocking gate here trains people to write "no doc impact" reflexively, and that destroys the signal. `governing_docs_for_path` answers by equality against the string each edge reached, so a document reaches the paths it names and no path under one of them. [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) holds what an author pays for that.

### Review-time checks

The same engine runs, and it reads the whole corpus. [Spec 6](06-engine-architecture.md#performance-targets) refuses a flag that takes a caller's list of changed documents. Such a flag puts a second input into a verdict that no reviewer sees. The content-addressed cache pays for the position instead, and [HW-OBL-0080](../obligations/0080-changed-only-is-the-content-addressed-cache-under-another-name.md) is the record that measures it.

The measurement over this repository, at 166 checked documents and a warm cache, is 54 ms for `headwater check --strict`. Spec 6 allows 200 ms at this position. A cold run costs 234 ms, and `headwater route` costs 35 ms. So the budget holds at this corpus size, and the cache does the work that no flag has to.

Findings reach a reviewer in the vocabulary the reviewer reads. `--format sarif` is what a forge ingests as a check run, and `--format markdown` is a job summary or a review comment. Each finding carries its remediation, and the [fixability](12-check-layer.md#fixability) bar decides which ones carry a patch as well.

### The hook contract, and what a hook cannot bind

A hook is a position where a harness hands control to this engine and takes it back. Four terms fix it. The fourth decides what the other three are worth.

**What the harness passes.** The smallest fact that the moment holds. At intent, the task description as text. At write, one path and the tool that is about to touch it. At review, nothing at all. A hook never passes content and never passes a list of what changed, because the engine reads the checkout itself.

**What the engine returns.** An exit status and two streams, which is the contract every verb already has. Standard output carries the one artifact the harness feeds back to the agent. Standard error carries the account of what the engine did. The status carries the verdict.

**No hook introduces a verb.** A `headwater hook <moment>` verb is a second entry point to `route` and to `check`. Two entry points to one answer are two answers as soon as one drifts. Each position therefore calls the verb that ships. The review position goes further and calls the commit gate itself. What stops a turn and what stops a commit are then one file rather than two that agree today.

**What the harness does with a refusal.** It stops the action and gives the agent the reason. A refusal names the offending file and the verb that repairs it. A hook that cannot decide returns nothing and lets the action proceed, because the positions below a hook already hold the result.

**What happens when the harness ignores the hook.** Nothing happens, and this is the term that governs the design.

- A harness hook is configuration in the reader's own tree. One setting turns every hook off, and no record of that setting reaches the repository.
- A harness that is not this one never had the hooks at all.
- A write-time refusal matches a tool by name. The same bytes written through a shell command reach no hook, and the session gate that this repository retired recorded the same bypass about itself.
- The commit gate is skipped with `git commit --no-verify`, which leaves no mark on the commit.

**So a hook binds nothing, and the position under it does.** The CI job runs on the pull request, and the author of a change cannot turn it off. Every position above CI buys earliness rather than enforcement, and the two are worth different things. A refusal at write time costs one retry. The same refusal at review time costs a rewrite of finished work. That is the capture cost that [spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) names as the thing that kills a corpus.

**A session in which no hook spoke is therefore no evidence.** No harness hook is a control in the [assurance model](04-assurance-model.md), and the reason is the position rather than a judgment about severity. Spec 4 counts the commit gate and the CI job, and a change cannot leave either one.

**One asymmetry runs the other way, and it is the argument for a harness hook.** Git installs no repository hook by itself, so a commit gate needs one command in every clone. A harness hook loads when the repository opens. The weaker position therefore installs itself and the stronger one does not, and a first change in a fresh clone meets the harness hooks alone.

**A hook is invisible or it is bypassed.** That is spec 6's performance argument read at this position, and the measurements above are what hold it.

**A hook calls a verb and never a skill.** Each moment has a deterministic half and a judgment half, and a hook reaches the first one. The judgment half is an [authoring skill](#agent-surfaces), and [how a skill reaches an agent](#how-a-skill-reaches-an-agent-and-what-nothing-does) is a separate question with its own answer below.

The hooks of this repository are `.claude/hooks/`, which sits outside the corpus root. No census counts them and no check reads them. `.claude/hooks/fixtures.sh` is what holds them, and it provokes every refusal on purpose. A hook that has refused nothing is a hook that nobody has seen work.

## Agent surfaces

Beyond files, there are three richer surfaces:

**The corpus MCP server.** The server exposes the graph as tools that an agent calls directly: `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check`. This is strictly better than to make an agent grep a corpus that it does not understand. The graph already knows the answers, and a tool call returns them at no context cost for exploration.

#### What the server may do, and the axis that decides it

"Read-only or not" is the wrong question, and [Q7](09-decisions.md#q7--scope-of-the-mcp-surface) asked it for a while. `headwater check --fix` writes files today, in a human's working tree, and the result lands in a diff that the human commits. A hosted server that commits to a branch produces the same bytes with no review at any point. The axis is **whose review the result passes through**, not whether bytes move.

| Class | Tools | Ships | Why |
|---|---|---|---|
| **Query** | `route`, `governing_docs_for_path`, `resolve_identifier`, `related`, `explain`, `check` | first release | It changes nothing |
| **Working-tree write** | `new`, `fix` | first release, and off by default per server ([what the switch registers](#what-the-working-tree-write-class-registers-and-what-a-session-looks-like-after-a-write)) | The human reviews at commit, and the [fixability bar](12-check-layer.md#fixability) forbids a judgment-bearing patch |
| **Landed write** | a commit, a push, a merge, a server that writes to a repository | never | Acceptance is a human act ([spec 3](03-authoring-and-lifecycle.md#provenance-is-recorded-not-assumed)), and no forge is privileged in the core |

The third row is a refusal and not a deferral. A server-side commit produces a document with no `accepted_by`, or with an invented one. The provenance model forbids it before any judgment about trust arrives. Headwater instead emits what a change proposal needs: findings, patches, and a task list. An adapter opens the proposal, with the credential that its operator granted it. That is the same boundary that keeps the engine out of the merge-queue business ([Q21](09-decisions.md#q21--terminological-succession-and-validity-under-merge)).

Working-tree writes stay off by default, because a client may connect to a checkout that the user did not intend to change. The opt-in is per server, and `headwater mcp --write` is it.

**The annotation is not the enforcement.** The protocol lets a server declare that a tool only reads. It also states that a client must not treat that declaration from an untrusted server as a guarantee. Headwater annotates its tools correctly and relies on something else. Where a class of tool is off, the server does not register it, so no handler exists to call. A property that a caller reads off a tool list is a hint. A property with no code path behind it is a guarantee.

**A write tool is a disclosure channel, and that is why the third row is a refusal rather than a preference.** The published attacks on this protocol put attacker text into a model's context at discovery time, before any tool runs. A confirmation prompt at each call therefore never sees them. What the attacks then need is an actuator. The observed case against a widely deployed server used a write tool as the exit. The agent read private content, then published it by opening a proposal on a public repository. A server that cannot land a write lends an injected instruction nothing. That argument is about [Q17](09-decisions.md#q17--governed-access-and-the-solution-layer) as much as about this entry.

**The server applies no filter to a corpus that its reader already holds.** It runs in-process against a checkout, so the reader has every byte. A filter there would control one reading path while the bytes stay readable along another. [HW-EVAL-adjacent-work §L.6](../evaluations/adjacent-work.md#l6-a-filter-in-the-tool-layer-is-advisory-and-the-documentation-says-so) records that failure in a shipped tool. A server that serves a reader who holds no checkout serves exactly one declared [export profile](06-engine-architecture.md#an-export-profile-carries-a-filter) and never mixes the two sources.

**A hook binds nothing, and a tool call is a choice.** The [hook contract](#the-hook-contract-and-what-a-hook-cannot-bind) ends on the term that one setting turns every hook off with no record. A tool has no such term to lose. A harness that calls a tool has already chosen to call it. A harness that calls none is in the state the hook contract calls no evidence. So the two surfaces fail in opposite directions. A tool is weaker at the moment nobody asks, and stronger at the moment somebody does. Neither is a control in the [assurance model](04-assurance-model.md), and the commit gate holds both positions from below.

What that changes is where the effort belongs. A hook earns its cost by reaching a moment the agent did not think to ask about. A tool earns its cost by answering the question the agent did ask, in full, with no second answer anywhere. That is why the tool list is closed, and why the three decisions below are settled here rather than at a call site.

#### What a `check` tool decides, and where each decision is taken

`check` is the one tool of the query class that runs the check layer rather than reading the graph. It needs a clock, a cache and a corpus walk. The CLI takes each of those at its own boundary. A tool that took them again would put a second set of defaults behind a protocol. So the tool takes none of the three, and every one arrives from whoever started the server.

**The clock is fixed at start-up, and stated in every result.** `headwater mcp` reads `--now` where the operator gives one, and the system date otherwise. That is what `headwater check` reads. A host that cannot say what day it is gets no server. Two other answers were available and both are worse. A date argument on the tool lets a caller pick the value that answers the way it wants. It also spends the tool's one argument on the input a caller is least fit to choose. A refusal to answer a windowed question makes the tool useless for the one rule that reads the clock. That is the rule an author most needs a reminder about. So the server holds one date for its whole life, and a caller that wants another date starts another server.

**The tool uses no cache, because a cache write is a write.** The cache is a store in the checkout, and a run that used it would put bytes back. [Q7](09-decisions.md#q7--scope-of-the-mcp-surface) rules that a class of tool that is off is not registered. So the property that no tool of this server writes is a code path rather than a promise. A cached read tool would end it. A cache that read and never wrote is a third mode no other caller of this engine has. That is the same second set of defaults, reached by another road. What makes the refusal free of consequence is the invariant [spec 12](12-check-layer.md#determinism-concretely) already carries. A cached run and an uncached run write the same bytes, so the cost is time and never an answer.

**The corpus is walked once, before the server accepts a message.** Every tool answers from that walk, so `check` costs no walk that `route` does not. The measurement over this repository is 199 to 222 ms for one call. It is flat across the four formats and across repeated calls. The warm CLI run costs 54 ms and the cold one costs 234 ms. So one call costs a cold run, which is what an uncached run is.

**The answer is about the whole corpus, and the tool takes no path.** Coverage is computed against the census as its denominator. A report filtered to a path carries one of two denominators. It carries the whole census with fewer findings under it, or a smaller count that no run evaluated. Both are a second answer about one corpus.

**The one argument is the output format, and it has no default.** [Spec 6](06-engine-architecture.md#cli) closes that set at four. One dispatcher writes all four, for the CLI and for this tool alike. So the answer is `headwater check --format <name>` byte for byte. The argument is required, because a default chosen inside the tool is a decision taken at a call site. That also settles the question a richer argument would have opened: one string is enough, and no second tool is needed.

#### What the working-tree write class registers, and what a session looks like after a write

The second row ships as `new` and `fix`, off by default. The switch is `headwater mcp --write`. A working-tree write tool is the read above with a verb behind it. Every decision that verb needs is taken where the terminal takes it. The tool holds the two verbs as functions rather than as parts. So a call runs the verb that ships, and never a second composition of a scaffolder.

**The opt-in is a word somebody typed.** Two other shapes were available and both are worse. A setting in the checkout would grant the class to every client that opens the repository. The person who started the server would then have said nothing, and the consent would be a fact about somebody else's commit. A second verb would be a second server to hold in step with this one, and [spec 6](06-engine-architecture.md#cli)'s grammar names verbs rather than modes. So the switch sits beside `--now`. Those are the two inputs a caller is answerable for.

**A tool takes the arguments its verb takes.** One string was enough for every read. It is not enough for `new`, which takes a kind, a title and a repeatable relation. A relation is written `<relation>=<identifier>`, which is the form the terminal takes. `fix` takes a format and nothing else. Neither takes a clock, because the server holds one for its life. Neither takes a root, because the checkout is the one the server started over. Neither takes a path, for the reason `check` takes none.

**A call that moves a byte of the checkout ends the server.** The corpus is walked once, before the server accepts a message, so a write ends the tree that walk described. [HW-OBL-0028](../obligations/0028-a-run-cannot-report-the-corpus-tree-because-nothing-computes.md) measures what a stale walk costs. A server that wrote and then kept answering would be that measurement, with the server as its own cause. So every later call is refused, and the refusal names the tool that ended the session. "A caller that wants another tree starts another server" is what this section already says about the clock. Here it is a code path rather than advice.

Two halves of that rule matter equally. **The seal follows the bytes and never the call**, so a `fix` that found no patch leaves the session open. A scaffolder that refused leaves it open for the same reason. **The writing call itself answers from a fresh walk**, because the verb re-reads the tree it wrote before it reports. So the one answer that could be stale is the answer such a server never gives from the old walk.

**A tool result carries the two streams the verb wrote.** Standard output is the artifact and standard error is the account, which is the [hook contract](#the-hook-contract-and-what-a-hook-cannot-bind)'s term at this position. A protocol call has one result. So the two arrive as two content blocks, and the artifact block is the bytes a terminal reads.

#### What replaced "the server registers no tool that writes"

That sentence was a property of the code, and four narrower ones stand in its place. Each is a code path, and each is asserted in the suite rather than promised here.

- **A server with no switch is the server that was here before.** The tool table holds the six reads, and no handler that writes exists to reach. That is the shape a client meets when it connects to a checkout that nobody meant to change.
- **The landed-write class is reachable by no argument.** The third row is a refusal rather than a deferral, and no flag turns it into anything. That is the row the disclosure argument rests on. An injected instruction needs an actuator, and a server that cannot land a change lends it none.
- **Every write leaves a record in the tree a human commits.** `new` appends a capture-cost reading that names the protocol as the surface of the run. The reading and the document then arrive in one diff. `fix` writes only what a finding derived under the [fixability bar](12-check-layer.md#fixability), and it answers with the run after the write. So a patch that produced a document the checks reject reports it in the same answer.
- **No write tool accepts a document.** `headwater new` writes no `accepted_by` and no `accepted` warrant, so neither does the tool. [Stop rule 5](#the-stop-rules) forbids an agent from stamping its own output. [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) measures how often an agent does it anyway. This surface adds nothing to that count, because the verb behind it has no field for the stamp.

What none of the four does is stop an agent from writing a bad document. A scaffolded document is an authored document from the moment it lands, and every check reads it. The commit gate and the CI job hold it from below. That is the position the [hook contract](#the-hook-contract-and-what-a-hook-cannot-bind) ends on, and the axis at the head of this section is why it is enough. The result passes through a human's diff.

**Authoring skills.** Packaged procedures for the work that bears judgment: to draft a decision record, to run a corpus-wide sweep, to propose a taxonomy change. Skills carry the doctrine that an agent needs, and they call the deterministic engine for everything mechanical. Thus the LLM does the reasoning and never the arithmetic. This repository ships three of them, in `.claude/skills/`. `headwater-authoring` owns a document and `headwater-taxonomy` owns a declaration, which is the same boundary that [stop rule 3](#the-stop-rules) draws. `headwater-sweep` owns the reading of a slice, and its work is the one no engine performs. The [sweep](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) has three parts. The engine holds the first and the third, and the skill is the second.

This is also the system's answer to the capture-cost problem that killed every prior design-rationale tool ([spec 3](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric)). An agent can draft a decision record from evidence that already sits in the commit, the ticket, and the conversation. That moves the cost off the author, who was never the beneficiary. The claim is falsifiable, and it is tracked as the assisted fraction. If we enable agent authoring and the fraction does not rise, the argument fails, and we must then expect the adoption curve of gIBIS.

**A maintainer subagent.** A context-isolated agent that owns documentation upkeep across a change: which documents this touched, what is now stale, what decision lacks evidence. It runs with a scoped instruction subset — its own bounded context — and does not inflate the always-on prompt of every session. `.claude/agents/headwater-maintainer.md` is that agent here, and its report carries a fourth part for what it could not decide. A report with no such part is a report that nobody can calibrate.

#### How a skill reaches an agent, and what nothing does

A hook cannot make a skill load, and the [hook contract](#the-hook-contract-and-what-a-hook-cannot-bind) states the reason. This repository once ran the design. A `PreToolUse` gate refused an edit until a named skill had loaded, and it retired with the linter it served. A refusal at that position catches the sessions that would have complied and misses the ones that would not.

**A description is the mechanism that a harness supplies.** A harness reads the description of each installed skill, and a model picks one. That act is the routing act of [intent time](#intent-time-routing), performed by the model rather than by this engine, over a list the harness supplies. So a skill description is a summary in the [scent](#scent-is-the-thing-being-engineered) sense, and the same two measures grade it. Distinctiveness runs against the other installed skills, and non-restatement runs against the skill's own name.

**One mechanism is stronger than a description, and it is not free.** An instruction file that a harness always loads may name a skill and say when to use it. That is a directive rather than a cue, and it reaches the model before any judgment about relevance. It also costs context on every session, and most sessions touch no document at all. This repository pays that cost in `CLAUDE.md`, for `ste-editor` and for the two skills above. The cost is the always-on prompt that a [subagent](#agent-surfaces) exists to avoid, so the two surfaces trade against each other rather than stack.

**Routing cannot carry a skill, because a skill is not a document.** A skill sits outside the corpus root, so no census row covers it, and `headwater route` offers pointers to documents alone. A skill that moved inside the corpus would put prose that a harness executes under the language regimes. It would also give the shelf model a document that no reader reads as one.

**So the reach of a skill is a measurement rather than a property.** The `opened` [expectation](#a-probe-is-a-document-with-a-declared-expectation) over a transcript is the instrument that says whether a description won. No probe has run, so nothing about the reach of a skill is known, and no claim about the [assisted fraction](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric) rests on one.

**What holds a skill is what holds a hook.** `.claude/skills/fixtures.sh` drives every claim that a skill makes about this engine against the engine. Half of its cases are derived from the skill files rather than listed. So a rule that a skill names and this engine drops fails the suite with no edit here. It runs as a blocking CI step, for the reason the hook suite does. A skill that describes an engine that moved under it produces a confident wrong answer.

**A skill carries no rule of its own.** Every mechanical statement in one is a call to a verb that ships. A skill that copied a taxonomy rule would be the second authoring surface that [#130](https://github.com/headwater-ai/headwater/issues/130) refused, in a harness directory rather than a package.

### The machinery is the adoption model

These surfaces read as conveniences, and they are not: they are the supply chain for the graph. Everything distinctive — routing, lifecycle propagation, impact detection, absence findings — degrades together when edge density stays low. The traceability literature that the design cites ([HW-EVAL-theoretical-foundations](../evaluations/theoretical-foundations.md#b5-traceability-information-models--our-idea-has-a-name-and-a-literature)) says that author-maintained links decay because the payer is not the beneficiary. A release can omit the authoring skills, the hooks that put this engine at intent, write, and review time, and the telemetry that watches both. No hook invokes a skill, and the section above says what does. Such a release leaves its central bet untested. That is why [spec 0](00-vision-and-scope.md#what-we-build) ships them alongside everything else.

Three commitments make the machinery governable rather than merely present:

- **Judgment stays human.** Skills propose kinds, relations, summaries, and lifecycle changes from the artifacts already in the change. A human accepts. A skill that cannot justify a classification refuses it and does not guess ([the stop rules](#the-stop-rules)).
- **Every required datum is attributed.** `created_by` and the assisted fraction say whether the machinery actually maintains the graph. An assisted fraction that *falls* is an assurance finding — evidence that the adoption model fails — not a dashboard curiosity.
- **Generated context is checked before it is trusted.** Agent-facing projections carry freshness policy and size budgets like anything else, and the engine verifies them before they load. Stale governance instructions delivered efficiently are an efficient source of wrong behavior.

## What structured knowledge buys

The reason to build any of this is machine behavior that is more predictable. We must state the claim accurately, or it will shape the work wrongly.

**A governed corpus does not make a language model deterministic.** Sampling is stochastic. Identical prompts produce different outputs, and no amount of schema changes that. Anyone who claims otherwise has something to sell.

What it does buy:

| Mechanism | Effect |
|---|---|
| **Ambiguity removal** | Fewer legitimate readings of the input, so fewer defensible-but-divergent outputs. Variance narrows. It does not vanish |
| **Oracles** | We can check output against a declared expectation ([spec 2](02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle)) rather than judge it by eye |
| **Attribution** | When output deviates, the artifact that licensed it is identifiable, so the fix lands on the corpus or the prompt rather than on a hunch |
| **Reproducible comparison** | A pinned corpus and a pinned model give a baseline, and we can diff later runs against it |

The achievable target is **bounded, auditable non-determinism**: output that varies within a space that the corpus defines, deviations visible, causes attributable.

That is not a lesser goal, and it sets the investment priority. Effort belongs in oracles and traceability — checkable expectations, and citation of what licensed each decision. Effort does not belong in prompt engineering that tries to coax a model to repeat itself. The first compounds and is measurable. The second is a treadmill.

### Generated artifacts cite what licensed them

Any artifact that an agent produces under the corpus's direction cites the identifiers of the artifacts that governed it. Examples are a document, a generated test, and an implementation written against a specification. The citation is inline, at the point of the decision.

```python
# per REQ-INGEST-014 (specifications/ingest/parser/functional.md)
# version 5 and 6 swap these two fields
```

This is cheap, and it survives refactoring better than a link in a commit message. It changes "why does the code do this?" from an investigation into a lookup. It is also what makes attribution work in practice. When generated output is wrong, the citation says whether the corpus misled the agent or the agent ignored the corpus. Those have opposite fixes, and without the citation no one can tell them apart.

Where sources conflict, the agent cites **both** and flags the conflict ([spec 2](02-taxonomy-model.md#disagreement-is-adjudicated-not-ranked)). If an agent resolves a contradiction silently, it destroys the evidence that one existed.

Where a human already settled the conflict, the agent cites the **adjudication**. A settled disagreement is a decision that `overrides` the document whose effect it displaces, and derived reading precedence puts the successor first. So the agent follows a ruling that a named person made, rather than making the same ruling again with no record ([Q18](09-decisions.md#q18--recording-adjudicated-disagreements)).

## The stop rules

These are explicit behaviors that an assistant who works in the corpus must show:

1. **No decision record without external evidence.** If no work item, commit, discussion, or measurement supports it, halt and ask. If the human confirms that none exists, record the document as `unevidenced` ([spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)). Never invent rationale — a fabricated *why* is worse than an admitted absence, because someone will cite it.
2. **No hand edits to a generated file.** Change the source and regenerate.
3. **No new shelf, kind, or facet invented in place.** Structural change is a taxonomy change: propose it, version it, migrate it.
4. **No duplication of a fact that exists elsewhere.** Link. If the target is hard to find, fix the routing — do not copy.
5. **No self-acceptance.** Self-acceptance is a document that reaches `main` with no human review between the draft and the merge. Acceptance is the merge onto `main` ([HW-DR-0034](../decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md)). An agent may write `accepted_by` on a document it drafts. It may also change a [warrant](01-conceptual-model.md#warrant) to `accepted`. Both hold where the document goes into a pull request that a named human reviews before the merge. Where no human will read the document before the merge, the agent marks it `asserted` and writes no `accepted_by`. Nothing then stands between the draft and `main`. A stamp that reaches `main` unread is the mark that Serena's onboarding pass never wrote, with a false name attached ([HW-EVAL-adjacent-work §L.5](../evaluations/adjacent-work.md#l5-onboarding-ships-the-synthesized-tier-and-marks-nothing)). [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) holds the count that raised this ruling.

These are the rules that a capable model breaks most readily under pressure to be helpful. That is exactly why we state them as stop conditions rather than preferences.

## Measuring whether any of this works

Instruction files are written on the assumption that the assistant reads and follows them. That assumption is testable. When it is not tested, it is usually optimistic.

A **probe suite** runs scenarios against the corpus in a controlled session. It grades behavior from the tool-call transcript, not from the model's self-report. The grader re-derives every verdict from what the agent actually opened and did.

#### A probe is a document with a declared expectation

The grader constraint below needs something to bind, so the specification says what a probe is. A **probe** is a document in the corpus, with a kind, a shelf, an identifier and an acceptance. It declares a category, a task statement, and an **expectation**. The expectation is a predicate over the run record, and its forms are a closed set.

| Form | Satisfied when |
|---|---|
| `opened` | The transcript shows that the session read one of the named documents |
| `not_opened` | It read none of them |
| `cited` | A produced artifact cites one of the named identifiers ([above](#generated-artifacts-cite-what-licensed-them)) |
| `answered` | The final answer is one named value from a closed set that the probe declares |
| `patched` | A produced patch passes a named check, which is the oracle route ([spec 2](02-taxonomy-model.md#contract-sidecars-the-specification-as-oracle)) |

**Two of the five forms name something beyond the predicate, and each names it in a different place.** A `patched` probe names its oracle in a facet, and every probe of another form writes the sentinel `none` there. An `answered` probe declares its closed set of values in a fenced block under the `Expectation` section, which the kind already requires. The difference is the shape of the value. A facet of this taxonomy language holds a scalar, and a closed set is a list. The route matters beyond this section: a required section carries a per-document declaration where a facet cannot ([HW-OBL-0123](../obligations/0123-a-facet-that-applies-to-one-value-of-another-facet-has-nowhere-to-say-so.md)).

**A question whose answer needs a rubric is not a probe.** It is a coherence question, and the [sweep](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) owns those. The sweep reports findings rather than verdicts, and it is marked as agent-provenanced.

That rule is what makes the grader constraint a property rather than a promise. A predicate over an event log needs no model, so the grader holds none. This is the shape that [Q7](09-decisions.md#q7--scope-of-the-mcp-surface) used for the write path. A guarantee is a code path that does not exist, and not a declaration.

**A self-report and a produced output are different things.** A self-report is the agent's account of its own process, and no probe accepts one. A produced output is the artifact that the task asked for, and a declared expectation may read it. Without that distinction, Sufficiency has no instrument at all.

#### Probe categories

| Category | Asks |
|---|---|
| Discovery | Does the agent find the governing document at all? |
| Sufficiency | After it finds the document, does it have enough to act correctly? |
| Navigability | Can it get from a code path to the governing document, and back? |
| Consistency | Same question, different phrasings, same answer? |

Sufficiency needs a `patched`, `cited` or `answered` expectation. Without one of the three it is a question about prose quality, which belongs to the sweep. Consistency compares the read set and the cited identifiers of two runs. An equality over two prose answers is not available to a grader that reads no prose.

Two earlier categories are gone, and each removal is a finding rather than a simplification. **Fidelity is not a probe.** A derived rule is a generated projection, and `generate --check` proves that it agrees with its source. To pay a model for that comparison re-derives what the graph already declares. **The counterfactual is not a category either.** It is an **arm** of every probe: `present` or `absent`. Listing it beside the others hid that it applies to all of them, and hid that the pair doubles the cost of whatever it measures. The absent arm names a declared ablation, so what "corpus absent" removed is a recorded fact.

An A/B run over the two arms is the only evidence that the instruction surface earns its context cost. Without it, "the AI reads our docs" is a belief.

Two constraints protect the instrument, and [HW-EVAL-adjacent-work §M](../evaluations/adjacent-work.md#m--what-the-survey-shows-as-a-whole-convergence-is-not-evidence) records why both are needed. Every adjacent project that claims this benefit either graded itself or skipped the counterfactual. The literature shows what a weak grader does to a result. A systematic comparison of RAG and graph-based RAG reached the opposite conclusion to the original study. The cause was the grading method rather than the systems. The same authors found that an LLM judge reverses its verdict when the order of two candidates is reversed.

- **The grader is never the system under test.** A verdict comes from the tool-call transcript and a declared expectation. No model judges whether the corpus helped, and no probe accepts an agent's account of its own behavior.
- **A published claim carries its counterfactual.** Corpus present against corpus absent, with a pinned model and a recorded probe selection. A claim with no such pair is reported as unmeasured rather than as supported, and the [evidence register](04-assurance-model.md) carries that mark.

#### A run produces a snapshot and a document

A run emits a **transcript**. It holds the ordered tool-call events with their arguments and result identities, the identifiers of every produced artifact, and the final closed-set answer. It also holds the **run identity**: the model with its served version, the corpus tree hash, and the taxonomy lock hash. The identity continues with the probe selection hash, the rotation seed, the harness version, the arm, and the time. The transcript holds no model prose. That omission is the enforcement, in the way that scope enforcement is the feature in [spec 12](12-check-layer.md#scope--the-declaration-everything-else-rests-on).

A transcript is a committed snapshot that a `probe_run` anchor resolver reads. A probe run is an external system of record, and [Q19](09-decisions.md#q19--inbound-integration-an-external-system-of-record) built that machinery already. So nothing new arrives here, and the declaration count stays at thirteen. The **probe result** is a document, generated from the transcript, the expectations and the grader version. It carries the `regenerated` [warrant](01-conceptual-model.md#warrant), and `generate --check` proves it.

##### A probe result is citable because each of its three inputs is a committed artifact

The sentence above names three inputs, and a rate is citable when a reader can fetch every one of them and get the rate again. This repository declares where each one lives.

| input | where it is | who writes it |
|---|---|---|
| the transcript | a `probe_transcript` document on `docs/probe-runs/` | the recorder, mechanically |
| the expectations | a `probe` document on `docs/probes/` | a person |
| the grader version | the version of the code that evaluated them | the engine |

That placement separates a probe result from the [capture-cost store](03-authoring-and-lifecycle.md#capture-cost-is-a-tracked-metric), which sits outside the corpus root. The test is whether anything regenerates the artifact from a committed source. A capture-cost reading is a fact about a run that ended, and nothing recomputes it. A probe result is recomputed on every run of `generate --check`. So a result is corpus content, and the transcript under it is too.

A number that no reader can recompute is the failure this table prevents. A result with an uncommitted transcript carries the `asserted` warrant. An asserted document discharges no evidence obligation, so the measurement layer would produce content that this corpus refuses as evidence.

##### A transcript is recorded from outside the session, and never written back by the agent

The [coherence sweep](04-assurance-model.md#discharging-coherence-obligations-the-assisted-sweep) reaches a model in three parts. The engine writes a briefing, an agent reads it, and the engine reads back the file the agent wrote. **A probe cannot take that shape, and the reason is a rule this section states.** A file that an agent writes about the documents it opened is the agent's account of its own process, and no probe accepts one.

So the middle part of a probe run is a **recorder**: a process that drives a session and writes what it observes, event by event. It is the one component of this layer that reaches the network. No verb of the engine writes a transcript, and the engine plans a run and reads a recorded transcript back. [Spec 15](15-the-recorder-contract.md) states everything a recorder writes, and the four closed key sets that refuse a transcript whole.

The omission of model prose is enforced rather than asked for. Every key of a transcript is a member of a closed set, and one key outside it refuses the file. An omission that nothing tests is a claim. A transcript with a `reasoning` key beside the tool calls returns the self-report through the field the rule forbids.

##### The harness confirms five things, and one component after it grades

`headwater probe plan` fixes the six members of the run identity that exist before a run. They are the lock, the corpus tree, the selection, the read set, the seed and the harness version. It projects the sessions against the tier's declared budget and refuses a run above it. Five other conditions stop a run, and each stops the whole run rather than the probe that raised it:

- a `patched` probe that names a rule this engine does not carry
- a probe of another form that names a rule
- a probe over documents that names none
- an `answered` probe that declares no closed set of answers
- a probe of another form that declares one

A selection that dropped what it could not read reports a rate over a denominator nobody declared.

`headwater probe record` reads a transcript back. It confirms the taxonomy, the run identity, the membership of every probe named, and a recorded cost. It confirms that no key outside the closed set appears. It evaluates no expectation, because a verdict is the grader's, and a grader inside the intake is a grader nobody reviewed.

##### What earns the grader the right that every other agent-facing mechanism is denied

`headwater probe grade` returns a verdict. Nothing else in this engine that reaches toward a model does. The sweep verifies the citations of what an agent wrote and reports findings that a person accepts. The intake confirms the five things above and grades nothing. Three properties are what separate the grader from both, and each one is a shape of the mechanism rather than a promise about it.

**Its inputs carry no prose.** A transcript is an event log whose every key comes from a closed set, and an expectation is one of the five predicates above. Nothing the grader reads is a sentence, so nothing it does needs a rubric. This is the rule at the top of this section, read from the other end.

**A satisfied verdict names the event that satisfied it.** The verdict type holds that witness, so a satisfied verdict with nothing behind it cannot be written. A reader re-derives any verdict by counting events in the committed transcript. A verdict that is not satisfied carries the extent of the search. "Nothing matched" is a claim about how much was read.

**It refuses where a passing answer would be free.** Six conditions return no verdict rather than a verdict. A `patched` expectation whose probe declares no oracle. A produced artifact that nothing checked. A `not_opened` over a session that made no tool call, because a session that did nothing is no evidence that it avoided something. A form whose input the recorder never wrote down. A `cited` expectation over targets that carry no identifier. A probe the transcript never names. A refused session leaves the denominator of the rate rather than joining the numerator, and the result prints how many were refused beside the rate.

**Determinism is necessary and it is not the argument.** Two runs of the grader over one transcript write one set of bytes. That is what makes a result reproducible where the behavior under it is not. It is not what makes a result correct, because a grader that answered `satisfied` to everything is also deterministic. A recorded fixture set is what holds the verdicts. This correctness root carries one transcript that satisfies every predicate form, one that refutes every form, and one that refuses every form.

##### An empty list and a missing key are different facts about a run

Three keys of an event carry this distinction, and the grader is wrong without it. `calls: []` says the recorder watched the session and saw no tool call. An event with no `calls` key says that nothing watched. The same holds for `produced` and for `answer`. It holds for the `findings` of one produced artifact: `findings: []` is checked and clean, and no `findings` key is unchecked.

A grader that read the second as the first reports a verdict about the recorder as a verdict about the corpus. The direction of that error is the one this whole layer exists to prevent, because every one of those readings returns a pass.

Two keys of a produced artifact carry a derivation, and that is deliberate. A digest is an identity, and an identity answers no expectation. Two artifacts that cite different identifiers have different digests, and the digest says which cited what. So `cites` is every identifier of the corpus that appears in the artifact, and `findings` is every rule that reported over it. The recorder computes both the same way for every probe and consults no probe. A recorder that wrote only the identifiers one probe named would be a grader with no fixture set and no version.

**The seed is the caller's number.** A seed derived from the corpus moves the rotation whenever the corpus moves. A difference between two runs then carries a change in the phrasing and a change in the corpus at once. A run that repeats a seed repeats a selection.

The transcript is not optional, and the reason is the evidence rules. A result with no committed transcript has nothing inside the repository behind it, so its warrant is `asserted`. An asserted document discharges no evidence obligation ([spec 3](03-authoring-and-lifecycle.md#evidence-has-three-honest-states-not-two)). The measurement layer would then produce content that the corpus refuses as evidence.

**A model name is not a pin.** Two snapshots of one named model, three months apart, moved from 84% to 51% on a single task. The same pair moved in opposite directions on other tasks ([HW-EVAL-adjacent-work §R](../evaluations/adjacent-work.md#r--measuring-whether-the-corpus-works)). So a run records the served version where the provider exposes one, and records the name as a name where it does not.

#### Drift and variance are separated by an interval

The grading is deterministic and the behavior is not, so [principle 3](00-vision-and-scope.md#design-principles) answers in two halves. A regenerated result that disagrees with its own transcript is a **defect** in the grader, the parser, or the committed inputs, and `generate --check` catches it. A rerun that returns a different rate is either sampling variance or **drift**, and only an interval separates them. So a probe result reports an interval rather than a point. An interval that overlaps the previous one is variance. An interval that does not overlap is drift, and the run identity says where to look.

#### Two tiers, and the cadence follows the purpose

Cadence does not follow the category. It follows whether a run watches for a change or estimates a difference, and any category does either.

| Tier | Purpose | Shape | Cadence |
|---|---|---|---|
| **Regression** | Detect that something moved | A fixed scenario set, one arm, against a recorded baseline | Scheduled, and weekly is a sound default |
| **Campaign** | Estimate a difference for one named claim | Both arms, powered, one batch, one model version | On the claim: when it is published, and when a change voids it |

A campaign runs as one batch, or it is not one measurement. A run spread over weeks may hold a model that moved inside it. The regression tier runs one arm, so it establishes no effect and no published claim rests on it.

**A probe never runs against a proposed change.** The network is closed at check time, no LLM sits in the validation path, and a probe is a sampler rather than a check ([spec 12](12-check-layer.md#where-the-llm-coherence-sweep-fits)). A verdict that a rerun may reverse is not what a gate needs, and a per-change cost falls on the wrong payer.

What a change does instead costs nothing. A probe result is a verdict about one state of the corpus, so it carries a [read set](12-check-layer.md#the-read-set-and-what-a-merge-does-to-a-verdict). That set holds the documents that the transcript shows the session opened. It also holds the tree, the lock hash, the model identity, the selection and the harness version. A run reports which recorded probe results the change voided, through the machinery that already derives invalidated instances from a diff. The finding is advisory and it names the result. A trend over quarters needs this, because a line through points of unknown staleness compares nothing.

#### The envelope is declared, and the harness fails closed

The cost of the layer is the session count times the cost of a session, and the session count is scenarios times arms times repetitions. A campaign's count comes from statistical power rather than from taste. At 80% power and a 5% two-sided level, 0.50 against 0.75 takes about 58 sessions per arm. The same test on 0.60 against 0.75 takes about 152 per arm. A campaign is therefore one hundred to three hundred sessions, and a smaller effect costs a much larger run.

Each tier declares a budget. The harness projects the cost of a run before it starts, and it refuses to start a run that exceeds the budget. That is [principle 7](00-vision-and-scope.md#design-principles) read the way that an exporter reads it: a run that does not happen is the cheaper error. The harness reports realized cost beside the result, so the adaptive layer measures its own instrument ([spec 4](04-assurance-model.md#the-adaptive-layer-reports-cost-not-just-coverage)).

Probes are not latency-sensitive, which is a cost lever rather than a detail. A scheduled run tolerates a batch interface and its discount, and many scenarios against one corpus share a cached prefix.

Results feed the adaptive layer of the [assurance model](04-assurance-model.md). A rule that measurably changes nothing is a candidate for deletion, and deletion is a success.

## Anti-overfitting

Probes are written against **behavior**, not against phrasings. A probe that passes because a rule file contains a magic sentence tests the sentence. The harness rotates and paraphrases probes deterministically per run. The rotation seed is part of the run identity, so a selection reproduces even though a behavior does not. **A paraphrase varies the task statement and never the expectation.** A paraphrase that moves the predicate has written a second probe under one identifier.

Probes are also reviewed for the failure mode where the corpus is tuned to the probe suite instead of to its readers. One measure of the suite falls out of the arms. **A probe whose two arms never differ measures nothing about the corpus**, and [principle 6](00-vision-and-scope.md#design-principles) makes it a candidate for deletion. That is the same test that [HW-EVAL-theoretical-foundations](../evaluations/theoretical-foundations.md#what-the-theory-did-not-settle) sets for a coherence metric whose distribution never moves.

## What we do not do

- No LLM in the validation path. Verdicts are deterministic and reproducible.
- No retrieval-augmented generation as the primary mechanism. The precision argument matters: the graph is exact, cheap, and explainable, and embeddings are none of those. But the deeper objection is that **RAG accumulates nothing**. Every query re-derives its answer from fragments chosen by similarity, and the synthesis is discarded. Ask again next week, and the work happens again. A governed corpus compiles knowledge *once* — validated, related, projected — and keeps it current, so retrieval reads a standing answer and does not reconstruct one. Chunking strategies exist to manage the structural loss that embedding introduces — we decline the loss rather than manage it. A third objection, which [HW-EVAL-adjacent-work §L.2](../evaluations/adjacent-work.md#l2-prefer-references-to-search--a-third-argument-against-retrieval) takes from Serena's memory design, is that every retrieval method carries both error modes at once. Lexical and semantic search each return false positives and false negatives, and a named edge returns neither. Retrieval is therefore a source of variance, and a declared edge removes it. Embeddings remain permitted as a fallback for genuinely fuzzy lookup, and never as the authority. An external RAG system that consumes the corpus is a separate integration question, not a change to this.
- No agent-authored documents merged without review. The agent drafts and links. A human accepts.
