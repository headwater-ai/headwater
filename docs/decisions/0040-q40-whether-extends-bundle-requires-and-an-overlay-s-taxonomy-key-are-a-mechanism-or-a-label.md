---
id: HW-DR-0040
status: current
status_since: 2026-08-30
summary: "All four keys of the family are a label rather than a mechanism today, and the specification and the meta-schema now say so in plain prose."
last_verified: 2026-08-30
title: "Q40 — Whether extends, bundle, requires and an overlay's taxonomy key are a mechanism or a label"
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-sonnet-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
    - HW-SPEC-distribution-and-federation
    - HW-OBL-0082
  governs:
    - engine/crates/meta/meta-schema.yml
---

# Q40 — Whether extends, bundle, requires and an overlay's taxonomy key are a mechanism or a label

## Context

**[#295](https://github.com/headwater-ai/headwater/issues/295) found a family of four declarations that the meta-schema validates and that the resolver never reads.** A taxonomy source's root carries `extends: <base package>, or null`. An overlay's root carries `taxonomy: <name>`, `bundle: <name>`, `extends: <base package>` and `requires: [<bundle>, ...]`. `engine/crates/meta/meta-schema.yml` shape-validates all five as a scalar or a sequence of scalars. A fresh search of `engine/crates` for each key finds no code path that compares any of them to a value the resolver holds. A trace through `engine/crates/resolve/src/operation.rs` confirms it. `OpKind::APPLICATION_ORDER` reads only `add`, `override`, `add_to`, `remove_from` and `remove`. One function comes near this family: `resolve::package::agrees` (`package.rs:294`). It holds two pairs to each other, a manifest's `package:` against a taxonomy source's `taxonomy:`, and a manifest's `version:` against a source's `version:`. It runs only on a taxonomy source's own root, never on an overlay's.

**The bar for this piece: a key earns a reader only where giving it one is a comparison against a value the resolver already holds.** It must hold that value at the point it reads the key. No new fetch, no new lock content, no new resolution capability. [#212](https://github.com/headwater-ai/headwater/issues/212) held two already-loaded `version` values to each other. [#267](https://github.com/headwater-ai/headwater/issues/267) held two already-loaded `package`/`taxonomy` name values to each other. Neither one fetched anything, and neither wrote a byte into the lock. This record applies that same bar to all five sites of the family, once, and states one reason per site for the answer it reaches.

**Three facts, found while checking the premise, shape where the bar lands.** First, every bundle this repository ships already writes `bundle:`, `extends:` and `requires:`. `packages/headwater-standard/bundles/*/bundle.yml` opens with all three, in five real files, not a hypothetical publisher's draft. A fresh read of those five files finds that all five declare `extends: headwater/standard@1.0.0` or `@3.2.0`. `packages/headwater-standard/package.yml` states the actual package is at `3.3.0` today. So every one of the five is stale, not only the two the issue's own body names. `bundle:` in every one of the five already matches its own directory name. `docs/taxonomies/diataxis/doctrine.md`, and its mirror at `packages/headwater-standard/bundles/diataxis/doctrine.md`, name the overlay-root `extends:` staleness as Finding 1, Tier 2. That finding offers two remedies: "a comparison at resolve time or the removal of the field." Second, that finding has no issue of its own. It is one instance of the open class [#350](https://github.com/headwater-ai/headwater/issues/350) names: "no rule reads a word of the library's doctrine." It sits inside the population #295's own table already covers. Third, `decision-record/bundle.yml` declares `requires: [design-spec]`, and a path that has nothing to do with `requires:` already enforces that invariant. A scratch corpus that selects `bundles: [decision-record]` alone, and omits `design-spec`, fails `taxonomy resolve` on referential integrity. `kinds.obligation_record.purpose` reads a purpose that `decision-record` alone never declares, and the refusal fires either way. What `requires:` would add is a clearer message, not a missing refusal.

## Decision

**All five declaration sites of the family are a label today, and the specification and the meta-schema say so in plain prose beside each one.** No key in the family gains a reader in this piece. Each site fails the bar for its own stated reason.

**A taxonomy source's `extends:` fails the bar because a real reader needs a resolver capability this repository has not built.** Reading it for real means fetching or locating the named base package. It means layering this source over that package as a second, transitive package. That result has nowhere to live except the lock. [HW-OBL-0082](../obligations/0082-the-lock-is-half-generated-and-half-authored-and-nothing.md) is the still-open ruling on what the lock's authored-versus-generated seam may carry. A second lock-seam question here would either duplicate that open ruling or collide with it. [#78](https://github.com/headwater-ai/headwater/issues/78) stays blocked on the same ruling either way. This is real distribution work, deferred rather than refused.

**An overlay's `extends:` fails the bar because acting on it is a design decision, not a comparison.** The base package's own resolved version is already loaded by the time a bundle is read. So the fetch cost that stops the source-root case does not apply here. But `design-spec` and `decision-record` declare `@1.0.0` against an actual base at major version 3. `brd-prd`, `standards-spec` and `diataxis` declare `@3.2.0` against the same base at `3.3.0`. The doctrine finding that first reported this names two remedies without choosing between them: "a comparison at resolve time or the removal of the field." A comparison could still take an exact-match form or a floor form, which is design work of its own. Turning either one on today would immediately break every bundle this repository ships, until each one is corrected. That choice is design work this piece does not do.

**`bundle:` at an overlay's root fails the bar on evidence rather than on mechanism.** It is the closest structural match to #267: a directory name is an independent identity a reader could hold it against. But nothing in this repository exercises a mismatch, and all five entries already agree with their own directory by inspection. A mechanism with no measured defect behind it is not what this piece builds.

**`requires:` at an overlay's root fails the bar because the invariant it documents is already enforced, on an unrelated path.** Referential integrity refuses the selection that omits `design-spec` today, as the reproduction above shows. Reading `requires:` for real would be a better message on an already-refused case, not a missing refusal. That is a smaller, different defect than the one this record answers.

**An overlay's `taxonomy:` fails the bar because there is nothing to hold it against.** No content in this repository uses the key outside the specification's own worked example and the one fixture that mirrors it. No document records what an adopter's resolved taxonomy is named, for the restatement to be checked against.

**Removing all four keys instead was considered and rejected.** The declaration set of the meta-schema's root is closed, so dropping any one of them is a breaking major bump. The meta-schema's own [0.3.0 removal of `lifecycle_regime.terminal`](../../engine/crates/meta/meta-schema.yml) is the precedent for what that costs. It would immediately refuse all five of this repository's own `bundle.yml` files on their next validation. It would force an edit to strip a bundle's own name, its author's stated base, and its author's stated dependency. That gains no reader over stating plainly that the reader is absent.

## Consequences

**This record fully resolves the diataxis doctrine's Finding 1 without an issue of its own.** That finding named "a comparison at resolve time or removal" as the two remedies it saw. Label is the third remedy, and the finding's own prose already anticipated it, in the words "nothing reports the divergence." This record is what now makes the specification report, plainly, that it does not compare and is not expected to. A one-sentence note added to that finding, in both `packages/headwater-standard/bundles/diataxis/doctrine.md` and `docs/taxonomies/diataxis/doctrine.md`, records that #295 answered it.

**This record does not touch [#350](https://github.com/headwater-ai/headwater/issues/350).** #350 names the general class: no rule reads a word of the library's doctrine, and it stays open. Only the one instance inside #295's own stated scope closes here.

**This record does not touch [#78](https://github.com/headwater-ai/headwater/issues/78) or HW-OBL-0082.** The mechanism arm for a taxonomy source's `extends:` is named and deferred, not built. No lock content changes here.

**This record does not fix the `requires:` error-message quality gap that the reproduction above found.** It also does not fix the staleness of the five `extends:` values this record found. Under the label ruling, staleness stops being a defect this piece answers for. There is nothing left for the value to be wrong against. Both are worth a small change of their own. This record stated on 2026-08-30 that neither was filed. The reason it gave was that no adopter is blocked by either, and that no fixture of this repository's own corpus exercises either one. [#579](https://github.com/headwater-ai/headwater/issues/579) filed the message-quality half on 2026-09-06, and the board carries it as work an adopter cannot proceed without. So the premise this paragraph rested on is answered the other way, and the answer is the board's rather than this record's. The change #579 asked for derives the bundle from the names the refusal reports, by resolving each unselected bundle in turn. It reads no `requires:` value, so the ruling above stands unchanged. The staleness half remains unfiled.

**This record cites a `requires:` value that [HW-DR-0044](0044-q44-whether-bundles-decompose-into-capabilities-and-assemblies-compose-practices.md) moved, and nothing connects the two records.** The Context above measures `decision-record` as declaring `requires: [design-spec]`. That bundle declares `requires: [evidence-and-obligation]` now. The reason sits at the point of the change, in the comment above the key in `docs/taxonomies/decision-record/bundle.yml`. That comment names the two addresses that moved and the record that moved them. So the label is maintained rather than abandoned, and the argument the Decision makes about it holds. What is stale is this record's copy of one value, which is the ordinary cost of citing a measurement.

**This record governs `engine/crates/meta/meta-schema.yml`.** [HW-OBL-0104](../obligations/0104-a-governs-edge-reaches-the-path-it-names-and-nothing.md) prices a `governs` edge at the one path it names. The five comments the meta-schema carries, beside the family's five declaration sites, are the whole of the mechanism this record governs.
