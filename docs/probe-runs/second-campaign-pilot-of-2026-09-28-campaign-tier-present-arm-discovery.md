---
id: HW-RUN-second-campaign-pilot-of-2026-09-28-campaign-tier-present-arm-discovery
status: current
status_since: 2026-09-28
summary: "The present arm of the campaign tier over the discovery selection: 2 sessions, 88 cents, the intent hook live in 2."
last_verified: 2026-09-28
tier: campaign
arm: present
title: "Second campaign pilot of 2026-09-28, campaign tier, present arm, discovery"
---

# Second campaign pilot of 2026-09-28, campaign tier, present arm, discovery

## Run identity

```yaml
model: claude-sonnet-5
served_version: claude-sonnet-5
tree: sha256:d9aad3464b048f92f47d8c5779bffee5b728699fc710d7e0af2c66363e9b51b0
lock: sha256:62144f6124525b73c49768d80ccdc41d7324fff0b39836367db4db85aa502cb8
selection: sha256:8ec515cf6d769b34f8fc840d00f902606dc575c74a4105c7599670b563be6ad5
read_set: sha256:3099b30a529a8aaa647dd29d7678bf8f22ac68c76030388aff2044074069ab05
seed: 0
harness: 0.4.0
tier: campaign
arm: present
at: 2026-09-28
cost_cents: 88
```

**This is a second pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at one repetition per probe, to test the three changes the owner ruled on [#980](https://github.com/headwater-ai/headwater/issues/980) after the first pilot. First, every workspace of both arms carries the built `headwater`, so the intent hook can run in a present-arm session. Second, a closed-set word alone on the final line is the answer. Third, the seal removes HW-OBL-0198, the answer key of the `patched` probe. No comparison pools it with the full run, and no obligation is restated on it.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `ed9e61a7`, with the engine that commit builds, in one order that seed 981 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names a probe of the batch, and the answer keys of those probes. Each absent workspace also lost its tier's ablation.

## Events

```yaml
- probe: "HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor"
  session: "L6-campaign-present-p1-r1"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":1}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"pwd && ls -la && git status 2>&1 | head -20\",\"description\":\"Check working directory and git status\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find taxonomy-source -type f | head -50\",\"description\":\"List taxonomy-source files\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"wc -l taxonomy-source/headwater-standard/*.yml && echo --- && grep -n \\\"summary\\\" -ri taxonomy-source/headwater-standard/*.yml | head -50\",\"description\":\"Check taxonomy files for summary field references\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1/taxonomy-source/headwater-standard/taxonomy.yml"
      result: "sha256:83129704f7a9cc81cb529eedff4e0bd9ccebe3ae98469f780c239aa44dab7868"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1/taxonomy-source/headwater-standard/conformance.yml"
      result: "sha256:47ef5283be6f8872d13ebf1cd12fb7bd644efd015d23c8e9071e248f8d6f5904"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"^checks:\\\\|^  checks:\\\\|checks:\\\" taxonomy-source/headwater-standard/taxonomy.yml | head -20\",\"description\":\"Search taxonomy.yml for checks section\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"required section\\\" docs/acceptance-criteria/0002-*.md engine --include=*.rs -l 2>/dev/null; echo ---; find engine -iname \\\"*.rs\\\" | xargs grep -ln \\\"heading\\\" 2>/dev/null\",\"description\":\"Find rule implementation for required-section-heading check\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1/engine/crates/check/src/sections.rs"
      result: "sha256:800d162aecc8a7916ab6c27d608a4e92f5afa28ccde57b5502c0697518e9ee39"
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1/engine/crates/check && find . -iname \\\"*.rs\\\" | sort && echo --- && grep -rn \\\"facet_required\\\\|fn facet_required\\\\|mod facet_required\\\" src/lib.rs\",\"description\":\"List check crate files and find facet_required module\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1/engine/crates/check/src/facet_blank.rs"
      result: "sha256:1ebf373da656429c364f31a2b3168a5b4b849ca308ed11c9079ebaabff5f8869"
    - tool: "Bash"
      argument: "{\"command\":\"cd /home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1 && sed -n '1,200p' docs/spec/12-check-layer.md 2>/dev/null | grep -n \\\"origin\\\\|Shape-origin\\\\|Document-origin\\\\|declared\\\" | head -60\",\"description\":\"Search check-layer spec for origin/declaration terminology\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"sed -n '40,80p' docs/spec/12-check-layer.md\",\"description\":\"Read the five-origins section of the check-layer spec\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"facet.value.blank\\\\|facet_blank\\\" engine/crates/check/tests engine/crates/cli/tests docs 2>/dev/null\",\"description\":\"Find tests referencing the facet.value.blank rule\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p1-r1/docs/decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md"
      result: "sha256:12b684170c99791eb4055003524b0ec031649ec1ef7d08e69a458c8940254e41"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"forbid\\\" taxonomy-source/headwater-standard/taxonomy.yml\",\"description\":\"Check whether any kind forbids the summary facet\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens"
  session: "L6-campaign-present-p2-r1"
  calls:
    - tool: "ToolSearch"
      argument: "{\"query\":\"select:EnterWorktree\",\"max_results\":1}"
      result: ""
    - tool: "EnterWorktree"
      argument: "{}"
      result: ""
    - tool: "Skill"
      argument: "{\"skill\":\"headwater-orient\",\"args\":\"which shelf a new document belongs on, and where its identifier comes from — kinds, shelves, identifier schemes, headwater new\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"which headwater 2>/dev/null; ls .headwater/ 2>/dev/null; cat .headwater/corpus.json 2>/dev/null | head -100\",\"description\":\"Check for headwater CLI and corpus entry points\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -iname \\\"headwater\\\" -type f 2>/dev/null; find . -path ./.git -prune -o -iname \\\"*headwater-cli*\\\" -print 2>/dev/null | head -20; ls engine 2>/dev/null\",\"description\":\"Look for a built headwater CLI binary\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"./engine/target/dev-release/headwater route \\\"Decide which shelf a new document belongs on, and where its identifier comes from.\\\" --root . 2>&1 | head -80\",\"description\":\"Run headwater route on the question\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls docs/spec/ | head -30\",\"description\":\"List specification parts\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"./engine/target/dev-release/headwater explain docs/spec/02-taxonomy-model.md --root . 2>&1 | head -60\",\"description\":\"Explain the taxonomy-model spec document\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"shelf\\\\|identifier scheme\\\\|identifier_scheme\\\" docs/spec/02-taxonomy-model.md | head -60\",\"description\":\"Search taxonomy-model spec for shelf and identifier scheme mentions\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p2-r1/docs/spec/02-taxonomy-model.md"
      result: "sha256:d82f2e97d492172a490c375208722d6ca290e1d32c049d54a38cfff019026870"
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L6-campaign-present-p2-r1/docs/spec/02-taxonomy-model.md"
      result: "sha256:d82f2e97d492172a490c375208722d6ca290e1d32c049d54a38cfff019026870"
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"reconcile-first\\\\|namespace\\\\|claim store\\\\|headwater new\\\\|\\\\.headwater/ids\\\" docs/spec/02-taxonomy-model.md docs/spec/03-authoring-and-lifecycle.md docs/spec/07-distribution-and-federation.md 2>/dev/null | head -60\",\"description\":\"Search specs for identifier allocation mechanics\"}"
      result: ""
  produced: []
  answer: null
```
