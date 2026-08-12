# The taxonomy binding of this repository

Two files, and neither one runs. `taxonomy.yml` says what this repository takes, and `overlay.yml` says how it differs. They exist because [issue #4](https://github.com/headwater-ai/headwater/issues/4) typed `docs/` against the [design-spec entry](../docs/taxonomies/design-spec/doctrine.md), and a typed corpus needs a declaration to be wrong against.

`tools/abox-check.py` reads both files, resolves them over the base package and the design-spec bundle, and checks the corpus. That script is the only reader either file has today.

## Why an overlay exists at all

One ordering constraint forced it. [Q4](../docs/spec/09-decisions.md#q4--relation-storage) rules that a relation target is an identifier and never a path. [Spec 3](../docs/spec/03-authoring-and-lifecycle.md#identifiers) gives identifiers to decisions, requirements, acceptance criteria, controls and obligations, and a document is none of those. So no document in this corpus had an identifier, and no edge could name one. The four schemes in `overlay.yml` are what the first declared edge needed.

The language regime is the second reason, and it is smaller. This repository holds its specification prose to an ASD-STE100 house profile, and `tools/ste-lint.py` enforces it. That rule lives in `CLAUDE.md`, where no check can read it. The [design-spec entry](../docs/taxonomies/design-spec/doctrine.md#what-this-entry-deliberately-does-not-declare) declines to bind a profile, and it names the adopter overlay as the place for one. This is that overlay.

## What these two files guess

Nothing below is settled anywhere in the specification. Each item is a guess that the typing pass had to make, and [13 — Open obligations](../docs/spec/13-open-obligations.md#design-work-that-nothing-blocks) carries the ones that matter.

| Guess | Why nothing settles it |
|---|---|
| The consumer declaration lives at `.headwater/taxonomy.yml` | [Spec 7](../docs/spec/07-distribution-and-federation.md#consuming) shows the block and fixes no path for it. It fixes `.headwater/corpus.json` and shows `.headwater/overlay.yml`, so this file follows both |
| `bundles:` names a bundle selection in that block | Spec 7 shows `profile:` and `overlay:` only. A selection has to be recorded, and the interview emits an overlay rather than this field |
| `{slug}` is a legal placeholder in a `pattern` | The meta-schema does not exist. The base uses `{namespace}` and `{seq:04d}`, and neither one names a value set for placeholders |
| `allocation: minted-once` is a legal policy | The base names `reconcile-first`, which counts. A slug is not allocated by counting, and no value set states the alternatives |
| A front-matter key named `id` carries the identifier | A kind declares `identifier: {scheme: …}`, and nothing states which key holds the minted value |
| `language:` on a kind binds a language regime | The base binds `voice:` and `lifecycle:` on a kind and declares `regimes.language` beside them. No kind in the base names a language regime |
| Two kinds may share one identifier scheme | Identifier integrity requires that no two schemes admit the same string. It says nothing about two kinds that point at one scheme |

## What this is not

It is not a proposal, and none of it belongs in the base package. Every guess above is a hole in the specification that this repository stepped around to finish one pass. When the meta-schema ships, the ones it settles should leave this file rather than stay as local custom.
