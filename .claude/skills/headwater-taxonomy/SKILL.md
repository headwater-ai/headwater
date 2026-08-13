---
name: headwater-taxonomy
description: Change what this repository's taxonomy declares — a kind, a facet, a shelf, a relation, an identifier scheme, a language regime, an obligation or a control. Use when `headwater new` refuses because a declaration is missing, when a document has nowhere to live or nothing to be, when a check must start or stop reading something, and whenever a change to `packages/`, `docs/taxonomies/` or `.headwater/overlay.yml` is asked for.
---

# Headwater taxonomy

A structural change is a taxonomy change: propose it, validate it, resolve it, commit the lock. Nothing in this repository invents a shelf, a kind or a facet in place.

Nothing here restates a rule that `headwater taxonomy validate` holds. If a sentence below disagrees with the engine, the engine is right and this file is stale — `.claude/skills/fixtures.sh` is what catches that.

## Where a declaration goes

Four sources resolve into one lock, in this order.

| source | what belongs there |
|---|---|
| `packages/headwater-standard/taxonomy.yml` | the invariant core, and what every adopter of the package gets |
| `docs/taxonomies/design-spec/bundle.yml` | the specification tradition: numbered parts, registers, reviews |
| `docs/taxonomies/decision-record/bundle.yml` | the decision-record tradition: one decision per document, obligation records |
| `.headwater/overlay.yml` | what is true of this repository and of no adopter |

Put a declaration at the widest source where it is true, and no wider. A bundle is a tradition that another repository could take whole. The overlay is this repository alone.

Each bundle carries a `doctrine.md` beside it that states why every declaration in it is there. A declaration added to a bundle owes a paragraph in its doctrine.

## The two commands, and why the order matters

    headwater taxonomy validate     # reads the sources, reports every rule of spec 2's list
    headwater taxonomy resolve      # writes .headwater/taxonomy.lock

**`headwater check` reads the lock and never the sources.** An edit that is not resolved changes nothing that any check sees, and `taxonomy resolve --check` is a blocking CI step that reports the stale lock. Resolve after every edit, and commit the lock in the same change as the source that produced it.

`taxonomy validate` writes nothing. It reports what each rule decided and, for a rule that decides only part of its question, what it did not decide. Read the `not decided` lines: they are the honest edge of the validator, not a passing grade.

## What the validator refuses, and what it cannot judge

The validator holds a taxonomy to spec 2's list. Three of its rules catch most first drafts.

**Kind rigidity.** A kind is what a document *is*, permanently. A kind that collides with a value of the state vocabulary is refused, and so is a bare phase adjective. `draft_decision` is a state wearing a kind's clothes.

**Facet ascertainability.** A facet with a closed value set owes `guidance` for every value, so that something states when an author picks each one. A facet that no rule reads, no expectation reads, no projection filters on and that carries no engine role is refused as unread.

**Permanence.** Every facet declares `volatility`. A `mutable` facet may not appear in an identifier pattern, a shelf path or a shelf layout, because every reference to the value dies at the change.

Four judgments are yours, because no validator makes them.

- **Is this a kind or a facet?** A kind is a species of document and a facet is a property of one. If two documents differ only in a value, they are one kind with a facet.
- **Does the shelf pattern overlap another?** Placement is primary, so a path that two shelves claim resolves to a kind by accident.
- **Is the purpose right?** Routing matches a declared purpose against the intent of a task before it matches any text, so a wrong purpose is a wrong answer at the highest-value moment.
- **Who pays for an edge?** `created_by` names an actor from a closed set, and it is a claim about the world. [OBL-repo-0105](../../../docs/obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) records two relations that name `hook` where nothing mechanical writes them. Do not write a value that no code and no procedure honors.

## Three declarations that are easy to forget

**A shelf whose files carry a number owes a `layout`.** Without one the scaffolder names a file from the title alone, and a numbered shelf gains a sibling that sorts by its first letter. `{seq:04d}-{slug}.md` reads the sequence of the identifier that the run mints, so the number is one value rather than two that can disagree. A layout placeholder that nothing fills is a refusal.

**A kind that a relation may name owes an identifier scheme.** Otherwise a document of it is neither end of any edge, `identifier.unusable` reports it on every run, and `headwater new` refuses to write it at all. The base package ships one such kind, and [OBL-repo-0107](../../../docs/obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md) records it.

**A required facet with a closed value set owes a role that determines it, or the kind cannot be scaffolded.** A prompt is not a member of a closed set, so `headwater new` refuses rather than writing a value nobody chose.

## Adding a rule

A rule is declared in the taxonomy and implemented in `engine/crates/check/`. Two bars apply and neither is negotiable.

**A check without a failing fixture does not ship.** The fixture is the evidence that the rule can fire at all. A green run over the corpus is not that evidence.

**A rule is an error when its remediation is mechanical and total, and advisory otherwise.** A contraction, a British spelling and a hard-wrapped block are errors. A sentence past the word limit, a semicolon and a stock metaphor are advisory, because the remedy for each is a rewrite. A rule that blocks on a judgment is a rule somebody disables.

Every rule reaches an obligation, and `headwater check` reports the rule that reaches none.

## The audit that nothing runs

`headwater new <kind>` over every concrete kind is a taxonomy audit, and it finds defects that no check reaches: a kind on no shelf, a kind on two, a required facet nothing determines, a scheme whose pattern cannot be read, a shelf layout with a hole in it. The check layer reads documents, so a kind with no document of it is invisible to every rule. Run the constructor after a change that touches a kind, a shelf or a scheme.

## Finish with the engine

    headwater taxonomy validate
    headwater taxonomy resolve
    headwater check
    cargo test --workspace --manifest-path engine/Cargo.toml

A taxonomy change moves the lock digest, which moves `.headwater/corpus.json` and three recorded engine fixtures. Run `headwater generate`, re-record with `HEADWATER_BLESS=1`, and read the diff.
