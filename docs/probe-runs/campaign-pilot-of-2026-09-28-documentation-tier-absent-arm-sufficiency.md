---
id: HW-RUN-campaign-pilot-of-2026-09-28-documentation-tier-absent-arm-sufficiency
status: current
status_since: 2026-09-28
summary: "The absent arm of the documentation tier over the sufficiency selection: 12 sessions, 236 cents, the intent hook live in 0."
last_verified: 2026-09-28
tier: documentation
arm: absent
title: "Campaign pilot of 2026-09-28, documentation tier, absent arm, sufficiency"
---

# Campaign pilot of 2026-09-28, documentation tier, absent arm, sufficiency

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
tier: documentation
arm: absent
at: 2026-09-28
cost_cents: 236
```

**This is a pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at three repetitions per probe, which `headwater probe plan --repetitions 3` narrowed from the 30 the tier declares. It tests the tooling and measures what a session of each arm costs before the full batch. No comparison pools it with the full run, and no obligation is restated on it. The owner ruled on the scale and the timing on [#980](https://github.com/headwater-ai/headwater/issues/980).

**One batch, one tree.** Every session of the pilot, over seven selections and both arms, ran in a fresh copy of one `git archive` of commit `f035296f`, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument and every record under `docs/` that names a probe of the batch (`tools/probe/seal.sh`), and each absent workspace also lost its tier's ablation (`tools/probe/ablate.sh`). No workspace held a `.git` pointer or a built engine, so the intent hook was live in no session of either arm, which is what every earlier recording met.

**The oracle.** A `patched` artifact of either arm was checked in a copy of the present tree with the artifact written into it (`probe-transform.sh --oracle-tree`), so one oracle graded both arms.

## Events

```yaml
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L3-documentation-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/docs -type f | head -200\",\"description\":\"List files under docs directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/\",\"description\":\"List working directory contents\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/ && find . -iname \\\"docs\\\" -type d; echo \\\"---\\\"; find . -iname \\\"*tombstone*\\\"; echo \\\"---amber---\\\"; grep -ril \\\"amber\\\" . 2>/dev/null\",\"description\":\"Search for docs dir, tombstone files, and amber mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/ && echo \\\"=== recovery.md ===\\\"; cat engine/crates/cli/fixtures/answered-export/docs/answers/recovery.md; echo \\\"=== verify.py ===\\\"; cat engine/crates/cli/fixtures/answered-export/verify.py; echo \\\"=== dir listing ===\\\"; find engine/crates/cli/fixtures/answered-export -type f\",\"description\":\"Show contents of matched files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/ && grep -n \\\"^docs_dir\\\" mkdocs.yml; echo \\\"---mkdocs dir---\\\"; find mkdocs -maxdepth 3 -type d; echo \\\"---site dir---\\\"; find site -maxdepth 2 -type d\",\"description\":\"Check mkdocs config for docs_dir and list mkdocs/site directories\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/ && grep -n \\\"docs\\\" .gitignore\",\"description\":\"Check gitignore for docs entries\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/ && sed -n '20,35p' .gitignore\",\"description\":\"Show gitignore context around docs line\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r1/ && grep -n -i \\\"doc\\\" .gitignore\",\"description\":\"Search gitignore for any doc-related patterns\"}"
      result: ""
  produced: []
  answer: "absent"
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L3-documentation-absent-p1-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/docs -type f | head -200\",\"description\":\"List files under docs/\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/\",\"description\":\"List contents of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/ && find . -iname \\\"*tombstone*\\\" -o -iname \\\"*.tombstone\\\" 2>/dev/null; echo \\\"---\\\"; grep -ril \\\"tombstone\\\" . 2>/dev/null; echo \\\"---amber---\\\"; grep -ril \\\"amber\\\" . 2>/dev/null\",\"description\":\"Search for tombstone and amber references\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/fixtures/answered-export && find . -type f | sort\",\"description\":\"List all files in answered-export fixture\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/fixtures/answered-export/docs/answers/recovery.md"
      result: "sha256:004433da618be44f09c72b05b3e590a6188020df229c6af7e28def1b63f6640d"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/fixtures/answered-export/docs/answers/context.md"
      result: "sha256:a835a5eb984c7cf13eb2d361c8c0e844cb80e2157df006945c06ce1011e04986"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/fixtures/answered-export/verify.py"
      result: "sha256:d51d579a794cfb58d8b642093d892bc374ff02c6110f46a752b6fc65ffa615f5"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/fixtures/answered-export/export.record"
      result: "sha256:87b8513e919b9f81abfa6b1274d22f5d62a9f681301d99bf618cc7a0780864c7"
    - tool: "Bash"
      argument: "{\"command\":\"for f in .headwater/taxonomy.yml .headwater/taxonomy.lock packages/acme-answered-export/package.yml packages/acme-answered-export/taxonomy.yml; do echo \\\"=== $f ===\\\"; cat \\\"$f\\\"; echo; done\",\"description\":\"Show taxonomy config files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2 && find . -type d -iname \\\"docs\\\" 2>/dev/null\",\"description\":\"Find all directories named docs\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/fixtures/answered-export && find . -iname \\\"*.json\\\" -o -iname \\\"exports*\\\"; echo \\\"---tests referencing this fixture---\\\"; grep -rl \\\"answered-export\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2/engine/crates/cli/tests/ 2>/dev/null\",\"description\":\"Check for generated export json files and referencing tests\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r2 && for d in integrations/dashboard/fixtures/tree/docs engine/crates/cli/fixtures/check-rule-anchor/docs engine/crates/cli/fixtures/change/docs engine/crates/cli/fixtures/migration/docs engine/crates/cli/fixtures/probes/docs engine/crates/generate/fixtures/listanchor/docs engine/crates/check/fixtures/outside-root/docs; do echo \\\"=== $d ===\\\"; find \\\"$d\\\" -type f | head -20; done\",\"description\":\"List files in other docs fixture directories\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-counted-tombstone-separates-a-withheld-answer-from-an-absent-answer"
  session: "L3-documentation-absent-p1-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/docs -type f | head -200\",\"description\":\"List files under docs directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/\",\"description\":\"List working directory contents\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/ && find . -iname \\\"*docs*\\\" -maxdepth 3 -type d; echo \\\"---\\\"; grep -ril \\\"amber\\\" . 2>/dev/null; echo \\\"---tombstone---\\\"; grep -ril \\\"tombstone\\\" . 2>/dev/null\",\"description\":\"Search for docs dirs and amber/tombstone mentions\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/fixtures/answered-export/ && find . -type f | sort\",\"description\":\"List all files in the answered-export fixture\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/fixtures/answered-export/docs/answers/recovery.md"
      result: "sha256:004433da618be44f09c72b05b3e590a6188020df229c6af7e28def1b63f6640d"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/fixtures/answered-export/docs/answers/context.md"
      result: "sha256:a835a5eb984c7cf13eb2d361c8c0e844cb80e2157df006945c06ce1011e04986"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/fixtures/answered-export/verify.py"
      result: "sha256:d51d579a794cfb58d8b642093d892bc374ff02c6110f46a752b6fc65ffa615f5"
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/fixtures/answered-export/ && echo \\\"=== export.record ===\\\" && cat export.record && echo \\\"=== .headwater/taxonomy.yml ===\\\" && cat .headwater/taxonomy.yml && echo \\\"=== .headwater/taxonomy.lock ===\\\" && cat .headwater/taxonomy.lock && echo \\\"=== packages/acme-answered-export/package.yml ===\\\" && cat packages/acme-answered-export/package.yml && echo \\\"=== packages/acme-answered-export/taxonomy.yml ===\\\" && cat packages/acme-answered-export/taxonomy.yml\",\"description\":\"Show remaining fixture files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/fixtures/answered-export/ && find . -iname \\\"exports\\\" -o -iname \\\"*.json\\\" | sort; echo \\\"---\\\"; grep -rn \\\"answered-export\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p1-r3/engine/crates/cli/tests/export.rs\",\"description\":\"Check for generated export json and referencing test\"}"
      result: ""
  produced: []
  answer: "present"
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L3-documentation-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -type f | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" . 2>/dev/null | grep -v \\\"\\\\.git/\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"accepted_by\\\" taxonomy-source/ engine/crates/*/src 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"sed -n '1,120p' engine/crates/query/src/mcp.rs\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -path \\\"*docs/spec*\\\" -o -path \\\"*docs/obligations*\\\" | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"*.rs\\\" | xargs grep -ln \\\"accepted\\\" 2>/dev/null | grep -v fixtures\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r1/engine/crates/check/src/promotion.rs"
      result: "sha256:e694ff623defba242adcc5c136b78dfb0741a47779ad0b4a648fa638022b3179"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L3-documentation-absent-p2-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r2 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat engine/crates/check/fixtures/promotion/drafted.md engine/crates/check/fixtures/promotion/promoted.md engine/crates/check/fixtures/before/accepted-already.md 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"spec 3\\\\|act spec\\\\|promotion\\\" --include=*.rs engine/crates/check/ | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -path \\\"*docs/spec*\\\" 2>/dev/null; echo \\\"---\\\"; sed -n '1,140p' engine/crates/check/src/promotion.rs\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-event-that-makes-a-document-accepted"
  session: "L3-documentation-absent-p2-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"accepted_by\\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3/engine/crates/check -maxdepth 2 -type d; echo ---; ls /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3/engine/crates/check/src; echo ---; cat /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3/engine/crates/check/fixtures/promotion/drafted.md; echo ---PROMOTED---; cat /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3/engine/crates/check/fixtures/promotion/promoted.md; echo ---BEFORE---; cat /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3/engine/crates/check/fixtures/before/accepted-already.md\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p2-r3/engine/crates/check/src/promotion.rs"
      result: "sha256:e694ff623defba242adcc5c136b78dfb0741a47779ad0b4a648fa638022b3179"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L3-documentation-absent-p3-r1"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L3-documentation-absent-p3-r2"
  calls: []
  produced: []
  answer: "draft"
- probe: "HW-PROBE-a-session-names-the-status-a-settled-decision-carries-in-its-pull-request"
  session: "L3-documentation-absent-p3-r3"
  calls: []
  produced: []
  answer: "current"
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L3-documentation-absent-p4-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1 -maxdepth 4 | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1/tools/repo/obligation-register-fixtures.sh\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1\\\" && ls docs 2>/dev/null; echo \\\"---\\\"; find . -iname \\\"*obligation*\\\" -not -path \\\"*/tools/*\\\"; echo \\\"---spec13---\\\"; find . -path \\\"*spec/13*\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1\\\" && grep -ril \\\"HW-OBL\\\" . 2>/dev/null; echo \\\"---\\\"; grep -ril \\\"obligation\\\" . --include=\\\"*.md\\\" 2>/dev/null | head -50; echo \\\"---find docs dirs---\\\"; find . -maxdepth 2 -type d -iname \\\"*doc*\\\"; echo \\\"---AGENTS/README hints---\\\"; sed -n '1,200p' AGENTS.md 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1\\\" && echo \\\"== search HW-OBL ==\\\"; grep -rl \\\"HW-OBL\\\" . 2>/dev/null; echo \\\"== register.rs mentions obligations dir? ==\\\"; grep -n \\\"obligation\\\" engine/crates/check/src/register.rs 2>/dev/null | head -30; echo \\\"== AGENTS.md ==\\\"; wc -l AGENTS.md; sed -n '1,120p' AGENTS.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1\\\" && ls -la | head -30\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cd \\\"/home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r1\\\" && echo \\\"== .gitignore docs? ==\\\"; grep -n \\\"docs\\\" .gitignore; echo \\\"== README docs mentions ==\\\"; grep -n \\\"docs/\\\" README.md DEVELOPING.md | head -40; echo \\\"== CLAUDE.md anywhere? ==\\\"; find / -maxdepth 6 -iname \\\"CLAUDE.md\\\" 2>/dev/null | grep -v \\\"/.claude/\\\" \"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L3-documentation-absent-p4-r2"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r2\",\"description\":\"List contents of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la CLAUDE.md 2>&1; echo \\\"---\\\"; find . -maxdepth 2 -iname \\\"*todo*\\\" -o -iname \\\"*debt*\\\" -o -iname \\\"*backlog*\\\" -o -iname \\\"*issues*\\\" -o -iname \\\"*known*\\\" 2>/dev/null; echo \\\"---readme head---\\\"; head -40 README.md\",\"description\":\"Check CLAUDE.md symlink target and search for debt/TODO tracker files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rliE \\\"\\\\b(owe|owes|ledger|debt|gap|budget)\\\\b\\\" docs 2>/dev/null | head -30; echo \\\"---\\\"; find . -maxdepth 3 -iname \\\"*.md\\\" | xargs grep -liE \\\"\\\\bowes\\\\b\\\" 2>/dev/null\",\"description\":\"Search docs for concepts of gap/debt/ledger/budget tracking\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"=== CONTRIBUTING.md ===\\\" && grep -n -i -B2 -A5 \\\"\\\\bowes\\\\b\\\" .github/CONTRIBUTING.md; echo \\\"=== headwater-maintainer.agent.md ===\\\" && grep -n -i -B2 -A5 \\\"\\\\bowes\\\\b\\\" .github/agents/headwater-maintainer.agent.md; echo \\\"=== dashboard README ===\\\" && grep -n -i -B2 -A5 \\\"\\\\bowes\\\\b\\\" integrations/dashboard/README.md\",\"description\":\"Show context around 'owes' mentions in three files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"=== 13-open-obligations.md ===\\\" && cat docs/spec/13-open-obligations.md; echo; echo \\\"=== docs/obligations listing ===\\\" && ls docs/obligations/\",\"description\":\"Read the open-obligations spec file and list the obligations directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 1 -type d; echo \\\"---\\\"; find . -iname \\\"*obligation*\\\" 2>/dev/null; echo \\\"---\\\"; find . -type d -iname \\\"docs\\\" 2>/dev/null; find . -type d -iname \\\"spec\\\" 2>/dev/null\",\"description\":\"Find docs/spec/obligations directories anywhere in the repo\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"=== gitignore docs? ===\\\" && grep -n -i \\\"docs\\\" .gitignore .gitattributes 2>/dev/null; echo \\\"=== obligation-register-fixtures.sh ===\\\" && cat tools/repo/obligation-register-fixtures.sh; echo \\\"=== mkdocs.yml references to docs/spec or obligations ===\\\" && grep -n -i \\\"obligation\\\\|docs/spec\\\\|docs/decisions\\\" mkdocs.yml | head -20\",\"description\":\"Check gitignore for docs exclusion and inspect obligation fixtures script\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"^docs_dir\\\\|^site_dir\\\\|nav:\\\" mkdocs.yml | head -5; echo \\\"---site contents---\\\"; find site -maxdepth 2; echo \\\"---mkdocs dir contents---\\\"; find mkdocs -maxdepth 2; echo \\\"---.codex---\\\"; ls -la .codex\",\"description\":\"Check mkdocs config and site/mkdocs/.codex directories for the missing docs tree\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-records-an-unmeasured-claim-in-the-shape-this-corpus-checks"
  session: "L3-documentation-absent-p4-r3"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L3-documentation-absent-p4-r3 -maxdepth 3 2>&1 | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"debt\\\\|owe\\\\|ledger\\\\|IOU\\\" --include=\\\"*.md\\\" . 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- README ---\\\" && cat README.md | head -150\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la && echo \\\"---AGENTS.md---\\\" && cat AGENTS.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- taxonomy.yml ---\\\" && cat taxonomy-source/headwater-standard/taxonomy.yml | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- find docs ---\\\" && find . -maxdepth 4 -iname \\\"docs\\\" -o -iname \\\".headwater\\\" | grep -v node_modules && echo \\\"--- gitignore docs mentions ---\\\" && grep -n \\\"docs\\\" .gitignore && echo \\\"--- mkdocs.yml docs_dir ---\\\" && grep -n \\\"docs_dir\\\\|^nav\\\\|docs/\\\" mkdocs.yml | head -20\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"echo \\\"--- find docs/.headwater anywhere ---\\\"; find . -maxdepth 5 \\\\( -iname \\\"docs\\\" -o -iname \\\".headwater\\\" \\\\) ; echo \\\"--- gitignore mentions of docs ---\\\"; grep -n \\\"docs\\\" .gitignore; echo \\\"--- mkdocs.yml docs_dir ---\\\"; grep -n \\\"docs_dir\\\" mkdocs.yml; echo \\\"--- CLAUDE.md anywhere ---\\\"; find . -iname \\\"CLAUDE.md\\\"\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"obligation\\\" DEVELOPING.md | head -30\"}"
      result: ""
  produced: []
  answer: null
```
