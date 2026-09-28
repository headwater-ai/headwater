---
id: HW-RUN-second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability
status: current
status_since: 2026-09-28
summary: "The present arm of the campaign tier over the navigability selection: 3 sessions, 98 cents, the intent hook live in 3."
last_verified: 2026-09-28
tier: campaign
arm: present
title: "Second campaign pilot of 2026-09-28, campaign tier, present arm, navigability"
---

# Second campaign pilot of 2026-09-28, campaign tier, present arm, navigability

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0
lock: sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8
selection: sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76
read_set: sha256:00323cb8f153c3f22044099b6eeca18c5787b93cdb0d89262d285b926703ee49
seed: 0
harness: 0.4.0
tier: campaign
arm: present
at: 2026-09-28
cost_cents: 98
```

**This is a second pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at one repetition per probe, to test the three changes the owner ruled on [#980](https://github.com/headwater-ai/headwater/issues/980) after the first pilot. First, every workspace of both arms carries the built `headwater`, so the intent hook can run in a present-arm session. Second, a closed-set word alone on the final line is the answer. Third, the seal removes HW-OBL-0198, the answer key of the `patched` probe. No comparison pools it with the full run, and no obligation is restated on it.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `ed9e61a7`, with the engine that commit builds, in one order that seed 981 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names a probe of the batch, and the answer keys of those probes. Each absent workspace also lost its tier's ablation.

## Events

```yaml
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L4-campaign-present-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"Q8\\\" -A 60 \\\"docs/spec/09-open-questions.md\\\" | head -100\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p1-r1/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L4-campaign-present-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"scent\\\" --include=\\\"*.md\\\" . | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p2-r1/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"scent\\\\|cites\\\" docs/spec/02-taxonomy-model.md | head -40\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p2-r1/docs/spec/02-taxonomy-model.md"
      result: "sha256:d82f2e97d492172a490c375208722d6ca290e1d32c049d54a38cfff019026870"
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L4-campaign-present-p3-r1"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":1}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git rev-parse --is-inside-work-tree 2>&1; git status 2>&1 | head -20\",\"description\":\"Check if directory is a git repository\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la && find . -maxdepth 2 -iname \\\"*.git*\\\" && cat .claude/settings.json 2>/dev/null\",\"description\":\"Inspect repo root and settings for worktree hooks\"}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rIl \\\"count\\\" docs/decisions docs/decisions 2>/dev/null | xargs grep -liE \\\"corpus\\\" 2>/dev/null\",\"description\":\"Search decisions docs mentioning count and corpus\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"corpus.checks\\\\|export.json\\\\|HW-DR-0049\\\\|fold\\\\|derived and never stored\\\\|write skew\\\" DEVELOPING.md | head -60\",\"description\":\"Search DEVELOPING.md for related terms\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"corpus.checks\\\\|export.json\\\\|headwater export\\\\|census\\\\|graph_export\\\" DEVELOPING.md\",\"description\":\"Search DEVELOPING.md for corpus.checks and export.json references\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/DEVELOPING.md"
      result: "sha256:9e0b1b61b73c2b43d0bfff9054d4df1bbf74b939f65f4837b7b53a23261124fb"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"^## \\\" DEVELOPING.md\",\"description\":\"List section headings in DEVELOPING.md\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/DEVELOPING.md"
      result: "sha256:9e0b1b61b73c2b43d0bfff9054d4df1bbf74b939f65f4837b7b53a23261124fb"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"export.json\\\\|corpus.checks\\\" --include=\\\"*.md\\\" . 2>/dev/null | grep -v \\\"docs/decisions/0049\\\"\",\"description\":\"Search for other mentions of export.json and corpus.checks outside decision 0049\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/engine/README.md"
      result: "sha256:3ced47e09cfb6ec0aeb2494a67d0d60f48aab3abc498ce13f60bee01272b03a7"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/docs/how-to/keep-derived-files-from-conflicting-in-parallel-pull-requests.md"
      result: "sha256:0e19e101adefffeeb0aade75c6d4bdaa0b1401c0e321ab1beb77784abf5a98a3"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/engine/README.md"
      result: "sha256:3ced47e09cfb6ec0aeb2494a67d0d60f48aab3abc498ce13f60bee01272b03a7"
    - tool: "Edit"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/engine/README.md"
      result: "sha256:3ced47e09cfb6ec0aeb2494a67d0d60f48aab3abc498ce13f60bee01272b03a7"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"corpus.checks\\\\|check/fixtures\\\" tools/engine/engine-readme-fixtures.sh tools/repo/*.sh 2>/dev/null | head -20\",\"description\":\"Check whether fixture scripts pin the edited paragraph's exact text\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L4-campaign-present-p3-r1/engine/README.md"
      result: "sha256:3ced47e09cfb6ec0aeb2494a67d0d60f48aab3abc498ce13f60bee01272b03a7"
  produced:
    - path: "engine/README.md"
      result: "sha256:3ced47e09cfb6ec0aeb2494a67d0d60f48aab3abc498ce13f60bee01272b03a7"
      cites:
        - "HW-DR-0045"
        - "HW-DR-0049"
        - "HW-DR-0064"
        - "HW-DR-0075"
        - "HW-OBL-0080"
        - "HW-OBL-0115"
        - "HW-OBL-0128"
      findings: []
  answer: null
```
