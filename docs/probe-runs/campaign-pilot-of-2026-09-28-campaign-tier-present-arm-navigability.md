---
id: HW-RUN-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-navigability
status: current
status_since: 2026-09-28
summary: "The present arm of the campaign tier over the navigability selection: 9 sessions, 278 cents, the intent hook live in 0."
last_verified: 2026-09-28
tier: campaign
arm: present
title: "Campaign pilot of 2026-09-28, campaign tier, present arm, navigability"
---

# Campaign pilot of 2026-09-28, campaign tier, present arm, navigability

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
arm: present
at: 2026-09-28
cost_cents: 278
```

**This is a pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at three repetitions per probe, which `headwater probe plan --repetitions 3` narrowed from the 30 the tier declares. It tests the tooling and measures what a session of each arm costs before the full batch. No comparison pools it with the full run, and no obligation is restated on it. The owner ruled on the scale and the timing on [#980](https://github.com/headwater-ai/headwater/issues/980).

**One batch, one tree.** Every session of the pilot, over seven selections and both arms, ran in a fresh copy of one `git archive` of commit `f035296f`, in one order that seed 980 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument and every record under `docs/` that names a probe of the batch (`tools/probe/seal.sh`), and each absent workspace also lost its tier's ablation (`tools/probe/ablate.sh`). No workspace held a `.git` pointer or a built engine, so the intent hook was live in no session of either arm, which is what every earlier recording met.

**The oracle.** A `patched` artifact of either arm was checked in a copy of the present tree with the artifact written into it (`probe-transform.sh --oracle-tree`), so one oracle graded both arms.

## Events

```yaml
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L4-campaign-present-p1-r1"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":5}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"Q8\\\" -A 40 \\\"docs/spec/09-open-questions.md\\\" | head -80\",\"description\":\"Locate Q8 probe cost and cadence section\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r1/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L4-campaign-present-p1-r2"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":3}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r2/docs/spec/09-open-questions.md"
      result: "sha256:1025b8d978b9740587ce82d3f6a4ab7beb4f0f6254e03a8d8265622d9b4390ac"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r2/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-an-agent-reaches-the-adjudication-from-the-document-that-lost-it"
  session: "L4-campaign-present-p1-r3"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":3}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find /home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r3/docs/spec -iname \\\"09-open-questions.md\\\"\",\"description\":\"Locate the open questions spec file\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"^## \\\" /home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r3/docs/spec/09-open-questions.md | grep -i \\\"q8\\\"\",\"description\":\"Find line number of Q8 section heading\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r3/docs/spec/09-open-questions.md"
      result: "sha256:1025b8d978b9740587ce82d3f6a4ab7beb4f0f6254e03a8d8265622d9b4390ac"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p1-r3/docs/decisions/0008-probe-cost-and-cadence.md"
      result: "sha256:862eaf8e574ca818108a9defb10a1aa30801297f8e5d704f2eec151b7a627c73"
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L4-campaign-present-p2-r1"
  calls:
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"where scent lives\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"headwater route \\\"where scent lives\\\" --root . 2>&1 | head -100\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"scent\\\" --include=*.md . 2>&1 | head -50\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p2-r1/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"scent\\\\|cue\\\" docs/spec/02-taxonomy-model.md | head -60\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L4-campaign-present-p2-r2"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":1}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git rev-parse --is-inside-work-tree 2>&1; ls -la | head -20\",\"description\":\"Check if directory is a git repository\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find .claude -maxdepth 2 -iname \\\"settings*.json\\\" -o -iname \\\"*.json\\\" | head -20\",\"description\":\"Look for Claude settings files that might configure worktree hooks\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p2-r2/.claude/settings.json"
      result: "sha256:8c9b53bc45588da25ca093ed41dfac371d5de60d1ff660cf60a74446653c4fa9"
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"scent\\\" --include=\\\"*.md\\\" . | head -50\",\"description\":\"Search markdown files for the term 'scent'\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"scent lives\\\\|lives in the\\\\|where.*scent\\\" docs/ --include=\\\"*.md\\\" -i | grep -i scent | head -50\",\"description\":\"Search for phrases about where scent lives\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-a-session-answers-from-the-register-without-opening-the-question-it-replaced"
  session: "L4-campaign-present-p2-r3"
  calls:
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"where scent lives\"}"
      result: ""
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":3}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git rev-parse --is-inside-work-tree 2>&1; ls -la | head -20\",\"description\":\"Check if directory is a git repo\"}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"which headwater 2>&1; ls engine/target 2>&1 | head; grep -ril \\\"scent\\\" . --include=\\\"*.md\\\" 2>/dev/null\",\"description\":\"Look for scent references in docs\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p2-r3/docs/decisions/0020-where-scent-lives.md"
      result: "sha256:9a6c7b0b7893386e7564e2a38a53cfdaacc53303563751be07d094bcf835c39f"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"scent\\\" docs/spec/glossary.md docs/spec/02-taxonomy-model.md docs/decisions/0004-relation-storage.md docs/spec/09-decisions.md 2>/dev/null\",\"description\":\"Search for scent mentions in spec/glossary/related decisions\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L4-campaign-present-p3-r1"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":5}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la && echo --- && git status 2>&1 | head -20\",\"description\":\"List directory contents and check git status\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"count\\\" docs/decisions docs/spec docs/process/decisions 2>/dev/null\",\"description\":\"Search decision and spec docs for mentions of \\\"count\\\"\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r1/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/\",\"description\":\"List top-level docs shelves\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/doctrine docs/how-to docs/evaluations | head -80\",\"description\":\"List doctrine, how-to, and evaluations shelf contents\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r1/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Bash"
      argument: "{\"command\":\"sed -n '1,200p' .headwater/overlay.yml | grep -n \\\"retired_terms\\\" -A 30\",\"description\":\"Show retired_terms list in overlay config\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find .headwater -iname \\\"overlay.yml\\\" -exec grep -n \\\"retired_terms\\\" -A 40 {} \\\\;\",\"description\":\"Locate overlay.yml and print retired_terms section\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"why-corpus-counts-are-derived-not-stored\\\\|HW-EVAL-why-corpus-counts-are-derived\\\" --include=\\\"*.md\\\" . | grep -v \\\"^./docs/evaluations/why-corpus-counts-are-derived-not-stored.md\\\"\",\"description\":\"Find backlinks to the existing evaluation document\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r1/docs/obligations/0124-a-probe-result-is-printed-and-never-committed-so-nothing-regenerates-one.md"
      result: "sha256:5311a5ef3ba7264cc09109d73dcc8ffb6904880d7caeec7880313adeba1e0f83"
  produced: []
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L4-campaign-present-p3-r2"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":1}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la && git status 2>&1 | head -20\",\"description\":\"Check directory contents and git status\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find docs/decisions -iname \\\"*0054*\\\" -o -iname \\\"*allocator*\\\" 2>/dev/null\",\"description\":\"Find decision 0054 about the allocator\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md"
      result: "sha256:dc39ad3508032fef66f58640206082c0f863a47c2fa2315934079e72223b1eb8"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"fold over a corpus\\\\|holds no count\\\\|derived and never stored\\\\|no count anywhere\\\" docs/ 2>/dev/null\",\"description\":\"Search for the general ruling about corpus folds and counts\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/evaluations/ && echo --- && cat .headwater/overlay.yml | grep -A 30 \\\"retired_terms\\\"\",\"description\":\"List evaluations dir and check retired terms list\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 4 -iname \\\"headwater\\\" -type f 2>/dev/null; ls engine/target/dev-release/ 2>/dev/null; which headwater 2>/dev/null; echo done\",\"description\":\"Check whether a built headwater CLI binary exists\"}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"ste-editor\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/README.md"
      result: "sha256:3a4e4e9237a6a3a51efdbc7d042747b9e3d01046874eea5c596d96a6fc6a3087"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rl \\\"drafted_by\\\" docs/evaluations/*.md | xargs grep -A2 \\\"provenance:\\\" | head -60\",\"description\":\"Check provenance field conventions across evaluation docs\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md"
      result: "sha256:5e8cf3e064214715cbca7a9cafac21871bf5baf37e87d4dd0ba39b669b9c3e1f"
    - tool: "Edit"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Edit"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Edit"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Edit"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
    - tool: "Edit"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r2/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
  produced:
    - path: "docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:dc6206618d974c49f6c272d0dc05f527b423472e13bc03f5a020c41c29157f34"
      cites:
        - "HW-DR-0049"
        - "HW-PD-0005"
      findings:
        - "language.controlled.not_met"
  answer: null
- probe: "HW-PROBE-what-a-session-writes-points-back-at-the-ruling-it-rests-on"
  session: "L4-campaign-present-p3-r3"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":5}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"git rev-parse --is-inside-work-tree 2>&1; echo \\\"---\\\"; ls -la; echo \\\"---\\\"; git worktree list 2>&1\",\"description\":\"Check git status of working directory\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"cat .claude/settings.json 2>/dev/null | head -100\",\"description\":\"Check claude settings for worktree hooks\"}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"why is a count taken over the whole corpus computed at read time rather than stored in a committed file? Looking for the decision/ruling about derived counts, regeneration, derived artifacts vs committed state, corpus-wide aggregates.\"}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-engine\",\"args\":\"need to run `headwater route` and `headwater explain` to find the ruling about why a corpus-wide count is computed at read time rather than committed to a file\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r3/docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md"
      result: "sha256:d255555868b70722079d0b561bc02aff4d595a7793080b8c557c37fe8035d976"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"HW-DR-0049\\\" --include=*.rs --include=*.md -l engine/ docs/ .headwater/ 2>/dev/null\",\"description\":\"Find existing references to HW-DR-0049 in code and docs\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot-2026-09-28/ws/L4-campaign-present-p3-r3/docs/evaluations/why-corpus-counts-are-derived-not-stored.md"
      result: "sha256:3a7be026c31e35bfd308787760aa7b8f0eacae392319439e4128e9717090f44e"
    - tool: "Bash"
      argument: "{\"command\":\"ls -la engine/target/*/headwater 2>/dev/null; ls engine/target 2>/dev/null; which headwater 2>/dev/null; cat .headwater/corpus.json 2>/dev/null | head -20\",\"description\":\"Check for a prebuilt headwater binary\"}"
      result: ""
  produced: []
  answer: null
```
