# Headwater authoring conventions

This file holds only what every agent in this repository must obey, because every agent pays for it on every dispatch ([HW-PD-0001](docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md)). How each rule is enforced lives in [DEVELOPING.md](DEVELOPING.md) under *What holds this repository*.

## Work in a worktree

Call `EnterWorktree` at the start of every session in this repository, before reading or editing a file, unless the user has already put the session in a worktree another way. A session editing the checkout that Claude Code itself is running from can collide with another session doing the same, and a worktree keeps a change isolated until it is ready to commit. After `EnterWorktree`, run `git config --get core.hooksPath` and expect `.githooks`: the tool rewrites that key to an absolute path in the shared config, and a worktree with an absolute path silently runs the main checkout's hook body instead of its own.

## The name is Headwater

**Headwater** is the name of the system, not a working name (decided 2026-08-10, [Q10](docs/spec/09-decisions.md#q10--naming)). Capitalize it in prose, including compounds: Headwater's, Headwater-specific, Headwater-native.

Write it as `headwater` in lower case only when it is an identifier: the CLI verb (`headwater check`), package names (`acme/headwater-taxonomy`), the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://w3id.org/headwater/` namespace. Do not capitalize inside code spans or fenced code. Documents under `docs/reviews/` are point-in-time records and stay as written.

## No arbitrary line breaks in Markdown

Do not hard-wrap Markdown source. Write each paragraph, list item, and blockquote paragraph as one logical line. Never insert a line break for line-length reasons, and remove such breaks when you edit a file that has them.

Line breaks are structural only: blank lines between blocks, one line per list item or table row, fenced/indented code kept verbatim. A deliberate hard break inside a paragraph (rare) uses a trailing backslash, not two spaces.

## American spelling

Use American English spelling everywhere: organization, artifact, behavior, customize, judgment, license, catalog. This is the ruling that ASD-STE100 rule 1.14 requires (decided 2026-08-10); do not reintroduce British forms. Documents under `docs/reviews/` are point-in-time records and stay as written.

## Avoid stock AI phrasing

Do not use the stock metaphors and intensifiers that AI-assisted prose overuses: "load-bearing", "first-class (citizen)", "battle-tested", "north star", "delve", "seamless", "holistic", "deep dive", "leverage" as a verb, "robust" as filler. Name the concrete thing the metaphor points at instead: who depends on it, what breaks without it, what it enforces. The list a check reads is `retired_terms` in `.headwater/overlay.yml`, and this paragraph is the statement of why. Add a term in both places, with the reason in the overlay.

Exception: documents under `docs/reviews/` are point-in-time review records and stay as written.

## What a check does with these rules

Every rule above is a check that `headwater check` runs, declared in this repository's taxonomy, and no second copy of any rule lives in a script. A rule is an error when its remedy is mechanical and total: a contraction, a British spelling, a hard-wrapped block, and a retired term that names its replacement. A rule is advisory when the remedy is a rewrite: a sentence past 25 words, a semicolon, a stock metaphor, and every voice category. `.githooks/pre-commit` runs `headwater check --strict`, which fails on an error and reports everything else, and CI runs the same check on every pull request. Each clone needs these three once, and the gate prints them when a clone has not set them:

    git config core.hooksPath .githooks
    git config merge.headwater-regenerate.name "regenerate a derived artifact"
    git config merge.headwater-regenerate.driver ".githooks/merge-regenerate %O %A %B %P"

`headwater check --fix` writes the corrections the engine can derive without judgment and prints what it wrote on standard error. It leaves every finding whose remedy is a rewrite, and it refuses a file rather than half-writing one. Read the diff before you commit it.

The hook needs a built engine and fails open with one printed line when there is none. Build it with `cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked`; the newer of the two profiles answers, and `--release` is for the shipped artifact rather than a checked commit. Run `headwater check` yourself to read the advisory findings, which the hook does not print.

One harness hook you will meet: `.claude/hooks/write.sh` refuses a raw `Write` of a document under `docs/` that does not exist yet and names `headwater new`; an edit to an existing document passes. No harness hook binds. What holds a change is the commit gate and the CI job.

**Two escape hatches.** A directive on the offending block marks a deliberate exception, with the reason in the source where a reader will find it:

    …the term the cited authors use. <!-- headwater allow=language.retired_term.used scope=block until=2027-12-31 reason=false_positive note=quoting Star and Griesemer -->

`reason` is `false_positive` where the rule is wrong and `accepted_deviation` where the rule is right and the text stands anyway. `scope` is `file` or `block`. `until` is required, so no exception is permanent. The second hatch is for debt: `headwater infer --owner <name> --write` records `(document, rule)` pairs under a task in the `adoption` block of `.headwater/taxonomy.lock`, and a pending finding does not fail a strict run.

**Writing governed prose.** Invoke the `ste-editor` skill before you rewrite a document under `docs/spec/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/`, `docs/tutorials/` or `docs/process/decisions/`, and run `headwater check` on the result. The skill carries the rules that no check reads.

## The seven skills, and when each one loads

Nothing makes a skill load. A harness reads a description and a model picks, which [spec 5](docs/spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) states as a measurement rather than a property. This table is the one mechanism stronger than a description, and it costs context on every session.

| skill | invoke it before |
|---|---|
| `headwater-orient` | you search `docs/` or open a specification part to find out what this repository already says |
| `headwater-authoring` | you add or revise any document under `docs/`, and whenever `headwater new` refuses |
| `headwater-taxonomy` | you touch `packages/`, `docs/taxonomies/` or `.headwater/overlay.yml` |
| `ste-editor` | you rewrite prose under the governed shelves above |
| `headwater-engine` | the first cargo or CLI command of a session, and whenever a build fails on the toolchain |
| `repo-cleanup` | you retire a worktree, a local branch or a remote branch that a merged change left behind |
| `headwater-sweep` | you are asked to read a slice of the corpus for what no check can see |

The first two are ordered: orientation finds the document, and authoring writes one. `headwater explain` answers from the graph the engine already built, so open a specification part only after it has named the right one.

`.claude/agents/headwater-maintainer.md` reports what a change touched, what is now stale, and what the corpus is owed; it proposes and never accepts. `.claude/agents/headwater-product-owner.md` reads the whole board, owns its structure and never its scope, and `/product-owner` runs it standalone. **The value rule both serve:** work must name a reader who is not this repository. `.claude/commands/next-run.md` states it once, and every other file cites it rather than restating it.

## Filing an issue

`.github/ISSUE_TEMPLATE/issue.md` is the shape, and it is the only copy of it. Anything filing through `gh` reads that file first and follows it by hand. **An issue opens with an `## ELI5` section**: two to five sentences of plain language for a reader who has never opened this repository, with no spec numbers, rule identifiers, verb or crate names. It is written last and read first. Nothing checks this, so the template and this paragraph are the whole mechanism.

## The identifier claim store

`headwater new` mints under a `reconcile-first` scheme by reading the tree, and two branches cut from one `main` mint the same number and merge silently. `.headwater/ids/<scheme>/<identifier>` is the answer ([HW-DR-0054](docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md)): one file per claim, written once, so two branches that claim one value hit an `add/add` conflict. Three things bind a change: **a claim file is never empty**, **`.gitattributes` sets no `merge` attribute on `.headwater/ids/**`**, and **both writers create and never overwrite**. Before minting, scan every remote branch for the highest identifier in use, not only `main`.
