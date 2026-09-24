---
id: HW-HOW-relocate-a-shelf
status: draft
status_since: 2026-09-16
summary: "An overlay relocates a shelf with one override key, and the move breaks every relative link into the old path until you repair them."
last_verified: 2026-09-16
title: "Relocate a shelf"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-taxonomy-model
---

# Relocate a shelf

**Audience:** an adopter of Headwater, in a repository of their own. The consumer surface in `.headwater/overlay.yml` lists this guide, and `headwater check` holds it to that list.

## Before you start

You need three things.

- A corpus that already passes `headwater check --strict`. This guide repairs no untyped document for you.
- A built engine. `cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked` builds one.
- An overlay you can edit, and a shelf whose directory you want to move.

**Two things move in a shelf relocation, and they move in different steps.** The taxonomy moves first, at the `override`. The directory moves second, by hand. Between the two, every document on the shelf reads as untyped. After the second, every relative link that named the old path reads as broken. Both states are expected, and this guide shows you both.

This guide moves this repository's own `decisions` shelf, from `docs/decisions/**` to `docs/adr/**`. [Spec 2](../spec/02-taxonomy-model.md#the-immutable-core) names that exact move as its worked example of what an overlay may do without breaking the core. Every command below ran, for real, against a scratch copy of this repository, on 2026-09-16. Replace `decisions` and `docs/adr/**` with your own shelf's name and the path you want.

## Steps

### Add the override

An overlay's `override` key replaces a value at a path that already exists. Add one line under it, naming your shelf and the new path.

```yaml
override:
  shelves.<your-shelf>.path: <your-new-path>/**
```

This repository's own overlay had no `override` key before this guide. It only added. Here is the concrete line this guide adds.

```yaml
override:
  shelves.decisions.path: docs/adr/**
```

### Resolve the override

```sh
headwater taxonomy resolve
```

```
wrote .headwater/taxonomy.lock
  from .headwater/packages/headwater-standard/taxonomy.yml
  from .headwater/packages/headwater-standard/bundles/design-spec/bundle.yml
  from .headwater/packages/headwater-standard/bundles/evidence-and-obligation/bundle.yml
  from .headwater/packages/headwater-standard/bundles/decision-record/bundle.yml
  from .headwater/overlay.yml
  carried the adoption block through, 1 task
```

`resolve` merges the override into the lock. It reads no document.

### Check before you move one file

```sh
headwater check
```

Trimmed to one line of the census.

```
  docs/decisions/0007-scope-of-the-mcp-surface.md
    untyped: no shelf pattern claims this path
```

Every document under `docs/decisions/` now reads as untyped, because the `decisions` shelf requires `docs/adr/**`, and nothing has moved to match it. Placement decides the kind. You just changed what placement means.

### Move the directory

```sh
git mv docs/decisions docs/adr
```

```sh
headwater check --strict
```

```
446 findings
      207 ✗ error
      239 ▲ warn
```

Every document types correctly again. The exit code is `1`, and every one of the 207 errors is the same rule, `link.path.unresolved (OB-LINK-2)`. A document's own placement moved with the directory. A link written in another document's prose did not.

```
docs/spec/09-decisions.md:81:70 ✗ error
  link.path.unresolved (OB-LINK-2):
    `../decisions/0007-scope-of-the-mcp-surface.md` names no file of this
    repository: nothing stands at
    `docs/decisions/0007-scope-of-the-mcp-surface.md`
  fix: point it at a file that exists, or restore
    `docs/decisions/0007-scope-of-the-mcp-surface.md`. A document that moved
    keeps its identifier, so `headwater explain` finds where it went
```

No fix here is mechanical, so `headwater check --fix` writes none of these. A rule that named the right substitution for one link cannot know whether every author meant the old path or a different target. So it leaves the repair to you.

### Repair the broken links

Find every file that links into the old path.

```sh
grep -rl -E ']\((\.\./)+decisions/|]\(docs/decisions/' --include="*.md" docs | wc -l
```

```
66
```

Sixty-six files carried the 207 broken links between them. Rewrite the relative prefix in each one.

```sh
grep -rlZ -E ']\((\.\./)+decisions/|]\(docs/decisions/' --include="*.md" docs \
  | xargs -0 sed -i \
      -e 's#](\(\.\./\)\+decisions/#](\1adr/#g' \
      -e 's#](docs/decisions/#](docs/adr/#g'
```

```sh
headwater check --strict
```

```
  239 findings
        2 ✗ error
```

Two errors remain, and neither is a link this repair missed. Both are a link the repair got wrong. A relative prefix depends on how deep the *linking* file sits, and one substitution assumed one depth for every file. `docs/process/decisions/0004-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md` links `../decisions/0054-….md`. From `docs/process/decisions/`, `../` reaches `docs/process/`, not `docs/`, so the rewrite needs `../../adr/`, not `../adr/`. `headwater check` named both files. Fix the two paths it names, at the depth each one needs.

```sh
headwater check --strict
```

```
239 findings
```

Zero errors. The count matches this repository's own advisory baseline before the move, so the relocation added no finding of its own.

### Regenerate the shelf index

```sh
headwater generate
```

```
projections
  shelf_index docs/adr/README.md
    unchanged
```

The shelf index lists documents by their own filename, not by the shelf's path, so the move changed nothing it writes. A shelf with a document that had never been indexed before this move would print `wrote docs/adr/README.md` here instead.

## How to know it worked

```sh
headwater check --strict
echo $?
```

```
0
```

```sh
headwater explain HW-DR-0054
```

```
docs/adr/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md
  HW-DR-0054
  kind decision
    shelf `decisions` matched on `docs/adr/**`
    `decisions` is homogeneous, so placement carries the kind `decision`
```

Three facts, together, say the relocation is complete. `headwater check --strict` exits `0`. The finding count reads the same as it did before you moved anything. `headwater explain` resolves the identifier you moved to its new path, under the shelf name you never changed. The core spec 2 names holds. You renamed a directory. Every identifier, edge, and rule that named a document by identifier, rather than by path, kept working through the move.
