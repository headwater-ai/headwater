---
id: HW-DR-0057
status: draft
status_since: 2026-09-07
summary: "The claim store covers an identifier where the file name does not determine it: a shelf that declares a layout, or a scheme that reconciles"
last_verified: 2026-09-07
title: "A shelf layout is the second half of what the identifier claim store covers"
---

# A shelf layout is the second half of what the identifier claim store covers

## Context

The identifier claim store holds one file for each identifier a corpus spends, at `.headwater/ids/<scheme>/<identifier>`. [HW-DR-0054](0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md) ruled the store in for a `reconcile-first` scheme, and declined the extension to any other. That ruling reads the allocation the scheme declares. The property that decides the question is a property of the shelf.

A shelf that declares a `layout` writes each file name from a template. Two branches that mint one identifier then place their documents at two paths. Two paths merge clean, and no gate reads the pair until it stands on the trunk. Measured over a copy of this corpus: two branches each declaring `HW-SPEC-the-replay-contract`, at sequences 17 and 18, merge at exit 0 with nothing unmerged. `identifier.claimed_twice` reports the pair after the second merge, where the remedy is a renumber of a published identifier.

Five shelves of this repository declare a layout. Four of them carry a `reconcile-first` scheme, and the store covered those. `spec_series` carries `spec_id` and `register_id`, both of them `minted-once`, and the store covered neither. Sixteen identifiers of this corpus stood outside it: thirteen specification parts and three registers.

## Decision

**An identifier takes a claim where the file name does not determine it.** Two declarations put a corpus in that state, and the predicate is the union of them.

A shelf that declares a `layout` writes its file names from a template, so one identifier reaches two paths.

A scheme that allocates `reconcile-first` mints a sequence that every branch cut from one state reads alike, so one identifier reaches every branch.

The predicate is one function of the check layer, and no second copy of it exists. `headwater new` asks that function once, when it plans a document, and carries the answer on the plan. The rule `identifier.claim.missing` asks the same function over the taxonomy. Two copies of the predicate in a writer and in a rule cover two sets, and nothing reports the difference.

## Consequences

Sixteen identifiers of this corpus take a claim that no file held. `headwater check --fix` writes them, which is what bootstraps a store that a corpus minted before it had one. The predicate and the sixteen files land together, because a tree that carries one without the other reports sixteen errors.

Nothing the store already covered leaves it. The allocation half of the predicate stands, because a shelf with no layout can still carry a scheme that reconciles. The fixture taxonomy of the scaffolder declares that shape, and a predicate that read the shelf alone takes the claim away from it.

A rule of this engine reads a shelf layout for the first time. [HW-OBL-0106](../obligations/0106-a-shelf-layout-names-a-file-at-birth-and-no-rule-reads-it.md) records that no rule read one, and this closes that gap in one direction. A rule that holds a file name to its layout after the birth of the document is a separate question, and this ruling settles none of it.

A sequence is in no identifier, so the store says nothing about one. Two documents at one sequence, on a shelf whose layout declares a sequence, merge clean and no rule of this engine reports them.
