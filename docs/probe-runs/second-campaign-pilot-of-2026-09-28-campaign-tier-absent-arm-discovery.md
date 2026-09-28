---
id: HW-RUN-second-campaign-pilot-of-2026-09-28-campaign-tier-absent-arm-discovery
status: deprecated
status_since: 2026-09-28
summary: "The absent arm of the campaign tier over the discovery selection: 2 sessions, 58 cents, the intent hook live in 0. The recording is retired as a pilot, and it is graded again since #1292."
last_verified: 2026-09-28
tier: campaign
arm: absent
title: "Second campaign pilot of 2026-09-28, campaign tier, absent arm, discovery"
---

# Second campaign pilot of 2026-09-28, campaign tier, absent arm, discovery

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
arm: absent
at: 2026-09-28
cost_cents: 58
```

**This is a second pilot and not the campaign.** `tools/probe/campaign.sh` recorded it on 2026-09-28 at one repetition per probe, to test the three changes the owner ruled on [#980](https://github.com/headwater-ai/headwater/issues/980) after the first pilot. First, every workspace of both arms carries the built `headwater`, so the intent hook can run in a present-arm session. Second, a closed-set word alone on the final line is the answer. Third, the seal removes HW-OBL-0198, the answer key of the `patched` probe. No comparison pools it with the full run, and no obligation is restated on it.

**One batch, one tree.** Every session ran in a fresh copy of one `git archive` of commit `ed9e61a7`, with the engine that commit builds, in one order that seed 981 shuffled, four at a time, on `claude-sonnet-5` with a cap of 60 turns. Each workspace lost the instrument, every record under `docs/` that names a probe of the batch, and the answer keys of those probes. Each absent workspace also lost its tier's ablation.

**This transcript stands at `deprecated`, and the reason is the lock it pins.** The digest above is the lock of the tree that these pilot sessions met. The taxonomy change that declared the `subsystem_spec` kind on 2026-09-28 moved that lock, and the first of the five confirmations refused this recording until #1292. Since #1292, that confirmation reads a transcript with a moved lock when the read set of its probes did not move. The read set of this selection did not move, so the result derived from it carries verdicts again. The transcript stays at `deprecated` because it is a pilot. No comparison pools it with the full run, and a reader may take no rate from it as a measurement. The full campaign batch records its own sessions.

## Events

```yaml
- probe: "HW-PROBE-a-cold-agent-reaches-the-governing-document-through-the-corpus-descriptor"
  session: "L7-campaign-absent-p1-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 4 -not -path '*/node_modules/*' | sort | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"find . -maxdepth 2 -type d | sort; echo '---'; find . -iname '*.rs' | xargs grep -l \\\"summary\\\" -i 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"summary\\\" engine/crates/check/src/ -i | head -80\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L7-campaign-absent-p1-r1/engine/crates/check/src/facet_required.rs"
      result: "sha256:fa72bbf3fba7b435ffbf3565d1b984b7cbee8e4f287c54a7aac77d31a14b3859"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rln \\\"empty\\\\|blank\\\" engine/crates/check/src/ | sort; echo '---frontmatter---'; sed -n '1,140p' engine/crates/check/src/frontmatter.rs\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L7-campaign-absent-p1-r1/engine/crates/check/src/facet_blank.rs"
      result: "sha256:1ebf373da656429c364f31a2b3168a5b4b849ca308ed11c9079ebaabff5f8869"
    - tool: "Bash"
      argument: "{\"command\":\"cat .headwater/corpus.json 2>/dev/null | head -200\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"ls -la .headwater/ 2>&1; echo '---'; find . -maxdepth 1 -name '.headwater*'; echo '---find any corpus.json---'; find . -iname 'corpus.json' 2>/dev/null\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L7-campaign-absent-p1-r1/docs/decisions/0014-discovery-surface.md"
      result: "sha256:6eef63703f179a2d9b738bee73916ef271b405fe7cd290bf2a47cf8d2a59b830"
    - tool: "Bash"
      argument: "{\"command\":\"grep -rn \\\"facet_blank\\\\|facet::blank\\\\|Blank::over\\\\|facet.value.blank\\\" engine/crates/check/src/lib.rs engine/crates/check/src/register.rs 2>/dev/null; echo '---decision 0058---'; sed -n '1,80p' docs/decisions/0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md\"}"
      result: ""
  produced: []
  answer: null
- probe: "HW-PROBE-the-pointer-this-corpus-offers-for-a-task-is-the-document-a-session-opens"
  session: "L7-campaign-absent-p2-r1"
  calls:
    - tool: "Bash"
      argument: "{\"command\":\"ls -la /home/james/probe-batches/980-pilot2-2026-09-28/ws/L7-campaign-absent-p2-r1\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -ril \\\"shelf\\\" . --include=*.md --include=*.py --include=*.yml --include=*.yaml --include=*.json 2>/dev/null | head -50\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -rli \\\"shelf\\\" docs/spec docs/decisions 2>/dev/null\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"shelf\\\" docs/spec/02-taxonomy-model.md docs/spec/03-authoring-and-lifecycle.md docs/spec/glossary.md\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n -i \\\"identifier\\\" docs/spec/glossary.md | head -40\"}"
      result: ""
    - tool: "Bash"
      argument: "{\"command\":\"grep -n \\\"^#\\\\|identifier\\\\|Identifier\\\\|allocat\\\\|sequence\\\\|reconcile\\\" docs/spec/03-authoring-and-lifecycle.md | sed -n '1,80p'\"}"
      result: ""
    - tool: "Read"
      argument: "/home/james/probe-batches/980-pilot2-2026-09-28/ws/L7-campaign-absent-p2-r1/docs/decisions/0025-q25-where-the-namespace-goes-in-an-identifier-and-who-declares-it.md"
      result: "sha256:2a5c533802bdf87f98a2a6af7adaecd9e43882f41720a25a305e404f22eba986"
  produced: []
  answer: null
```
