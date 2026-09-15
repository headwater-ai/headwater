# The taxonomy binding of this repository

Two files. `taxonomy.yml` says what this repository takes, and `overlay.yml` says how it differs. They exist because [issue #4](https://github.com/headwater-ai/headwater/issues/4) typed `docs/` against the [design-spec entry](../docs/taxonomies/design-spec/doctrine.md), and a typed corpus needs a declaration to be wrong against.

They run now. `headwater check` reads `taxonomy.yml` for the package it takes, the bundles it selects and the corpus it walks, and `headwater-resolve` merges the base package at [`.headwater/packages/headwater-standard/`](../.headwater/packages/headwater-standard/) with the design-spec bundle and this overlay. Every source is validated against the meta-schema before anything merges, and a source the meta-schema refuses stops the run. Two throwaway readers used to do this and each applied `add` operations at dotted addresses and stopped there. [The resolver](https://github.com/headwater-ai/headwater/issues/50) retired both.

## Why an overlay exists at all

One ordering constraint forced it. [Q4](../docs/spec/09-decisions.md#q4--relation-storage) rules that a relation target is an identifier and never a path. [Spec 3](../docs/spec/03-authoring-and-lifecycle.md#identifiers) gives identifiers to decisions, requirements, acceptance criteria, controls and obligations, and no document of this corpus was any of those on the day this file was written. So no document in this corpus had an identifier, and no edge could name one. The four schemes in `overlay.yml` are what the first declared edge needed. Two of those five artifacts do have a kind here now: `requirement` and `acceptance_criterion` arrived with [#398](https://github.com/headwater-ai/headwater/issues/398), and each one mints a scheme of its own.

The language regime is the second reason, and it is smaller. This repository holds its specification prose to an ASD-STE100 house profile. That rule lived in `CLAUDE.md` and in a Python script, where no check could read it, and `regimes.language.ste_house` is where a check reads it now. The [design-spec entry](../docs/taxonomies/design-spec/doctrine.md#what-this-entry-deliberately-does-not-declare) declines to bind a profile, and it names the adopter overlay as the place for one. This is that overlay.

## What these two files guess

Nothing below is settled anywhere in the specification. Each item is a guess that the typing pass had to make, and [13 — Open obligations](../docs/spec/13-open-obligations.md#design-work-that-nothing-blocks) carries the ones that matter.

| Guess | Why nothing settles it |
|---|---|
| The consumer declaration lives at `.headwater/taxonomy.yml` | [Spec 7](../docs/spec/07-distribution-and-federation.md#consuming) shows the block and fixes no path for it. It fixes `.headwater/corpus.json` and shows `.headwater/overlay.yml`, so this file follows both |
| `bundles:` names a bundle selection in that block | Spec 7 shows `profile:` and `overlay:` only. A selection has to be recorded, and the interview emits an overlay rather than this field |
| `{slug}` is a legal placeholder in a `pattern` | The meta-schema types `pattern` as a string and marks the position `gap:`. The base uses `{namespace}` and `{seq:04d}`, and neither one names a value set for placeholders |
| `allocation: minted-once` is a legal policy | The base names `reconcile-first`, which counts. A slug is not allocated by counting, no value set states the alternatives, and the meta-schema marks the position `gap:` for that reason |
| A front-matter key named `id` carries the identifier | A kind declares `identifier: {scheme: …}`, and nothing states which key holds the minted value |
| `language:` on a kind binds a language regime | The base binds `voice:` and `lifecycle:` on a kind and declares `regimes.language` beside them. No kind in the base names a language regime |
| Two kinds may share one identifier scheme | Identifier integrity requires that no two schemes admit the same string. It says nothing about two kinds that point at one scheme |
| A `corpus:` block names the root the census walks, and the paths it excludes | [Spec 6](../docs/spec/06-engine-architecture.md) says the census covers "every file under the corpus root" and never says where a repository declares that root. [Spec 7](../docs/spec/07-distribution-and-federation.md) has the descriptor carry one root per corpus, and the descriptor is generated from this declaration rather than the source of it |
| Package content is not corpus content | [13 — Open obligations](../docs/spec/13-open-obligations.md) states the gap: neither the base package nor the design-spec entry says whether a taxonomy package that lives inside the corpus it types is part of that corpus. The exclusion answers it for this repository only, with the reason in the file |

## What this is not

It is not a proposal, and none of it belongs in the base package. Every guess above is a hole in the specification that this repository stepped around to finish one pass. The meta-schema has shipped and both files validate against it, which is a narrower claim than it sounds: it says that the shapes are legal, and a position the meta-schema marks `gap:` is one where the specification still states no form. A guess that a later ruling settles should leave this file rather than stay as local custom.
