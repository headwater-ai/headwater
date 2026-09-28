---
id: HW-RUN-second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-28
summary: "The absent arm of the campaign tier over the sufficiency selection: 4 sessions, 169 cents, the intent hook live in 0."
last_verified: 2026-09-28
tier: campaign
arm: absent
title: "Second campaign pilot of 2026-09-28, campaign tier, absent arm, sufficiency"
---

# Second campaign pilot of 2026-09-28, campaign tier, absent arm, sufficiency

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0
lock: sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8
selection: sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d
read_set: sha256:7a76c0b28551210b534705cf2b69216c68b6543dba250fc82fb24861e4949f46
seed: 0
harness: 0.4.0
tier: campaign
arm: absent
at: 2026-09-28
cost_cents: 169
```

**This is a second pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at one repetition per probe, to test the three changes the owner ruled on [#980](https://github.com/headwater-ai/headwater/issues/980) after the first pilot. First, every workspace of both arms carries the built `headwater`, so the intent hook can run in a present-arm session. Second, a closed-set word alone on the final line is the answer. Third, the seal removes HW-OBL-0198, the answer key of the `patched` probe. No comparison pools it with the full run, and no obligation is restated on it.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `ed9e61a7`, with the engine that commit builds, in one order that seed 981 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names a probe of the batch, and the answer keys of those probes. Each absent workspace also lost its tier's ablation.

## Events

```yaml
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L2-campaign-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p1-r1/docs -type f | head -200\",\"description\":\"List files under docs directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"amber\\\" docs/ ; echo \\\"---recovery word---\\\"; grep -ril \\\"recovery word\\\" docs/; echo \\\"---tombstone---\\\"; grep -ril \\\"tombstone\\\" docs/\",\"description\":\"Search docs for amber, recovery word, and tombstone mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"---recovery---\\\"; grep -ril \\\"recovery\\\" docs/; echo \\\"---counted tombstone---\\\"; grep -rn \\\"counted\\\" docs/ | grep -i tomb; echo \\\"---tombstone count---\\\"; grep -rn -i \\\"tombstone\\\" docs/**/*.md 2>/dev/null | head -80\",\"description\":\"Search docs for recovery and counted tombstone mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"amber\\\" docs/ 2>/dev/null; echo \\\"done: $?\\\"; find docs -iname \\\"*amber*\\\"; echo \\\"search for 'recovery word' exact\\\"; grep -rn \\\"recovery word\\\" docs/ 2>/dev/null\",\"description\":\"Double check for amber anywhere in docs including filenames\"}"
      result: ""
  produced: []
  answer: "absent"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L2-campaign-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" . 2>/dev/null\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p2-r1/docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md"
      result: "sha256:6de683b661a06755c335d85abbf92235bf188d5832a9ef657f7aa8b66fedf1d4"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p2-r1/docs/obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md"
      result: "sha256:1016178b65cdeac2092fe0e6f36c6510a46783d5173c72ead2ec10dcbde04330"
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p3-r1"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L2-campaign-absent-p4-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/ 2>&1\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- CLAUDE.md exists? ---\\\"; ls -la CLAUDE.md 2>&1; echo \\\"--- searching for debt/todo/backlog files ---\\\"; find . -maxdepth 3 -iname \\\"*todo*\\\" -o -iname \\\"*debt*\\\" -o -iname \\\"*backlog*\\\" -o -iname \\\"*known_issues*\\\" -o -iname \\\"*owe*\\\" 2>/dev/null | grep -v node_modules; echo \\\"--- repo name hint ---\\\"; head -30 README.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/obligations/ | sort\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations/README.md"
      result: "sha256:77f915e091292a6f00cf98516edb3cc61d7c2fb07efc20d258c63e852f749aa2"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations/0222-ten-rules-of-the-editions-ledger-have-one-recorded-corpus-so-a-rule-change-blessed-with-an-edit-to-that-corpus-passes.md"
      result: "sha256:5cabeb4143ff124d566f80b4631d1d5a2fcb58dbb8c2842560e1b4b4f93e6a63"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/AGENTS.md"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la CLAUDE.md AGENTS.md 2>&1; echo \\\"---\\\"; find . -maxdepth 2 -iname \\\"CLAUDE.md\\\" 2>&1; find . -iname \\\"CLAUDE.md\\\" 2>&1 | head -20\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -i claude .gitignore; echo \\\"---.codex---\\\"; ls -la .codex; echo \\\"---search repo for standing instructions text---\\\"; grep -ril \\\"standing instructions\\\" . 2>/dev/null; grep -ril \\\"context budget\\\\|token budget\\\\|session budget\\\" . 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations && ls -la | head -5; echo \\\"total files:\\\"; ls *.md | wc -l; echo \\\"--- highest numbers ---\\\"; ls *.md | sort | tail -15; echo \\\"--- checking for gaps 0197-0222 ---\\\"; for n in $(seq -w 190 222); do f=$(ls | grep \\\"^0$n-\\\" 2>/dev/null); if [ -z \\\"$f\\\" ]; then echo \\\"MISSING: 0$n\\\"; fi; done\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls | grep -E \\\"^000[5]|^0010|^0013|^0014\\\" ; echo \\\"---checking taxonomy-source for a duplicate copy---\\\"; find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 -path \\\"*/obligations/*0005*\\\" 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 && grep -n -i \\\"standing instructions\\\" docs/spec/glossary.md docs/spec/02-taxonomy-model.md docs/evaluations/theoretical-foundations.md docs/reviews/core-concepts-review-findings.md docs/taxonomies/brd-prd/doctrine.md docs/taxonomies/standards-spec/doctrine.md 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 && grep -n \\\"standing instructions\\\" docs/spec/glossary.md docs/spec/02-taxonomy-model.md docs/evaluations/theoretical-foundations.md docs/reviews/core-concepts-review-findings.md docs/taxonomies/brd-prd/doctrine.md docs/taxonomies/standards-spec/doctrine.md docs/evaluations/shacl-worked-example.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 && grep -rn \\\"standing instructions\\\" docs/ 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 && grep -rn \\\"0219\\\\|HW-OBL-0219\\\" --include=\\\"*.md\\\" . 2>/dev/null; echo \\\"---which headwater binary---\\\"; which headwater 2>&1; echo \\\"---check .headwater or taxonomy lock for id counters---\\\"; find . -iname \\\"*.lock\\\" -o -iname \\\"*headwater.toml*\\\" 2>/dev/null | head; echo \\\"--- check most recent obligations for filed-together pattern, e.g. 0203, 0208 multi-item ---\\\"; sed -n '1,40p' docs/obligations/0203-three-small-run-tooling-gaps-from-run-20260922-1121-filed-together.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat .gitignore | grep -n -B2 -A2 -i \\\"claude\\\\|agents\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"CLAUDE.md\\\\|AGENTS.md\\\\|context budget\\\\|token budget\\\\|before any work\\\\|session budget\\\" DEVELOPING.md | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"byte ceiling\\\\|byte_ceiling\\\\|BYTE_CEILING\\\\|context window\\\\|context budget\\\" --include=\\\"*.md\\\" --include=\\\"*.sh\\\" --include=\\\"*.rs\\\" . 2>/dev/null | head -40\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find docs/spec -iname \\\"*13*\\\"; echo \\\"---\\\"; grep -rn \\\"0197\\\\|0198\\\\|0219\\\" docs/ 2>/dev/null | grep -v \\\"Binary\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -path \\\"*/templates/*obligation*\\\" -o -iname \\\"*obligation*template*\\\" 2>/dev/null | grep -v docs/obligations; echo \\\"---overlay retired terms---\\\"; find . -iname \\\"overlay.yml\\\" 2>/dev/null; echo \\\"---\\\"; grep -rn \\\"waiting_on\\\" taxonomy-source/ 2>/dev/null | head -20\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/taxonomies/decision-record/templates/obligation_record.md"
      result: "sha256:f1a0c17d6498ae281c9c2a224bcc6a0cc2ccadc427b431f12c5e0573511ab110"
    - tool: "Bash"
      argument: "{\"command\":\"find taxonomy-source -iname \\\"*.yml\\\" | xargs grep -ln \\\"waiting_on\\\" 2>/dev/null\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations/0221-subagents-read-skills-from-the-shared-checkout-and-a-worktree-isolated-integrator-cannot-fast-forward-it-after-a-merge.md"
      result: "sha256:c13ec6716be81f8c743270094988380d5ffc3e7ce90793e438fde3a9d3171d92"
    - tool: "Bash"
      argument: "{\"command\":\"find docs/spec -iname \\\"*ai-integration*\\\"; grep -n \\\"context budget\\\\|dispatch\\\\|session\\\" docs/spec/05-ai-integration.md 2>/dev/null | head -40\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"byte ceiling\\\\|ceiling\\\" --include=\\\"*.rs\\\" --include=\\\"*.sh\\\" --include=\\\"*.md\\\" . 2>/dev/null | grep -vi \\\"docs/obligations\\\" | head -30\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations && grep -h \\\"^waiting_on:\\\" *.md | sort | uniq -c\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -A1 \\\"^---$\\\" 0100-the-size-of-a-committed-snapshot-with-full-requirement-text.md | head -3; grep \\\"waiting_on\\\" 0100-the-size-of-a-committed-snapshot-with-full-requirement-text.md; echo \\\"---\\\"; sed -n '1,40p' 0100-the-size-of-a-committed-snapshot-with-full-requirement-text.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"for f in $(grep -l \\\"^waiting_on: measurement\\\" *.md | head -3); do echo \\\"=== $f ===\\\"; sed -n '1,30p' \\\"$f\\\"; echo; done\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/spec/05-ai-integration.md"
      result: "sha256:0293231ed221ab41d3608df090c0e3c8ebad4ace48f8f46101cc6da13cdfc116"
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 && grep -n \\\"^#### \\\\|^### \\\\|^## \\\" docs/spec/05-ai-integration.md | sed -n '1,60p'\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations && grep -l \\\"DEVELOPING.md\\\" *.md | head -3; grep -h \\\"DEVELOPING.md\\\" *.md | head -5\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ho \\\"\\\\[[^]]*\\\\](\\\\.\\\\./\\\\.\\\\./DEVELOPING.md[^)]*)\\\" *.md | head -5\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/spec/05-ai-integration.md"
      result: "sha256:0293231ed221ab41d3608df090c0e3c8ebad4ace48f8f46101cc6da13cdfc116"
    - tool: "Write"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1/docs/obligations/0223-nothing-states-how-much-of-a-sessions-budget-the-standing-instructions-of-this-repository-spend-before-any-work-starts.md"
      result: "sha256:f00844408301676ae6ab66326eda8ec92f062bbdf1aa7586e11fdb00cdc5e2e4"
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L2-campaign-absent-p4-r1 && ls docs/decisions/0076-a-probe-budget-prices-a-run-identity-fixed-before-the-run-and-a-committed-transcript-and-a-sweep-has-neither.md && ls DEVELOPING.md && ls docs/spec/05-ai-integration.md && echo OK\"}"
      result: ""
  produced:
    - path: "docs/obligations/0223-nothing-states-how-much-of-a-sessions-budget-the-standing-instructions-of-this-repository-spend-before-any-work-starts.md"
      result: "sha256:f00844408301676ae6ab66326eda8ec92f062bbdf1aa7586e11fdb00cdc5e2e4"
      cites:
        - "HW-DR-0076"
        - "HW-OBL-0223"
      findings:
        - "identifier.claim.stale"
        - "language.controlled.not_met"
  answer: null
```
