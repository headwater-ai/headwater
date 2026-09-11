# Draft: an issue to file on n8n-io/n8n

**Nobody has sent this.** It is a draft for a person to rewrite in their own words and to file under their own name. n8n's `CONTRIBUTING.md` section 4 requires the author to understand every line and forbids pasted model output, so sending this text unchanged is the thing that policy names. Section 1 requires a linked issue before a bug-fix pull request, which is why this file exists at all.

The measurements below were taken on 2026-09-08 against `n8n-io/n8n@master`. Re-run each one before filing. [`docs/evaluations/n8n-worked-example.md`](../../docs/evaluations/n8n-worked-example.md) holds the commands and the number that would strike each item.

Suggested title: `Three defects in .agents/ and the expression-runtime architecture doc`

---

Three small factual defects in the repository's own governing documents. Each one is checkable in a few seconds and none of them depends on any tooling outside this repository.

**1. A reference link in `packages/@n8n/expression-runtime/ARCHITECTURE.md` resolves to no path.** Line 426 writes `- [n8n workflow package](../workflow/)`. From that directory the target is `packages/@n8n/workflow/`, which does not exist. The package it means is `packages/workflow/`. The fix is `../../workflow/`.

**2. `.agents/skills/spec-driven-development/SKILL.md` names a directory that is not in the tree.** Line 8 says specs live in `.agents/specs/` and are the source of truth for architectural decisions, API contracts and implementation scope. `.agents/` holds `review-rules` and `skills` and nothing else, and `.gitignore` does not name `.agents/specs/`. Step 1 of *Before Starting Work* gives the literal command `ls .agents/specs/`, so that command fails for every feature rather than for an undocumented one. Step 3 handles a feature with no spec and does not handle a root with no specs.

I do not know which way you want this resolved. Either the directory is coming and the skill should say so, or the claim should become conditional. The pull request takes the second reading because it is the smaller change.

**3. `.agents/review-rules/README.md` names two agents where `cubic.yaml` has three.** Line 25 reads "Security and QA & DX deliberately don't link `testing/`". `cubic.yaml` declares five agents, and only Backend and Frontend link `.agents/review-rules/testing/coverage.md`. DB migrations does not either. The Layout table two paragraphs above already maps `testing/` to Backend and Frontend, so the table was updated when `db-migrations/` arrived and this sentence was not.

I am happy to send a pull request for all three together, or to drop any of them.
