---
id: HW-RUN-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-sufficiency
status: deprecated
status_since: 2026-09-28
summary: "The present arm of the campaign tier over the sufficiency selection: 12 sessions, 252 cents, the intent hook live in 0. The recording is retired, because a later change moved the lock it pins."
last_verified: 2026-09-28
tier: campaign
arm: present
title: "Campaign pilot of 2026-09-28, campaign tier, present arm, sufficiency"
---

# Campaign pilot of 2026-09-28, campaign tier, present arm, sufficiency

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
arm: present
at: 2026-09-28
cost_cents: 252
```

**This is a pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at three repetitions per probe, which `headwater probe plan --repetitions 3` narrowed from the 30 the tier declares. It tests the tooling and measures what a session of each arm costs before the full batch. No comparison pools it with the full run, and no obligation is restated on it. The owner ruled on the scale and the timing on [#980](https://github.com/headwater-ai/headwater/issues/980).

**One batch, one tree.** Every session of the pilot, over seven selections and both arms, ran in a fresh copy of one `git archive` of commit `f035296f`, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument and every record under `docs/` that names a probe of the batch (`tools/probe/seal.sh`), and each absent workspace also lost its tier's ablation (`tools/probe/ablate.sh`). No workspace held a `.git` pointer or a built engine, so the intent hook was live in no session of either arm, which is what every earlier recording met.

**The oracle.** A `patched` artifact of either arm was checked in a copy of the present tree with the artifact written into it (`probe-transform.sh --oracle-tree`), so one oracle graded both arms.

**This transcript stands at `deprecated`, and the reason is the lock it pins.** The digest above is the lock of the tree that these pilot sessions met. The taxonomy change that declared the `subsystem_spec` kind on 2026-09-28 moved that lock, and the first of the five confirmations now refuses this recording. The result derived from it carries no verdict on a later tree, and its verdicts stand only in the history of that result. A reader may take no rate from it as a measurement of a tree whose lock differs from the digest above. The full campaign batch records its own sessions.

## Events

```yaml
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L1-campaign-present-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"amber\\\" docs/ 2>/dev/null; echo \\\"---tombstone---\\\"; grep -ril \\\"tombstone\\\" docs/ 2>/dev/null\",\"description\":\"Search docs/ for 'amber' and 'tombstone'\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"recovery word\\\\|recovery-word\\\" docs/ 2>/dev/null; echo \\\"---\\\"; grep -rin \\\"amber\\\" -r . --include=\\\"*.md\\\" 2>/dev/null; echo \\\"---count context---\\\"; grep -n -i \\\"tombstone\\\" docs/spec/13-open-obligations.md docs/obligations/README.md\",\"description\":\"Search for recovery word phrase and amber anywhere, and tombstone context\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rin \\\"recovery\\\" docs/ 2>/dev/null; echo \\\"---find tombstone dirs---\\\"; find docs -iname \\\"*tombstone*\\\"; echo \\\"---0013---\\\"; cat docs/obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md\",\"description\":\"Search docs for 'recovery' and tombstone files, view obligation 0013\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find docs/obligations -iname \\\"0013*\\\"; echo \\\"---\\\"; ls docs/obligations | grep 0013\",\"description\":\"Locate obligation 0013 file\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/obligations/ | head -50; echo \\\"---search tombstone ledger---\\\"; grep -rln \\\"tombstone\\\" .headwater/ 2>/dev/null; find . -iname \\\"*tombstone*\\\" -not -path \\\"*/fixtures/*\\\" 2>/dev/null\",\"description\":\"List obligations directory and search for tombstone ledger files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"=== ids/HW-OBL-0013 ===\\\"; cat .headwater/ids/obligation_record_id/HW-OBL-0013; echo \\\"=== probe.yml ===\\\"; cat .headwater/probe.yml\",\"description\":\"Show claim file and probe config for tombstone mechanism\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -B3 -A10 \\\"counted tombstone\\\" docs/spec/glossary.md docs/spec/13-open-obligations.md 2>/dev/null\",\"description\":\"Find definition of 'counted tombstone' in glossary and spec\"}"
      result: ""
  produced: []
  answer: "withheld"
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L1-campaign-present-p1-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"amber\\\" docs/ 2>/dev/null; echo \\\"---tombstone---\\\"; grep -ril \\\"tombstone\\\" docs/ 2>/dev/null\",\"description\":\"Search docs corpus for recovery word and tombstone references\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"---recovery word---\\\"; grep -rin \\\"recovery word\\\" docs/ 2>/dev/null; echo \\\"---recovery---\\\"; grep -ril \\\"recovery\\\" docs/ 2>/dev/null; echo \\\"---amber case-insensitive---\\\"; grep -rin \\\"amber\\\" docs/ 2>/dev/null; echo \\\"---counted tombstone---\\\"; grep -rin \\\"counted tombstone\\\\|tombstone.*count\\\\|count.*tombstone\\\" docs/ 2>/dev/null\",\"description\":\"Search for recovery word phrase and amber case-insensitive\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- actual withheld/placeholder entries ---\\\"; grep -rin \\\"withheld\\\" docs/ 2>/dev/null | grep -v \\\"the-serving-boundary\\\\|0017-governed\\\\|adjacent-work\\\\|the-measurement-layer\\\\|06-engine-architecture\\\\|02-taxonomy-model\\\\|README.md\\\\|13-open-obligations\\\"; echo \\\"--- find generated index files ---\\\"; find docs -iname \\\"*index*\\\" 2>/dev/null; echo \\\"--- find files with 'tombstone:' yaml field set ---\\\"; grep -rln \\\"tombstone: counted\\\\|tombstone: sealed\\\" docs/ 2>/dev/null\",\"description\":\"Search for actual tombstone placeholder instances rather than conceptual discussion\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"*census*\\\" -not -path \\\"*/fixtures/*\\\" 2>/dev/null; echo \\\"---\\\"; find . -iname \\\"*.headwater*\\\" -maxdepth 2 2>/dev/null; echo \\\"---projections dir---\\\"; find . -iname \\\"*projection*\\\" -not -path \\\"*/fixtures/*\\\" 2>/dev/null\",\"description\":\"Look for actual census or projection output files outside fixtures\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rin \\\"amber\\\" . --include=\\\"*.md\\\" 2>/dev/null | grep -v node_modules; echo \\\"done\\\"\",\"description\":\"Final broad check for amber anywhere in markdown files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r2/engine/crates/cli/fixtures && ls -la; echo \\\"---\\\"; find . -iname \\\"*recovery*\\\"; echo \\\"---withheld export dirs---\\\"; find . -maxdepth 1 -type d | grep -i \\\"export\\\\|withheld\\\\|tombstone\\\\|census\\\"\",\"description\":\"List fixture directories to check for a withheld/tombstone counterpart to answered-export\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find answered-export -type f | sort; echo \\\"=== probes ===\\\"; find probes -type f | sort\",\"description\":\"Inspect answered-export and probes fixture contents\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r2/engine/crates/cli/fixtures/answered-export && echo \\\"=== export.record ===\\\"; cat export.record; echo \\\"=== verify.py ===\\\"; cat verify.py; echo \\\"=== context.md ===\\\"; cat docs/answers/context.md; echo \\\"=== recovery.md ===\\\"; cat docs/answers/recovery.md\",\"description\":\"Read fixture files to understand the tombstone/export test scenario\"}"
      result: ""
  produced: []
  answer: "present"
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L1-campaign-present-p1-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"amber\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs 2>/dev/null\",\"description\":\"Search docs for the word amber\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3; echo \\\"--dir exists?--\\\"; ls -la \\\"$D\\\" | head -30; echo \\\"--docs exists?--\\\"; ls -la \\\"$D/docs\\\" 2>&1 | head -50\",\"description\":\"Check repo and docs directory structure\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; echo \\\"--amber case-insensitive--\\\"; grep -ril \\\"amber\\\" \\\"$D\\\"; echo \\\"--recovery word--\\\"; grep -ril \\\"recovery word\\\" \\\"$D\\\"; echo \\\"--tombstone--\\\"; grep -ril \\\"tombstone\\\" \\\"$D\\\"\",\"description\":\"Search docs for amber, recovery word, and tombstone mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; echo \\\"--recovery--\\\"; grep -rli \\\"recovery\\\" \\\"$D\\\"; echo \\\"--word (near recovery)--\\\"; grep -rn \\\"recovery\\\" \\\"$D\\\" 2>/dev/null | head -50\",\"description\":\"Search docs for recovery mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; grep -rn -i \\\"tombstone\\\" \\\"$D\\\"/decisions/0009-multi-repository-corpora.md \\\"$D\\\"/decisions/0063-*.md \\\"$D\\\"/decisions/0015-*.md \\\"$D\\\"/decisions/0017-*.md \\\"$D\\\"/obligations/README.md \\\"$D\\\"/obligations/0044-*.md \\\"$D\\\"/obligations/0057-*.md \\\"$D\\\"/spec/glossary.md \\\"$D\\\"/spec/13-open-obligations.md 2>/dev/null\",\"description\":\"Look at tombstone mentions in candidate docs\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; cat \\\"$D/obligations/0013-no-probe-tests-whether-a-counted-tombstone-stops-a-confident.md\\\"\",\"description\":\"Read the obligation document about counted tombstone probe\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; find \\\"$D/obligations\\\" -iname \\\"0013*\\\"\",\"description\":\"Find files matching obligation 0013\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; ls \\\"$D/obligations\\\" | grep -c md; grep -n \\\"0013\\\" -r \\\"$D\\\" 2>/dev/null\",\"description\":\"Count obligation files and find all references to 0013\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"D=/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p1-r3/docs; ls \\\"$D/obligations\\\" | sort | sed -n '1,20p'\",\"description\":\"List first obligations files to confirm 0013 absence\"}"
      result: ""
  produced: []
  answer: "withheld"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L1-campaign-present-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"accepted_by\\\" --include=\\\"*.md\\\" . 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"accepted_by\\\" docs/spec/*.md docs/decisions/README.md 2>/dev/null | head -30\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L1-campaign-present-p2-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p2-r2 --include=\\\"*.md\\\" -l 2>/dev/null | head -20\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p2-r2/docs/spec/ /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p2-r2/docs/process/ /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p2-r2/.headwater/ /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p2-r2/taxonomy-source/ 2>/dev/null\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L1-campaign-present-p2-r3"
  calls: []
  produced: []
  answer: "stamp"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p3-r1"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p3-r2"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L1-campaign-present-p3-r3"
  calls: []
  produced: []
  answer: "draft"
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L1-campaign-present-p4-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la && echo \\\"---git---\\\" && git status 2>&1 | head -20\",\"description\":\"Inspect actual contents of the working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/ && echo \\\"---obligations---\\\" && ls docs/obligations 2>&1 && echo \\\"---sample---\\\" && find docs/obligations -maxdepth 1 -type f | head -5\",\"description\":\"List docs directory and obligations subfolder\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p4-r1/docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md"
      result: "sha256:32e380400a3db2ef76a7961c2b4990cad2879f28a4cb8f2d1fc82a0fcc3f3e18"
    - tool: "Bash"
      argument: "{\"command\":\"sed -n '1,60p' docs/obligations/README.md && echo \\\"---grep for 0198 references---\\\" && grep -rl \\\"0198\\\" --include=\\\"*.md\\\" --include=\\\"*.lock\\\" --include=\\\"*.yml\\\" . 2>/dev/null | grep -v \\\"^./docs/obligations/0198\\\"\",\"description\":\"Check obligations README and search for references to obligation 0198\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L1-campaign-present-p4-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p4-r2/ 2>&1\",\"description\":\"List contents of the working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la .git 2>&1; echo \\\"---\\\"; git status 2>&1; echo \\\"---docs obligations---\\\"; ls docs/ 2>&1; echo \\\"---issue template---\\\"; find .github/ISSUE_TEMPLATE -type f 2>&1\",\"description\":\"Check for .git directory, docs layout, and issue template\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"== obligations dir ==\\\" && ls -la docs/obligations/ && echo \\\"== sample file(s) ==\\\" && find docs/obligations -maxdepth 2 -type f | head -20\",\"description\":\"List docs/obligations directory contents\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p4-r2/docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md"
      result: "sha256:32e380400a3db2ef76a7961c2b4990cad2879f28a4cb8f2d1fc82a0fcc3f3e18"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"0198\\\\|HW-OBL-0198\\\" docs/obligations/README.md | head -20\",\"description\":\"Search obligations register README for the 0198 entry\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L1-campaign-present-p4-r2/docs/obligations/0006-route-latency-at-a-harvesting-tier-is-unmeasured.md"
      result: "sha256:cbd52016da9b77f23922ca823680b39e2949fb0eb614707f3f4950b7e3439e2e"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"provenance\\\" docs/obligations/*.md | wc -l; echo \\\"total files:\\\"; ls docs/obligations/*.md | wc -l; echo \\\"---missing provenance---\\\"; for f in docs/obligations/*.md; do grep -q \\\"^provenance:\\\" \\\"$f\\\" || echo \\\"$f\\\"; done\",\"description\":\"Count obligation files with and without a provenance block\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"waiting_on\\\" docs/taxonomies/*.md taxonomy-source -r 2>/dev/null | head -20\",\"description\":\"Find taxonomy declaration of the waiting_on facet for obligations\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L1-campaign-present-p4-r3"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":3}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la && git status 2>&1 | head -20\",\"description\":\"Inspect working directory and git status\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- docs top-level ---\\\" && ls docs && echo \\\"--- obligations ---\\\" && ls docs/obligations 2>&1 | head -30 && echo \\\"--- looking for 'owes' or 'debt' or 'backlog' ---\\\" && grep -rlI \\\"still owes\\\\|owes it\\\\|this repository owes\\\\|backlog\\\" --include=*.md . 2>/dev/null | head -20\",\"description\":\"Explore docs structure for an obligations/debt ledger\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- obligations README ---\\\" && cat docs/obligations/README.md && echo \\\"--- sample obligation file ---\\\" && cat docs/obligations/0073-the-prior-version-still-owes-the-key-edit-that-the-clock.md\",\"description\":\"Read obligations README and a sample entry to learn the format\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/.claude/projects/-home-james-probe-batches-980-pilot-2026-09-28-ws-L1-campaign-present-p4-r3/4c19aa09-582a-45ce-b2ce-1f28f4661f67/tool-results/br211yrdq.txt"
      result: "sha256:0148e9a6c484ac38661c4d3f6c001c0f4eb0b08bbca064a3bcf08fb493142c37"
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/obligations/ | grep 0198 && echo \\\"---content---\\\" && cat docs/obligations/0198-nothing-states-how-much-of-a-session-s-budget-the-standing-instructions-consume-before-work-starts.md 2>&1\",\"description\":\"Check whether obligation 0198 file already exists and read it\"}"
      result: ""
  produced: []
  answer: null
```
