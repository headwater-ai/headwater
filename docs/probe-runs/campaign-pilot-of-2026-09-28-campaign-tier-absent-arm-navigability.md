---
id: HW-RUN-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-navigability
status: current
status_since: 2026-09-28
summary: "The absent arm of the campaign tier over the navigability selection: 9 sessions, 159 cents, the intent hook live in 0."
last_verified: 2026-09-28
tier: campaign
arm: absent
title: "Campaign pilot of 2026-09-28, campaign tier, absent arm, navigability"
---

# Campaign pilot of 2026-09-28, campaign tier, absent arm, navigability

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:a25a8d3faf10c1e6e60b7962d4fd61866b5abef14f8266d7e7efcea1a107963f
lock: sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8
selection: sha256:fc04ac718a7df0b14774c28174a93f9d44dea6305b9901ba6e2fe6556834db76
read_set: sha256:8929d61762fc7e209ea77b36f1abc73da6d2ef947f4f79a2dd4103c336128209
seed: 0
harness: 0.4.0
tier: campaign
arm: absent
at: 2026-09-28
cost_cents: 159
```

**This is a pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at three repetitions per probe, which `headwater probe plan --repetitions 3` narrowed from the 30 the tier declares. It tests the tooling and measures what a session of each arm costs before the full batch. No comparison pools it with the full run, and no obligation is restated on it. The owner ruled on the scale and the timing on [#980](https://github.com/headwater-ai/headwater/issues/980).

**One batch, one tree.** Every session of the pilot, over seven selections and both arms, ran in a fresh copy of one `git archive` of commit `f035296f`, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument and every record under `docs/` that names a probe of the batch (`tools/probe/seal.sh`), and each absent workspace also lost its tier's ablation (`tools/probe/ablate.sh`). No workspace held a `.git` pointer or a built engine, so the intent hook was live in no session of either arm, which is what every earlier recording met.

**The oracle.** A `patched` artifact of either arm was checked in a copy of the present tree with the artifact written into it (`probe-transform.sh --oracle-tree`), so one oracle graded both arms.

## Events

```yaml
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L5-campaign-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -path \\\"*docs/spec/09-open-questions.md\\\" 2>/dev/null\",\"description\":\"Locate the open-questions spec file\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"Q8\\\" -A 40 docs/spec/09-open-questions.md | sed -n '1,80p'\",\"description\":\"Show Q8 section and following context\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat docs/decisions/0008-probe-cost-and-cadence.md\",\"description\":\"Show full decision doc for probe cost and cadence\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L5-campaign-absent-p1-r2"
  calls:
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p1-r2/docs/spec/09-open-questions.md"
      result: "sha256:1025b8d978b9740587ce82d3f6a4ab7beb4f0f6254e03a8d8265622d9b4390ac"
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p1-r2/docs -iname \\\"0008-probe-cost-and-cadence*\\\"\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p1-r2/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L5-campaign-absent-p1-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p1-r3 -iname \\\"09-open-questions.md\\\"\",\"description\":\"Locate the open questions spec file\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"Q8\\\" -A 60 \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p1-r3/docs/spec/09-open-questions.md\\\" | sed -n '1,80p'\",\"description\":\"Show Q8 section and following content\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p1-r3/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L5-campaign-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r1 -type f | head -200\",\"description\":\"List files in working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"scent\\\" . 2>/dev/null | grep -v '\\\\.git/'\",\"description\":\"Search repo for files mentioning 'scent'\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r1/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"scent\\\" docs/spec/02-taxonomy-model.md docs/spec/09-decisions.md docs/spec/09-open-questions.md\",\"description\":\"Search spec files for scent mentions\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L5-campaign-absent-p2-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la && echo \\\"---\\\" && find . -maxdepth 3 -type f | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"scent\\\" . 2>/dev/null\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r2/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"scent\\\" docs/spec/02-taxonomy-model.md docs/decisions/0004-relation-storage.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -A6 \\\"Q20\\\" docs/spec/09-decisions.md\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L5-campaign-absent-p2-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r3 -type f | head -200\",\"description\":\"List files in working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rli \\\"scent\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r3 2>/dev/null\",\"description\":\"Search repo for the word 'scent'\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r3/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"scent\\\" \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r3/docs/spec/02-taxonomy-model.md\\\" \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r3/docs/decisions/0004-relation-storage.md\\\"\",\"description\":\"Find scent mentions in spec 02 and decision 0004\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -A 15 \\\"Q20 \\\" \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p2-r3/docs/spec/09-decisions.md\\\"\",\"description\":\"Show Q20 entry in spec 09-decisions.md\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L5-campaign-absent-p3-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la && find . -maxdepth 3 -type f | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rIl \\\"corpus\\\" --include=\\\"*.md\\\" --include=\\\"*.py\\\" --include=\\\"*.sh\\\" --include=\\\"*.rs\\\" . 2>/dev/null | grep -v /site/ | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p3-r1/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"0049*\\\" 2>/dev/null\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p3-r1/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L5-campaign-absent-p3-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 4 | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"corpus\\\" . --include=*.py --include=*.sh --include=*.rs --include=*.md 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"0049-*\\\"; echo ---; sed -n '1,200p' docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"HW-DR-0049\\\\|derived, not stored\\\\|corpus-wide fold\\\\|never stored\\\" engine/ --include=*.rs 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls engine/crates/census/src/ engine/crates/graph/src/ 2>/dev/null; echo ---; grep -n \\\"merge=\\\" .gitattributes 2>/dev/null | head -20; echo ---; ls .githooks 2>/dev/null\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L5-campaign-absent-p3-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"corpus\\\" . --include=\\\"*.md\\\" --include=\\\"*.py\\\" --include=\\\"*.js\\\" --include=\\\"*.ts\\\" 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p3-r3/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"0049-*\\\" -path \\\"*decisions*\\\"\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L5-campaign-absent-p3-r3/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"HW-PD-0005\\\" . --include=\\\"*.md\\\" | head -5\"}"
      result: ""
  produced: []
  answer: null
```
