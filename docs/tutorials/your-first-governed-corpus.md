---
id: HW-TUT-your-first-governed-corpus
status: current
status_since: 2026-09-06
summary: "An untyped repository reaches a passing strict check in sixteen steps, and the reader leaves holding shelf, kind, facet, overlay and obligation."
last_verified: 2026-09-09
title: "Your first governed corpus"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0016
---

# Your first governed corpus

This tutorial takes an empty directory to a repository that passes `headwater check --strict`. It teaches the model as it goes. Every step ends in a state that a command reports, so no step asks you to judge the result by eye.

**Every command below was run, and every output block below is what that run printed.** The date of the run is 2026-09-09.

## Before you start

You need four things.

- `git`, and a name and an email address configured in it.
- A Rust toolchain, version 1.90 or later, for `cargo install`. The floor is the highest `rust-version` in the resolved dependency graph, and `engine/Cargo.toml` declares it.
- `curl`, for step 3.
- About twenty minutes.

Three facts about the blocks below.

- **A block is a command or it is output, and the two look the same.** Every step gives the command first and what it printed after. Nothing marks the difference, so read the sentence above a block before you paste it.
- **Your dates differ.** The engine reads a clock, and it puts the date of your run into what it writes. Where a block below shows `2026-09-09`, yours shows the day you read this.
- **Long output is trimmed.** A block that is shorter than the real output says so on the line above it.

Install the engine once.

```
cargo install headwater-cli
```

**Check.** `headwater --version` prints a number.

`cargo install` puts the binary in `cargo`'s own bin directory. `cargo` already put that directory on your `PATH` when it set itself up. Nothing here needs exporting, and nothing has to survive into a new shell.

This installs whatever the newest published version is, from crates.io: no clone, and no local build. It does not get you `taxonomy-source`, the package this tutorial's step 3 needs. That package arrives the same way: fetched rather than cloned. Step 3 shows how. The README's *Obtaining a named version* section names the source-build route instead. It is for a reader who wants the engine and the package matched to one commit.

## Steps

### Step 1 — Make a repository that nothing describes

```
mkdir -p ~/headwater-tutorial/docs/decisions
cd ~/headwater-tutorial
git init
printf '# Store attempts in Postgres\n\nThe queue keeps every delivery attempt in Postgres.\n' > docs/decisions/postgres-note.md
```

**Check.** `ls docs/decisions` prints one line:

```
postgres-note.md
```

A repository with documentation in it, and nothing that says what the documentation is. Every adopter starts here.

### Step 2 — Ask the engine what it can read off the tree

```
headwater init
```

```
wrote .headwater/taxonomy.yml
wrote .headwater/overlay.yml

what this read off the tree
  corpus root docs
  package headwater/standard is not under `packages/`, and nothing here fetches one. Two routes reach a lock, and each one needs a different field of `.headwater/taxonomy.yml`. Copy a package directory into `packages/`, and pin `taxonomy.version` at the version that package declares. Or run `headwater taxonomy vendor <dir>` on a published artifact: that verb reads `taxonomy.digest` and refuses until it holds the digest the publisher printed, and `headwater taxonomy resolve` reads `taxonomy.version` after it, so the vendor route needs the digest first and the version as well

what it cannot read off a tree, and asked instead
  the phrases each purpose answers, which decide what a task routes to
  the identifier scheme, and what its prefix discriminates
  which bundles this corpus already follows

Answer them in .headwater/overlay.yml, then run `headwater taxonomy resolve` and `headwater infer`
```

**Check.** `ls .headwater` prints `overlay.yml` and `taxonomy.yml`.

The first heading names what a tree states about itself: the **corpus** is `docs`, because that directory holds the most Markdown. The second names what no tree states, and those three questions are the interview. The verb asks about your documents, and never about the model — the difference [spec 7](../spec/07-distribution-and-federation.md) draws.

### Step 3 — Fetch the package into your tree

```
curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/main/tools/headwater-bootstrap.sh | sh -s -- --tag v0.1.0 --expect sha256:733b3b4029800b203d53ad8f656294fbcb00bf802904ffed193a6102cfb84163
```

Trimmed to the account of what the script fetched.

```
headwater-bootstrap: fetching packages/headwater-standard at v0.1.0
vendored headwater/standard 4.1.0
  31 files, all of them the pinned bytes
  digest sha256:733b3b4029800b203d53ad8f656294fbcb00bf802904ffed193a6102cfb84163
  doctrine at packages/headwater-standard/doctrine/
```

**Check.** `ls packages/headwater-standard` prints seven lines:

```
assemblies
bundles
conformance.yml
doctrine
package.yml
release.yml
taxonomy.yml
```

A **package** carries a taxonomy: the kinds, the facets, the shelves and the rules. `headwater/standard` is the base package, and the taxonomy it declares is deliberately small. Four of the seven entries are not taxonomy at all. `assemblies/` holds the publisher recipes this package ships. `bundles/` holds optional traditions nothing here selects. `doctrine/` holds the prose that explains them to a person. `release.yml` is the publish record that carries the digest this fetch just checked. Nothing you run in this tutorial reads any of the four.

Step 2 named two routes, and this step took the second. `headwater taxonomy vendor` installs a **published artifact**, which is what `headwater taxonomy publish` writes. This fetch just checked it, file by file, against the digest you passed. `headwater-bootstrap.sh` is the harness around that verb, and not a part of this engine. It fetches a tagged release into a scratch directory nothing here keeps, extracts one package, and hands the result to `vendor`. Nothing under `engine/` opened a socket to get it. `vendor` itself takes a path and never a location. The script is what reached the network on `vendor`'s behalf.

The version this pulled, `4.1.0`, is one behind `taxonomy-source/package.yml` on the moving default branch, currently `4.2.0`. `v0.1.0` is the newest tag this package is fetchable from. The README's *Obtaining a named version* section says more about why. A second tag route, `taxonomy/headwater-standard/v<version>`, publishes this package alone. It needs no new engine tag. The same README section names it. Step 5 pins this version, and it pins the digest this block already printed. Step 16 reads back what the second pin buys.

### Step 4 — Meet the first refusal

```
headwater taxonomy resolve
```

```
headwater: the taxonomy did not resolve, so no lock is possible
  .headwater/taxonomy.yml: this takes headwater/standard 0.0.0, and the package here is 4.1.0
```

**Check.** `echo $?` prints `1`.

A refusal here is the design and not a fault. A **lock** is a validated taxonomy, so a taxonomy that does not validate produces no lock at all. `headwater init` wrote `version: 0.0.0`, because it had no package in front of it to read a number from.

### Step 5 — Pin the version and the digest, and meet the second refusal

Open `.headwater/taxonomy.yml`. Change `  version: 0.0.0` to `  version: 4.1.0`, and change `  # digest: sha256:<the digest the publisher printed>` to the digest step 3 printed: `  digest: sha256:733b3b4029800b203d53ad8f656294fbcb00bf802904ffed193a6102cfb84163`.

**Check.** `grep -E 'digest:|version:' .headwater/taxonomy.yml` prints two lines:

```
  digest: sha256:733b3b4029800b203d53ad8f656294fbcb00bf802904ffed193a6102cfb84163
  version: 4.1.0
```

Now resolve again.

```
headwater taxonomy resolve
```

```
headwater: the taxonomy does not validate, so no lock is written. A lock is a validated taxonomy or it is nothing
  the resolved taxonomy `identifier_schemes.decision_id`: identifier integrity: carries no namespace after resolution. A package leaves the namespace to the corpus that adopts it, so an overlay of this corpus has to declare one
```

**Check.** `echo $?` prints `1`.

The message names the second thing a package cannot know. A package that shipped a namespace would give every adopter the same one, so it ships none and refuses until you supply yours. The digest plays no part in this refusal. `headwater taxonomy resolve` reads the version and never the digest, so pinning it here changes nothing about what you see next. It matters starting at step 16.

### Step 6 — Answer the question in the overlay

An **overlay** is what your repository says on top of the package it takes. Open `.headwater/overlay.yml`, and replace the last line, `add: {}`, with these two lines:

```
add:
  identifier_schemes.decision_id.namespace: ACME
```

**Check.** `tail -2 .headwater/overlay.yml` prints those two lines back.

The overlay answers the question the refusal asked. Resolve again.

```
headwater taxonomy resolve
```

```
wrote .headwater/taxonomy.lock
  from packages/headwater-standard/taxonomy.yml
  from .headwater/overlay.yml
  no lock was there, so there was no adoption block to carry
```

**Check.** `ls .headwater/taxonomy.lock` prints the path.

The lock is the one file every later command reads. An edit to the package or to the overlay reaches nothing until you resolve again, and that is the trap that catches most newcomers.

### Step 7 — Check a corpus that nobody has typed

```
headwater check
```

Trimmed. The run prints the census, the graph, the rules, the register and the findings, and these are the lines that matter here:

```
census
  1 files under the corpus root
        1 untyped

  docs/decisions/postgres-note.md
    untyped: no front matter, so nobody has typed this file
```

```
  1 seen, 0 classified, 0 checked, 6 check instances
```

```
  0 findings
```

**Check.** `headwater check --strict > /dev/null 2>&1; echo $?` prints `0`.

**The most important state in the tutorial.** The run is green, and it is green because it checked nothing: one file seen, none classified, none checked. A green run over an untyped corpus measures your taxonomy rather than your documents. Read the census before the verdict, on every run, forever.

### Step 8 — Type the document

Delete the note, and let the engine write the document in its place.

```
rm docs/decisions/postgres-note.md
headwater new decision --title "Store attempts in Postgres" --summary "The queue keeps every delivery attempt in Postgres."
```

Trimmed to the first block. The run also prints the relations this document may declare, and a capture-cost reading:

```
wrote docs/decisions/0001-store-attempts-in-postgres.md

what the taxonomy decided
  kind decision on the shelf `decisions`
  identifier ACME-DR-0001 under `decision_id`, allocation reconcile-first
  status — `regimes.lifecycle.standard` opens at `draft`
  status_since — the facet is in the `state_entered` role, and the run's clock is the date
  summary — the facet is in the `scent` role, and `--summary` is the sentence
  last_verified — the facet is in the `freshness` role, and the run's clock is the date
  section `Context` — the kind requires it
  section `Decision` — the kind requires it
  section `Consequences` — the kind requires it
```

**Check.** `cat docs/decisions/0001-store-attempts-in-postgres.md` prints this, with your own dates:

```
---
id: ACME-DR-0001
status: draft
status_since: 2026-09-09
summary: "The queue keeps every delivery attempt in Postgres."
last_verified: 2026-09-09
---

# Store attempts in Postgres

## Context

TODO write this section.

## Decision

TODO write this section.

## Consequences

TODO write this section.
```

Three words of the model arrive in that one block, and the verb names each one as it uses it.

A **shelf** is a region of the tree that carries a purpose, and `docs/decisions/**` is the shelf named `decisions`. Placement is the loudest signal a document sends, so a shelf decides first and metadata never contradicts it.

A **kind** is what a document permanently is. The `decisions` shelf holds one kind, so placement alone settled that this file is a `decision`. A kind names the facets a document must declare, the sections it must carry, and the relations it may declare.

A **facet** is a property of one document, declared in the front matter. `status`, `status_since`, `summary` and `last_verified` are the four that `decision` requires. Three of them were filled by a role rather than by a guess. `status` came from the lifecycle regime, and two dates came from the run's clock. `summary` carries no role of its own. `--summary <text>` states it directly, exactly as `--title` states the name. Left unstated, the field carries a prompt for a person to answer.

### Step 9 — Check again, and read the difference

```
headwater check
```

Trimmed to the two lines that moved:

```
census
  1 files under the corpus root
        1 typed
        1 typed decision
```

```
  1 seen, 1 classified, 1 checked, 18 check instances
```

**Check.** `headwater check 2>/dev/null | grep 'check instances'` prints the second block above.

The corpus did not grow. The count of checks that ran went from 4 to 17, because a typed document is a document that rules can reach. That is the whole trade this system asks for: type a document, and seventeen questions become answerable about it.

### Step 10 — Commit the first governed corpus

Git refuses a commit from an author it cannot name. `git config user.name` prints yours. If it prints nothing, run `git config --global user.name "Your Name"` and then `git config --global user.email "you@example.com"` with your own values.

```
git add -A
git commit -m "A first governed corpus"
```

**Check.** `git log --oneline` prints one line that ends in `A first governed corpus`. `git ls-files .headwater` prints six files: the five you have already met, and the `.gitignore` that `headwater check` writes inside `.headwater/cache/`.

```
.headwater/cache/.gitignore
.headwater/capture-cost.jsonl
.headwater/ids/decision_id/ACME-DR-0001
.headwater/overlay.yml
.headwater/taxonomy.lock
.headwater/taxonomy.yml
```

**The file under `ids/` is the claim on the identifier that `headwater new` minted.** `decision_id` allocates reconcile-first, so the verb reads the tree for the highest value already spent. A tree tells it nothing about the branch somebody else holds. The claim file says the number is taken, and it holds the path of the document that took it. Two branches that mint one number add one path with two different contents, so the merge refuses and names both documents. Commit it with the document, and never write over one.

`.headwater/capture-cost.jsonl` is the one that takes a decision rather than a rule. It holds one reading per document that `headwater new` wrote, and it names no person and no agent. Headwater's own repository commits it, so that `headwater capture` can trend it. Keep it or ignore it, and know that you chose.

### Step 11 — Declare an edge between two documents

```
headwater new decision --title "Deliver at least once" --relates supersedes=ACME-DR-0001 --summary "Retrying a delivery is safe, so the queue may send one attempt twice."
```

Trimmed to the two blocks that matter:

```
wrote docs/decisions/0002-deliver-at-least-once.md
edited docs/decisions/0001-store-attempts-in-postgres.md
```

```
the edges it proposed
  supersedes ACME-DR-0001 — `created_by: scaffold`, so a scaffold pays for it
    the far half `superseded_by` went into docs/decisions/0001-store-attempts-in-postgres.md, because reciprocity is required
```

**Check.** `headwater check --strict > /dev/null 2>&1; echo $?` prints `0`.

A **relation** is a typed edge between two documents, and it names its target by identifier rather than by path. A path dies at the first rename and an identifier does not. `supersedes` requires both ends, so the verb wrote the far half into the target for you. Two documents now carry one edge between them.

### Step 12 — Break one half, and read the finding

```
git checkout -- docs/decisions/0001-store-attempts-in-postgres.md
headwater check
```

Trimmed to the findings block:

```
  1 findings
        1 ✗ error

  docs/decisions/0002-deliver-at-least-once.md:9:7 ✗ error
    relation.reciprocity.missing (OB-REL-1): `ACME-DR-0002` declares
      `supersedes: ACME-DR-0001`, and `supersedes` requires both ends, so
      docs/decisions/0001-store-attempts-in-postgres.md owes `superseded_by`
    fix (mechanical): add `superseded_by: ACME-DR-0002` under `relations:` in
      docs/decisions/0001-store-attempts-in-postgres.md
```

**Check.** `headwater check --strict > /dev/null 2>&1; echo $?` prints `1`.

The `git checkout` put the target back to the version you committed, which is the version before the far half arrived. One edge now has one end.

Read the identifier in parentheses. An **obligation** is a claim this system makes about itself, with the rule that verifies it named beside it. `OB-REL-1` is the obligation that this rule discharges, and every finding names the one it serves.

Trimmed to the head of the register and to its last line. The same run printed both, above the findings:

```
  register
    31 obligations: 31 verified, 0 gap, 0 unverifiable, 0 with no disposition
       13 high, 13 verified
       13 medium, 13 verified
        5 low, 5 verified
```

```
    facet.value.blank reaches no obligation, so it names none
```

**Check.** `headwater check 2>/dev/null | grep 'obligations:'` prints the `31 obligations:` line of the register block above, and nothing else.

**One of those readings is about the package and one is about your run.** The thirty-one obligations and their severities come from `headwater/standard`. They read the same on your first day and on your thousandth. The last line is derived from the run in front of you. It names every rule that fired at you with no obligation behind it. Most rules carry one, which is what makes the identifier in your finding worth reading.

One rule is named there. `facet.value.blank` reports a facet a document declares and leaves empty, and no control in `headwater/standard` names it yet. [HW-OBL-0170](https://github.com/headwater-ai/headwater/blob/main/docs/obligations/0170-the-blank-facet-value-rule-reaches-no-obligation-so-the-report-names-none.md) records that debt. A finding it raises is a true finding, and the line above is how a report tells you which of its rules answers to nothing.

**Your corpus will not show you a gap, and the reason is worth knowing.** A `gap` is a disposition that a package author writes, with an owner, for an obligation that no mechanism verifies. The base package declares none, so this row reads `0 gap` on every run of yours and no step here moves it. Headwater's own corpus takes a bundle that declares three obligations no mechanism verifies. The same block there reads `34 obligations: 31 verified, 2 gap, 1 unverifiable`, and it names the owner of each gap. The number worth watching is the one that is not `verified`.

The word `mechanical` on the fix line is the second thing to read. A rule is an error when the repair takes no judgment, and advisory when the repair is a rewrite. This one takes no judgment, so the next step is a command rather than an edit.

### Step 13 — Let the engine make the repair

```
headwater check --fix
```

Trimmed to the first line. The account of what was written goes to standard error, and the cache accounting follows it there:

```
headwater: fixed docs/decisions/0001-store-attempts-in-postgres.md (1 patch)
```

**Check.** `headwater check --strict > /dev/null 2>&1; echo $?` prints `0`, and `grep -A2 '^relations:' docs/decisions/0001-store-attempts-in-postgres.md` prints:

```
relations:
  superseded_by:
    - ACME-DR-0002
```

`--fix` writes only the repairs that need no judgment, and it leaves every finding whose remedy is a rewrite. Read the diff before you commit it.

### Step 14 — Ask the engine what it knows about one document

```
headwater explain ACME-DR-0002
```

```
docs/decisions/0002-deliver-at-least-once.md
  ACME-DR-0002
  kind decision
    shelf `decisions` matched on `docs/decisions/**`
    `decisions` is homogeneous, so placement carries the kind `decision`
  purpose rationale, to explain why a choice was made and what it forecloses
  summary Retrying a delivery is safe, so the queue may send one attempt twice.
  requires the facets status, status_since, summary, last_verified
  requires the sections Context, Decision, Consequences
  may declare supersedes to governed_document, and the other end writes
    superseded_by
  may declare governs to code_path
  may declare constrains to decision
  may declare conflicts_with to decision
  may declare traces_to to governed_document, code_path
  to ACME-DR-0001 supersedes — The queue keeps every delivery attempt in
    Postgres. (the target's own summary) [this document governs the reading]
  from docs/decisions/0001-store-attempts-in-postgres.md superseded_by — The
    queue keeps every delivery attempt in Postgres. (the target's own summary)
    [this document governs the reading]
```

**Check.** The first line of that output is the path of the document whose identifier you named.

Every word this tutorial taught is on that screen at once. There you read the shelf, the kind, the facets, the sections, the relations and both halves of the edge. This is the verb to reach for before you open a file.

### Step 15 — Ask a question in your own words

```
headwater route "why do we keep attempts in postgres"
```

```
route "why do we keep attempts in postgres"
  terms why do we keep attempts in postgres
  distinctive why do we keep attempts in postgres
  purpose behavior 3
  purpose rationale 3
  docs/decisions/0001-store-attempts-in-postgres.md — The queue keeps every
    delivery attempt in Postgres.
```

**Check.** The last line names `docs/decisions/0001-store-attempts-in-postgres.md`.

A **purpose** is the reader intent that a kind exists to serve. The base package declares two, `rationale` and `behavior`, and the route scored your question against both before it looked at any prose. That is why the `summary` facet deserves the most care in any document. Routing serves it as the only pointer a reader gets.

### Step 16 — Ask what you have not wired up

```
headwater conformance
```

Trimmed to the levels block:

```
levels
  L0 Pointed at — reached, 2 of 2 rules met
  L1 Classified — reached, 3 of 3 rules met
  L2 Regenerated — not reached, 3 of 4 rules met
    1 gap, 0 of them waived

L1 reached, against headwater/standard 4.1.0
  a level states what this repository wired up. It measures nothing about the
  corpus, no key declares one, and a waiver moves the exit status and never the
  level.
```

**Check.** The last line of the levels block reads `L1 reached, against headwater/standard 4.1.0`.

Your corpus already stands on the first two rungs, which a package copied by hand never reaches. `pin.current` is what carried you there. It reads the digest step 5 pinned against the record `packages/headwater-standard/release.yml` carries, and step 3 already gave you both. A package you copy from a source directory has no digest to pin, because nobody has published it. That route never reaches this rung: it stays off the ladder for the whole life of a corpus built that way. One gap is left, and it closes with a command:

```
headwater generate
headwater conformance
```

**Check.** `headwater conformance 2>/dev/null | grep 'L2'` prints `  L2 Regenerated — reached, 4 of 4 rules met` and, further down, `L2 reached, against headwater/standard 4.1.0`.

`headwater generate` wrote the two projections that were missing, `docs/decisions/README.md` and `.headwater/corpus.json`, and that closed `projections.current`, the last rule L2 asks for. Every rung this ladder has is reached. A level still measures what you wired up rather than what your documents say. Climbing it here took one pin and two commands, not a better decision record.

## The words this teaches

Say each of these back before you leave. A finding you meet later names two or three of them in one line.

**Corpus.** The tree of documents the engine walks, named by `corpus.root` in `.headwater/taxonomy.yml`. Everything outside it is invisible to every rule.

**Shelf.** A named region of the corpus, in practice a directory pattern. A shelf is the primary classification axis, because placement is the loudest signal a document sends. Metadata never contradicts a shelf.

**Kind.** What a document permanently is. A kind declares the purpose it serves and the facets a document of it must carry. It also declares the sections the document must contain, and the relations it may declare. `decision` is a kind. `draft decision` is not, because a draft is a state and not a species.

**Facet.** A property of one document, declared in the front matter. A facet may carry a role that the engine fills. That is how `status` arrives from the lifecycle regime, and how two dates arrive from the run's clock. A facet that no rule and no projection reads is refused as unread.

**Overlay.** What your repository declares on top of the package it takes. An overlay adds what a package cannot know: your namespace, your phrases, your extra kinds. `headwater taxonomy resolve` merges the package and the overlay into one lock, and every later command reads the lock.

**Obligation.** A claim this system makes about itself, with the rule that verifies it named beside it. Every finding prints the obligation it discharges, in parentheses after the rule name. The register block of every run counts them. An obligation whose author declared that nothing verifies it is counted there as a gap, with an owner.

Four more words earn their place beside those five.

**Package.** A versioned bundle of taxonomy that somebody publishes and you take. **Lock.** The resolved taxonomy, and the only file the checks read. **Census.** The account of every file under the corpus root, which says how many were typed and how many were checked. **Relation.** A typed edge that names its target by identifier, and that may require both ends.

## Where to go next

**Point the engine at your own repository.** The steps above used a corpus made for the purpose. A tutorial cannot state what you should now see about a tree it has never read. Your own tree is the one that answers whether this is worth adopting. Run `headwater init` in it, fetch the package as step 3 did, resolve, and then run this:

```
headwater infer
```

It reports the files that classify as nothing, and the documents that state no summary. Those two lists are the distance between your repository and a corpus. `headwater infer --owner <name> --write` records that distance as declared debt with an expiry, so a strict run passes while the work is outstanding.

**Read `headwater check` next.** [The verb contract](../interfaces/headwater-check.md) states what it reads, what goes to each of its two streams, and the eleven causes behind its one non-zero exit.

**This corpus reached `L2`.** Step 16 climbed the whole ladder, from an untyped file to every rule this base package states about itself. A level names what a repository wired up, never what its documents say. Step 16 said the same, and it is the last thing this page has for you.
