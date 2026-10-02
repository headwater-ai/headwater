---
name: headwater-taxonomy
description: Change what this repository's taxonomy declares — a kind, a facet, a shelf, a relation, an identifier scheme, a language regime, an obligation or a control. Use when `headwater new` refuses because a declaration is missing, when a document has nowhere to live or nothing to be, when a check must start or stop reading something, and whenever a change to `.headwater/taxonomy.yml`, `.headwater/overlay.yml` or a package source is asked for.
---

# Headwater taxonomy

A structural change is a taxonomy change: propose it, validate it, resolve it, commit the lock. Nothing in this repository invents a shelf, a kind or a facet in place.

Nothing here restates a rule that `headwater taxonomy validate` holds. If a sentence below disagrees with the engine, the engine is right and this file is stale.

## Where a declaration goes

`.headwater/taxonomy.yml` is the consumer declaration. It names the package this repository takes and the bundles it selects, and those resolve into one lock with the overlay.

| source | what belongs there |
|---|---|
| the package | the invariant core, and what every adopter of the package gets. A copy vendored under `.headwater/packages/` is read-only. Change the package where it is published, and vendor it again |
| a bundle the declaration selects | a tradition that another repository could take whole |
| `.headwater/overlay.yml` | what is true of this repository and of no other |

Put a declaration at the widest source where it is true, and no wider. A repository that takes a package it does not publish puts every change in the overlay.

## The two commands, and why the order matters

    headwater taxonomy validate     # reads the sources, and reports every rule of the taxonomy model
    headwater taxonomy resolve      # writes .headwater/taxonomy.lock

**`headwater check` takes its declarations from the lock and never from the sources.** An edit that is not resolved changes no declaration that any check sees, and `taxonomy resolve --check` reports the stale lock, so run it in CI. Resolve after every edit, and commit the lock in the same change as the source that produced it.

`taxonomy validate` writes nothing. It reports what each rule decided and, for a rule that decides only part of its question, what it did not decide. Read the `not decided` lines: they are the honest edge of the validator, not a passing grade.

## What the validator refuses, and what it cannot judge

The validator holds a taxonomy to the rules of the taxonomy model. Three of its rules catch most first drafts.

**Kind rigidity.** A kind is what a document *is*, permanently. A kind that collides with a value of the state vocabulary is refused, and so is a bare phase adjective. A kind named for a draft is a state wearing a kind's clothes.

**Facet ascertainability.** A facet with a closed value set owes `guidance` for every value, so that something states when an author picks each one. A facet that no rule reads, no expectation reads, no projection filters on and that carries no engine role is refused as unread.

**Permanence.** Every facet declares `volatility`. A `mutable` facet may not appear in an identifier pattern, a shelf path or a shelf layout, because every reference to the value dies at the change.

Four judgments are yours, because no validator makes them.

- **Is this a kind or a facet?** A kind is a species of document and a facet is a property of one. If two documents differ only in a value, they are one kind with a facet.
- **Does the shelf pattern overlap another?** Placement is primary, so a path that two shelves claim resolves to a kind by accident.
- **Is the purpose right?** Routing matches a declared purpose against the intent of a task before it matches any text, so a wrong purpose is a wrong answer at the highest-value moment.
- **Who pays for an edge?** `created_by` names an actor from a closed set, and it is a claim about the world. A relation that names `hook` where nothing mechanical writes it is a promise nobody keeps. `headwater new --relates` writes an edge whose relation names `scaffold` or `agent`, and it refuses every other actor. Do not write a value that no code and no procedure honors.

## Three declarations that are easy to forget

**A shelf whose files carry a number owes a `layout`.** Without one the scaffolder names a file from the title alone, and a numbered shelf gains a sibling that sorts by its first letter. `{seq:04d}-{slug}.md` reads the sequence of the identifier that the run mints, so the number is one value rather than two that can disagree. A layout placeholder that nothing fills is a refusal.

**A kind that a relation may name owes an identifier scheme.** Otherwise a document of it is neither end of any edge, `identifier.unusable` reports it on every run, and `headwater new` refuses to write it at all. Its refusal prints the overlay lines that declare a scheme for the kind.

**A required facet with a closed value set owes a role that determines it, or the kind cannot be scaffolded.** A prompt is not a member of a closed set, so `headwater new` refuses rather than writing a value nobody chose.

## A rule is an error only where its remedy is mechanical

**A rule is an error when its remediation is mechanical and total, and advisory otherwise.** A contraction, a British spelling and a hard-wrapped block are errors. A sentence past the word limit, a semicolon and a stock metaphor are advisory, because the remedy for each is a rewrite. A rule that blocks on a judgment is a rule somebody disables.

Every rule reaches an obligation, and `headwater check` reports the rule that reaches none.

## The sweep that nothing runs

`headwater new <kind>` over every concrete kind finds defects that no check reaches: a kind on no shelf, a kind on two, a required facet nothing determines, a scheme whose pattern cannot be read, a shelf layout with a hole in it. The check layer reads documents, so a kind with no document of it is invisible to every rule. Run the constructor after a change that touches a kind, a shelf or a scheme, in a scratch copy of the repository, because each run that succeeds writes a document.

## What the audit tells you that no rule does

`headwater taxonomy audit` measures the taxonomy against the corpus, and it answers the question this skill cannot: whether a declaration you added earns its place. A facet whose documents all share one value separates nothing. A relation captured by 1% of its eligible documents receives no maintenance. A creator that no relation declares is an arm that a comparison over this corpus cannot fill.

It gates nothing and it exits 0 whatever it finds, because nothing declares what any of those numbers would have to cross. So read it and decide. No run of it will decide for you.

## Finish with the engine

    headwater taxonomy validate
    headwater taxonomy resolve
    headwater taxonomy audit
    headwater check
    headwater generate

A taxonomy change moves the lock digest, which moves every generated file that states it. Run `headwater generate` and read the diff.
