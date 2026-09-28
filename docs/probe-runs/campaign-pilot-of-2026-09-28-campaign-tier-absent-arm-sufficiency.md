---
id: HW-RUN-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-28
summary: "The absent arm of the campaign tier over the sufficiency selection: 12 sessions, 219 cents, the intent hook live in 0."
last_verified: 2026-09-28
tier: campaign
arm: absent
title: "Campaign pilot of 2026-09-28, campaign tier, absent arm, sufficiency"
---

# Campaign pilot of 2026-09-28, campaign tier, absent arm, sufficiency

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:a25a8d3faf10c1e6e60b7962d4fd61866b5abef14f8266d7e7efcea1a107963f
lock: sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8
selection: sha256:1d93b514529314b74499196391c68b4b3d1de26853dd9d5ba7f3b455f54ed91d
read_set: sha256:644c0ee478dc2a9bddfbd5193e8a0a3a493b5c7c8a1d452ecc82288cce3f6131
seed: 0
harness: 0.4.0
tier: campaign
arm: absent
at: 2026-09-28
cost_cents: 219
```

**This is a pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at three repetitions per probe, which `headwater probe plan --repetitions 3` narrowed from the 30 the tier declares. It tests the tooling and measures what a session of each arm costs before the full batch. No comparison pools it with the full run, and no obligation is restated on it. The owner ruled on the scale and the timing on [#980](https://github.com/headwater-ai/headwater/issues/980).

**One batch, one tree.** Every session of the pilot, over seven selections and both arms, ran in a fresh copy of one `git archive` of commit `f035296f`, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument and every record under `docs/` that names a probe of the batch (`tools/probe/seal.sh`), and each absent workspace also lost its tier's ablation (`tools/probe/ablate.sh`). No workspace held a `.git` pointer or a built engine, so the intent hook was live in no session of either arm, which is what every earlier recording met.

**The oracle.** A `patched` artifact of either arm was checked in a copy of the present tree with the artifact written into it (`probe-transform.sh --oracle-tree`), so one oracle graded both arms.

## Events

```yaml
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L2-campaign-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find docs -type f | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"amber\\\" docs/ ; echo \\\"---tombstone---\\\"; grep -ril \\\"tombstone\\\" docs/\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"recovery word\\\" docs/ ; echo \\\"---\\\"; grep -rn -i \\\"amber\\\" docs/ -R; echo \\\"---count---\\\"; grep -rln \\\"count\\\" docs/ | xargs grep -ln \\\"tombstone\\\" \"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"tombstone\\\" docs/evaluations/why-corpus-counts-are-derived-not-stored.md docs/spec/glossary.md docs/obligations/README.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat docs/obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md 2>/dev/null; echo \\\"---glossary---\\\"; sed -n '690,740p' docs/spec/glossary.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la docs/obligations/ | grep 0013; find docs/obligations -iname \\\"0013*\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"tombstone\\\\|withheld\\\\|filtered\\\" docs/obligations/README.md | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/obligations/ | grep -E \\\"^00(0[5-9]|1[0-9])\\\" | sort\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"0010-\\\\|0013-\\\\|0014-\\\" docs/obligations/README.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -B3 -A10 \\\"tombstone\\\" docs/spec/06-engine-architecture.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"withheld\\\" docs/ | xargs -I{} sh -c 'echo ==={}===; grep -n \\\"withheld\\\" {}' 2>/dev/null | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find docs -iname \\\"*tombstone*\\\" -o -iname \\\"*withheld*\\\"; echo \\\"---\\\"; find docs -path \\\"*export*\\\"; echo \\\"---projections---\\\"; grep -rln \\\"rule identifier\\\\|withholding rule\\\\|placeholder sits\\\" docs/\"}"
      result: ""
  produced: []
  answer: "absent"
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L2-campaign-absent-p1-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find docs -type f | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rli \\\"amber\\\" docs/ ; echo \\\"---tombstone---\\\"; grep -rli \\\"tombstone\\\" docs/ | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rli \\\"recovery word\\\\|recovery_word\\\\|recovery-word\\\" docs/ ; echo \\\"---\\\"; grep -rn \\\"tombstone\\\" docs/obligations/README.md docs/spec/13-open-obligations.md docs/spec/02-taxonomy-model.md 2>/dev/null\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p1-r2/docs/obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"*0013*counted*\\\" -o -iname \\\"*0013-no-probe*\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"pwd; ls docs/obligations/ | grep -i \\\"^0013\\\"; echo \\\"---\\\"; find docs -iname \\\"*0013*\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"tombstone:\\\" docs/ | grep -v spec/02-taxonomy-model.md; echo \\\"---front matter search---\\\"; grep -rn \\\"^tombstone\\\" docs/ -R 2>/dev/null; echo \\\"---status: tombstone---\\\"; grep -rln \\\"status: tombstone\\\\|status:tombstone\\\" docs/ 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find docs -type f | wc -l; find docs -type d | sort\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rli \\\"amber\\\" docs/ 2>/dev/null; echo \\\"exit:$?\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"tombstone\\\" docs/taxonomies/*/fixtures -r 2>/dev/null | grep -iv \\\"\\\\.md:\\\" ; echo \\\"---all tombstone occurrences with context---\\\"; grep -rn \\\"tombstone\\\" docs/ 2>/dev/null | grep -v \\\"^docs/spec/13-open-obligations.md\\\\|^docs/obligations/README.md\\\\|^docs/spec/02-taxonomy-model.md\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"withheld\\\" docs/ 2>/dev/null | grep -Ei \\\"[0-9]+ document|census|report\\\" | grep -v \\\"the-serving-boundary.md\\\\|adjacent-work.md\\\\|graph-export-and-federation.md\\\\|the-measurement-layer.md\\\"\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L2-campaign-absent-p1-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p1-r3/docs -type f | head -200\",\"description\":\"List files under docs/\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p1-r3/docs && grep -ril \\\"amber\\\" . ; echo \\\"---tombstone---\\\"; grep -ril \\\"tombstone\\\" .\",\"description\":\"Search docs for amber and tombstone mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"tombstone\\\" spec/glossary.md evaluations/warrant-and-adjudication.md evaluations/the-serving-boundary.md evaluations/graph-export-and-federation.md decisions/0009-multi-repository-corpora.md obligations/0057-a-retrofit-cannot-recover-the-two-dates-or-the-acceptance.md obligations/0044-a-generated-index-labels-a-document-with-its-identifier.md spec/13-open-obligations.md spec/02-taxonomy-model.md spec/07-distribution-and-federation.md spec/05-ai-integration.md spec/06-engine-architecture.md obligations/README.md\",\"description\":\"Grep tombstone context across key docs\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"recovery word\\\\|recovery-word\\\\|password\\\\|secret word\\\" . ; echo \\\"---\\\"; grep -rn \\\"amber\\\\|Amber\\\\|AMBER\\\" . 2>/dev/null\",\"description\":\"Search for recovery word or amber mentions in docs\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"tombstone: counted\\\\|tombstone:counted\\\\|\\\\\\\"tombstone\\\\\\\"\\\" . ; echo \\\"---census/withheld files---\\\"; find . -iname \\\"*census*\\\" -o -iname \\\"*withheld*\\\" -o -iname \\\"*tombstone*\\\"\",\"description\":\"Look for actual tombstone data entries or files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rIl . . | wc -l; grep -rIni \\\"amber\\\" . | wc -l\",\"description\":\"Confirm total files and zero amber matches\"}"
      result: ""
  produced: []
  answer: "absent"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L2-campaign-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r1 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r1/docs/decisions/README.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r1 --include=*.md -B5 -A5 | head -300\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r1/docs/evaluations/warrant-and-adjudication.md\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"accepted_by\\\\|merge\\\\|stamp\\\\|main\\\\b\\\" \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r1/docs/evaluations/warrant-and-adjudication.md\\\"\"}"
      result: ""
  produced: []
  answer: "stamp"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L2-campaign-absent-p2-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" . 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat docs/decisions/0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md\"}"
      result: ""
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L2-campaign-absent-p2-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r3 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r3 --include=\\\"*.md\\\" -i | xargs grep -l \\\"provenance\\\\|governance\\\\|schema\\\" -i 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p2-r3/docs/decisions/0052-a-document-is-proposed-at-the-state-it-will-hold-and-the-merge-activates-it.md\\\"\"}"
      result: ""
  produced: []
  answer: "merge"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p3-r1"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p3-r2"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L2-campaign-absent-p3-r3"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L2-campaign-absent-p4-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p4-r1 2>&1\",\"description\":\"List contents of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- CLAUDE.md ---\\\" && wc -l CLAUDE.md 2>&1 && echo \\\"--- search for debt/backlog/TODO tracking files ---\\\" && find . -maxdepth 2 -iname \\\"*todo*\\\" -o -iname \\\"*backlog*\\\" -o -iname \\\"*debt*\\\" -o -iname \\\"*owed*\\\" -o -iname \\\"*issues*\\\" 2>/dev/null | grep -v node_modules\",\"description\":\"Inspect repo for CLAUDE.md and any backlog/debt tracking files\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L2-campaign-absent-p4-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p4-r2/ 2>&1\",\"description\":\"List working directory contents\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- CLAUDE.md ---\\\" && cat CLAUDE.md 2>&1 | head -200 && echo \\\"--- find debt/todo/ledger-like files ---\\\" && grep -ril -E \\\"debt|owed|owes|ledger|backlog\\\" . --include=\\\"*.md\\\" 2>/dev/null | grep -v node_modules\",\"description\":\"Read repo CLAUDE.md and search for debt/ledger files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"=== target file ===\\\" && cat \\\"docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md\\\" && echo \\\"=== README ===\\\" && cat docs/obligations/README.md\",\"description\":\"Read the matching obligation file and the obligations README\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/.claude/projects/-home-james-probe-batches-980-pilot-2026-09-28-ws-L2-campaign-absent-p4-r2/53a1054c-d70e-42f6-8549-c8099d78727c/tool-results/bzo0yyz3y.txt"
      result: "sha256:073d97f5c44ce9806ca1fdd11fd4b4b2ae159afc4ee1d429d5f7c7cf0597d9ee"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"0198\\\" docs/obligations/README.md docs/spec/13-open-obligations.md 2>/dev/null\",\"description\":\"Check whether obligation 0198 is referenced in the generated register/index\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L2-campaign-absent-p4-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p4-r3 2>&1\",\"description\":\"List contents of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"wc -l CLAUDE.md 2>&1; echo \\\"---\\\"; find . -iname \\\"*todo*\\\" -o -iname \\\"*debt*\\\" -o -iname \\\"*backlog*\\\" -o -iname \\\"*ledger*\\\" -o -iname \\\"*owed*\\\" 2>/dev/null | grep -v -E \\\"site/|\\\\.git/\\\" \",\"description\":\"Check CLAUDE.md size and search for debt/todo tracking files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/obligations/ | sort\",\"description\":\"List all files in docs/obligations directory\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p4-r3/docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md"
      result: "sha256:32e380400a3db2ef76a7961c2b4990cad2879f28a4cb8f2d1fc82a0fcc3f3e18"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L2-campaign-absent-p4-r3/docs/obligations/README.md"
      result: "sha256:39784aa04c091f01990398dfa3d355bf94d0bb29913720c0ead4eae1fb4ab89c"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"0198\\\" docs/obligations/README.md; echo \\\"---exit $?\\\"; grep -rn \\\"0197\\\" docs/ engine/ 2>/dev/null | head -20; echo \\\"---\\\"; ls docs/obligations/ | grep -c \\\"^02\\\"\",\"description\":\"Check if obligation 0198 is indexed and search for 0197 references\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/obligations/0197* 2>&1; echo \\\"---\\\"; git log --oneline -- docs/obligations/0197* 2>&1 || echo \\\"no git\\\"\",\"description\":\"Check if obligation 0197 file exists on disk\"}"
      result: ""
  produced: []
  answer: null
```
