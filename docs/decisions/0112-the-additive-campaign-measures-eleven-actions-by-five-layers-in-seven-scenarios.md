---
id: HW-DR-0112
status: current
status_since: 2026-10-29
summary: "The 0.07 campaign grades 11 canonical actions against 5 layers in 7 scenarios over 44 probes. Phase A tunes each layer pooled over the tuning probes, with a 22-session screen and a 145-session confirmation. Phase B reports held-out numbers per action. This record also keeps the probe shape, the verdict rule and the fixed model."
last_verified: 2026-10-29
title: "112 — The additive campaign measures eleven actions by five layers in seven scenarios"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - tools/probe/arms/hooks/.claude/settings.json
    - .headwater/probe.yml
    - tools/probe/oracles/**
    - tools/probe/oracle-fixtures.sh
    - tools/probe/oracle-args/**
    - tools/probe/oracle-cases/**
    - tools/probe/probe-oracle-fixtures.sh
    - tools/probe/ablate.sh
    - tools/probe/additive-campaign.spec
    - tools/probe/additive-screening.spec
    - tools/probe/try.sh
    - tools/probe/additive-launch.env
    - tools/probe/code-campaign.spec
    - tools/probe/code-pilot.spec
    - tools/probe/declared.py
    - tools/probe/usage.py
    - tools/probe/additive-dry-run.sh
    - tools/probe/leak.py
    - tools/probe/seal.sh
    - tools/probe/try-fixtures.sh
    - tools/probe/probe-record-fixtures.sh
  supersedes:
    - HW-DR-0111
  traces_to:
    - HW-PD-0026
---

# 112 — The additive campaign measures eleven actions by five layers in seven scenarios

## Context

[HW-DR-0111](0111-the-additive-campaign-adds-six-layers-to-plain-code-in-fourteen-scenarios-and-registers-one-comparison-each.md) built fourteen scenarios from six layers and measured them over nine probes. Each of those probes asked a question about the corpus. So the set did not cover most of what a person asks an assistant to do in a governed repository. The tuning round of that design closed on #1777, and its final report is comment 6467831567 there.

The owner re-cut milestone 0.07 on 2026-10-28 and 2026-10-29. His rulings are on #1777, in comments 6465567947 and 6466863444, and in the body of epic #1868. That body is the source of the design that this record states. The re-cut is the first application of [HW-PD-0026](../process/decisions/0026-we-pivot-rather-than-carry-debt-forward.md): a better structure is worth the loss of a comparison with earlier runs. The epic closed the fourteen scenario issues, #1780 to #1793, and folded #1835 and #1836 into #1874.

This record supersedes HW-DR-0111 as a whole, because a supersession sets its target to `superseded` and the taxonomy has no partial form. So this record states again, as its own rule, each part of HW-DR-0111 that #1868 keeps. No rule in force lives only in the superseded record.

The campaign is the next instrument for four open claims of spec 13: HW-OBL-0012, HW-OBL-0013, HW-OBL-0014 and HW-OBL-0016. This record reads those claims and closes none of them.

## Decision

### The grid: eleven canonical actions by five layers

The campaign measures eleven canonical actions. Each row is one action, and each layer column says how the campaign treats that layer for that action.

● the layer is in the tuning subset for this action · ○ measured, not tuned · – measured for harm only. The documents are the adopter's own: measured in every row, tuned in none. The engine is tuned by the text its verbs print, never its logic.

| # | canonical action | graded by | docs | engine | skills | hooks (moments) | MCP |
|---|---|---|---|---|---|---|---|
| 1 | ask or locate | grader: correct answer | ○ | ● `explain`, `route` | ● | ● intent | ● |
| 2 | change governed code, one refusal probe among them | oracle: test passes, or cited refusal | ○ | ● `route <path>` | ● | ● intent, read, write, review | ● |
| 3 | refactor across governed files | oracle: builds, check clean | ○ | ● | ● | ● read, write, review | ○ |
| 4 | add a document | oracle: typed at the right path, check clean, cites its ruling | ○ | ● `new` | ● | ● write refusal, review | ● |
| 5 | edit a document others depend on | oracle: reciprocal link, check clean | ○ | ● `check` | ● | ● write dependants, review | ○ |
| 6 | supersede or retire a document | oracle: successor typed, links updated | ○ | ● | ● | ● write, review | ○ |
| 7 | move or relocate a document or a shelf | oracle: moved, links repaired, check clean | ○ | ● | ● | ○ often bypassed via the shell | ○ |
| 8 | change taxonomy or config | oracle: check clean, lock regenerated | ○ | ● | ● taxonomy | ● write derived | ○ |
| 9 | repair a failing check | oracle: check clean, nothing else broken | ○ | ● `check --fix` | ● | ● review | ● |
| 10 | review someone else's change | grader: findings cite the right documents | ○ | ● `check --change` | ● maintainer | ○ intent, read | ○ |
| 11 | ungoverned control | oracle: done; count of hook warnings must be zero | – | – | – | – | – |

### The circumstance axes

Each probe has a value on each circumstance axis, and each result notes it. This round makes no claim about an axis alone. The axes are three:

- whether the task names the file,
- whether the target is superseded or contradicted, with one real `draft` fixture that disagrees with a `current` document, from #1836,
- the phrasing, with two or three phrasings for each probe, drawn for each session and pooled.

Three axes are deferred and recorded as debt: corpus size, tasks outside anything a document governs, and an adopter's own repository.

### The probe set

The set has 11 actions × (2 tuning + 2 held-out) = 44 probes. Each probe is tagged with its action and the hook moments it passes through. Rows 2–9 and 11 pass or fail by a mechanical oracle. Rows 1 and 10 pass or fail by the grader. A probe for rows 2–9 with no mechanical oracle is refused from the set.

### One probe shape

Every probe has one shape: one task in plain words, one target, and one key. Each probe document states them under three headings of the same names, `Task`, `Target` and `Key`. The grader checks the key from the recorded transcript, with no model as a judge. For an oracle row, the key is the oracle: the probe passes when the oracle command passes in the grading copy. #1874 defines the form of an oracle.

### The form of an oracle

Added on 2026-10-29 by #1874. A probe of rows 2–9 or 11 declares `expectation: patched` and `oracle: "command:<name>"` or `oracle: "test:<name>"`. The name is an entry under `oracles:` in `.headwater/probe.yml`, with the four fields that block already has: `paths`, `command`, `parser` and `holds`.

The command calls a generic script under `tools/probe/oracles/` with the arguments of the probe. The script runs from the root of the grading copy, and it reads no history, because that copy holds none. It exits 0 on a pass and 1 on a fail, and a usage error exits 2. It writes exactly one line of evidence to standard output.

The oracle holds each script that its command runs, and `lib.sh`, which each script sources. The recorder restores a held file over the copy that the session wrote. So a session cannot change the script that grades it.

A file that holds a value only the answer knows names its probe in its first ten lines, and the oracle holds it. A hidden test and a baseline of findings are two examples. The seal removes such a file from every workspace. A generic script names no probe. The seal keeps `probe.yml` as a fold, so each workspace still holds the command line. A command therefore states what the oracle checks and never the answer.

Added on 2026-10-29 by the second slice of #1874. A command can read its arguments from a held file with `--args <file>`, one argument to a line, as the last option. Such a file names its probe in its first ten lines, as above. No key under `oracles:` and no held path names a probe, because the seal removes each line of `probe.yml` that names a removed document. The seal also leaves errors in a sealed tree, on documents that cite a removed record. So `--check-clean` in `typed-at.sh` and `reciprocal-link.sh` counts only the errors on the documents that the script names.

Row 11 has two conditions. The `oracle:` facet names the oracle over the tree. `no-hook-warning.sh` reads the harness log of the session from `HEADWATER_ORACLE_HARNESS_LOG`, because the stream holds no hook attachment. #1875 exports that variable and runs the script on each session of row 11.

`tools/probe/oracle-fixtures.sh` runs each script over a tree that it must pass and a tree that it must fail. An oracle that passes both trees, or fails both, fails CI. The pilot of #1835 found probes of each kind.

`tools/probe/probe-oracle-fixtures.sh` runs the oracle of each probe in a sealed grading tree, in three fresh copies. The tree as it stands must fail, the reference change must pass, and the wrong change must fail. The two changes for each oracle are in `tools/probe/oracle-cases/<name>.sh`, which names its probe, so the seal removes it.

### The form of a phrasing

Added on 2026-10-29 by #1874. Under `## Task`, a probe has two or three subsections with the headings `### Phrasing 1`, `### Phrasing 2` and `### Phrasing 3`. Each one states the whole task in different words. The target and the key are the same for each phrasing. The shape check reads level-two headings only, so a probe with phrasings keeps the shape of `Task`, `Target` and `Key`. Until #1875 draws one phrasing for each session, a session reads every phrasing of its probe.

### The seven scenarios

| scenario | layers | base |
|---|---|---|
| 0 | none | – |
| 1 | documents | 0 |
| 2 | documents + engine | 1 |
| 3 | 2 + skills | 2 |
| 4 | 2 + hooks | 2 |
| 5 | 2 + MCP | 2 |
| 6 | 2 + skills + hooks + MCP, which is what `headwater init` ships | 2 (three layers, stated as the one exception) |

Each scenario except scenario 0 adds one layer to its base. Scenario 6 is the one exception: it adds three layers to scenario 2. The campaign measures no pairings.

`CLAUDE.md` is not a layer. The campaign tunes only the layers that `headwater init` ships. The skills, the hooks and the MCP server call the engine, so their additive base is scenario 2.

The hooks layer is `intent.sh`, `read.sh`, `write.sh` and `review.sh`, with `lib.sh` and `touch.sh`, and the `settings.json` entries that wire them. `review.sh` runs `headwater check --strict` itself. So `.githooks` stays repository tooling and is in no layer.

### Phase A: tuning

Phase A tunes four layers, each against one comparison:

- the engine's output text, in scenario 2 against scenario 1,
- the skills, in scenario 3 against scenario 2,
- the hooks, in scenario 4 against scenario 2,
- the MCP server, in scenario 5 against scenario 2.

Each comparison is pooled over the actions, and no row has its own spend. A round screens at most 4 variants. Each variant runs 22 sessions on a fixed draw, which is one session for each tuning probe. One phrasing is drawn once for the whole screen, and it is the same for each variant. The winner is then confirmed at 145 sessions, pooled.

A hook variant is screened and confirmed on the rows of its own moment, plus row 11:

| hook | rows |
|---|---|
| `write.sh` | 2–9 and 11 |
| `review.sh` | 2–9 and 11 |
| `read.sh` | 2, 3, 10 and 11 |
| `intent.sh` | all |

The four branches `tune/hook-r1-v1-pin` to `tune/hook-r1-v4-pin` are the first candidates for the intent hook. Each is rebuilt on the new base.

Phase B does not start until every phase A layer is kept or closed. Nothing in phase A waits on phase B.

### Phase B: the report

Phase B runs on the kept pin and on the held-out set only. It runs each row in scenario 1 and in scenario 6, at 72 sessions for each row. It also runs scenario 0 at 145 sessions, pooled, as the anchor. Scenario 2 comes from phase A.

A cell that saturates stops at 24 sessions. A cell saturates when all its sessions pass or all fail at 24.

The published table carries the held-out number of each row. The skills, hooks and MCP columns carry the pooled layer result from phase A, and a per-cell indication marked unconfirmed. No tuning-probe cell table is bought.

### The keep rule

A tuning change is kept when the 95% Newcombe interval of its gain clears zero. When the best so far is at or above 85%, a change is kept when its point estimate is up. Its 80% interval must also clear zero. The owner ruled the second clause on 2026-10-25, on #1777.

### The verdict rule

Each comparison is registered before the run. It is the difference from the base, and it is the only comparison that earns a verdict. The verdict is one word, on the 95% Newcombe interval of the difference. It is `gain` when the interval lies above zero, `loss` when it lies below zero, and `no detectable effect` when it holds zero.

### What counts as a call to the server

A call to the server is a `tool_use` event whose tool name starts with `mcp__`. Each scenario that holds the MCP server reports the share of its sessions that made no call. A verdict of `no detectable effect` with a low share reads as a server that sessions did not reach.

### One model, one harness

The subject model is `claude-sonnet-5`, on one version of one harness, and each record names that version. The engine is built from a recorded commit, and the transcript of each session names its digest. A table on another model is #1700's business, and another harness is #503's.

### A base that a change makes stale

A base that a later change can alter is measured again before a confirmation compares with it. `.headwater/probe.yml` declares `base_invalidators`, the paths whose change can make a measured base stale. A base is accepted again only by an owner acceptance that records the digest of each changed path. A confirmation waits for a base measured at its own commit. `tools/probe/try.sh` holds this mechanism, and this record keeps it.

### The spend and the pace

Each paid issue gets its own ceiling when its dry-run plan is posted on it. No run exceeds a ceiling. A run stops launching sessions at 85% of the 7-day window, unless the reset is under 24 hours away. In that case, the run goes on to 100%.

## Consequences

**The design of HW-DR-0111 is retired.** The fourteen-scenario baselines, the 9-probe set, the code set of #1835, and the ceilings of $700, $1,512 and $130 are retired. Each stays as a dated record. The two evaluations of that campaign are records of a moment and stay as written: [the screening round of 2026-10-18](../evaluations/what-the-additive-screening-round-of-2026-10-18-measured-by-scenario-against-its-base.md) and [the tuning round](../evaluations/what-the-tuning-round-of-the-additive-campaign-measured-layer-by-layer.md). #1874 folds the code set into the 44-probe set.

**The tools still carry the old design until their issues re-cut them.** `.headwater/probe.yml`, `tools/probe/*.spec` and the engine's `probe` crate still declare the fourteen scenarios and the old sets. #1872 re-cuts `probe.yml` to five layers and seven scenarios. #1873 adds the `action`, `moments` and `axes` facets. #1874 builds the 44-probe set, and #1875 builds the campaign tooling. Each of those issues repoints its own citations of HW-DR-0111. HW-DR-0111 keeps its `governs` edges on `tools/probe/` until #1872 or #1875 moves them.

**The open claims of spec 13 stay open.** This record reads HW-OBL-0012, HW-OBL-0013, HW-OBL-0014 and HW-OBL-0016, and it closes none of them. The evaluation that phase B publishes is the record that can discharge them.

**These rulings open again** if the owner changes the grid, the scenarios or the keep rule. They also open again if the owner approves a ceiling for a paid issue, or if a phase A layer cannot be kept or closed. The grid opens again if a held-out row saturates in the base of every scenario.
