---
id: HW-RUN-second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability
status: current
status_since: 2026-09-28
summary: "The absent arm of the campaign tier over the navigability selection: 3 sessions, 69 cents, the intent hook live in 0."
last_verified: 2026-09-28
tier: campaign
arm: absent
title: "Second campaign pilot of 2026-09-28, campaign tier, absent arm, navigability"
---

# Second campaign pilot of 2026-09-28, campaign tier, absent arm, navigability

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
arm: absent
at: 2026-09-28
cost_cents: 69
```

**This is a second pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at one repetition per probe, to test the three changes the owner ruled on [#980](https://github.com/headwater-ai/headwater/issues/980) after the first pilot. First, every workspace of both arms carries the built `headwater`, so the intent hook can run in a present-arm session. Second, a closed-set word alone on the final line is the answer. Third, the seal removes HW-OBL-0198, the answer key of the `patched` probe. No comparison pools it with the full run, and no obligation is restated on it.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `ed9e61a7`, with the engine that commit builds, in one order that seed 981 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names a probe of the batch, and the answer keys of those probes. Each absent workspace also lost its tier's ablation.

## Events

```yaml
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L5-campaign-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"Q8\\\" -A 50 \\\"docs/spec/09-open-questions.md\\\" | head -100\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p1-r1/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L5-campaign-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p2-r1 -type f | head -200\",\"description\":\"List files in working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"scent\\\" /home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p2-r1 --include=\\\"*.md\\\" -i 2>/dev/null\",\"description\":\"Search markdown files for the word scent\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"scent\\\" /home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p2-r1 -i 2>/dev/null\",\"description\":\"Find all files mentioning scent\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p2-r1/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L5-campaign-absent-p3-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la /home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p3-r1\",\"description\":\"List contents of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"corpus\\\" . --include=\\\"*.md\\\" --include=\\\"*.py\\\" --include=\\\"*.js\\\" --include=\\\"*.ts\\\" --include=\\\"*.yml\\\" --include=\\\"*.yaml\\\" --include=\\\"*.json\\\" 2>/dev/null | grep -v \\\"/site/\\\" | head -50\",\"description\":\"Search repo for files mentioning 'corpus'\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"count\\\" docs/decisions/ | xargs grep -l \\\"corpus\\\" 2>/dev/null\",\"description\":\"Find decision docs mentioning both count and corpus\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p3-r1/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
    - tool: "Bash"
      argument: "{\"command\":\"find engine/crates/census engine/crates/check -type f -name \\\"*.rs\\\" | xargs grep -ln \\\"len()\\\\|count\\\\|total\\\" 2>/dev/null\",\"description\":\"Find code files computing counts in census/check crates\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"sed -n '1,80p' engine/crates/census/src/census.rs\",\"description\":\"View start of census.rs\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"HW-DR-0049\\\\|0049-a-corpus-wide-fold\\\" --include=\\\"*.rs\\\" --include=\\\"*.md\\\" . 2>/dev/null | grep -v \\\"/site/\\\"\",\"description\":\"Find existing citations of decision 0049\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L5-campaign-absent-p3-r1/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"why-corpus-counts-are-derived-not-stored\\\\|HW-EVAL-why-corpus-counts-are-derived\\\" . 2>/dev/null | grep -v \\\"/site/\\\"\",\"description\":\"Find all references to the existing evaluation doc\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"warrant:\\\" docs/spec/ 2>/dev/null; echo \\\"---\\\"; grep -rn \\\"^warrant\\\\b\\\\|warrant \\\" docs/glossary* 2>/dev/null | head -20; echo \\\"---search glossary dir---\\\"; find docs -iname \\\"*glossary*\\\" -maxdepth 2\",\"description\":\"Look for warrant field definition in spec/glossary\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"warrant\\\\|asserted\\\\|accepted\\\\b\\\\|evidenced\\\\|grounded\\\" docs/spec/glossary.md | head -40\",\"description\":\"Search glossary for provenance vocabulary\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"why-corpus-counts\\\" docs/evaluations/README.md engine/crates/census/fixtures/corpus.census\",\"description\":\"Check listing entries for the evaluation doc\"}"
      result: ""
  produced: []
  answer: null
```
