# Headwater authoring conventions

## Work in a worktree

Call `EnterWorktree` at the start of every session in this repository, before reading or editing a file, unless the user has already put the session in a worktree another way. A background session gets this already, from the `worktree.bgIsolation` default in `.claude/settings.json`, which blocks `Edit`/`Write` in the main checkout until `EnterWorktree` runs. No `settings.json` key covers an interactive session the same way, so this instruction is what extends the same isolation to one: a session editing the checkout that Claude Code itself is running from can collide with another session doing the same, and a worktree keeps a change isolated until it is ready to commit.

## The name is Headwater

**Headwater** is the name of the system, not a working name (decided 2026-08-10, [Q10](docs/spec/09-decisions.md#q10--naming)). Capitalize it in prose, including compounds: Headwater's, Headwater-specific, Headwater-native.

Write it as `headwater` in lower case only when it is an identifier: the CLI verb (`headwater check`), package names (`acme/headwater-taxonomy`), the `.headwater/` directory, the `headwater:` annotation prefix, and the `https://w3id.org/headwater/` namespace. Do not capitalize inside code spans or fenced code. Documents under `docs/reviews/` are point-in-time records and stay as written.

## No arbitrary line breaks in Markdown

Do not hard-wrap Markdown source. Write each paragraph, list item, and blockquote paragraph as one logical line; editors and renderers reflow as appropriate. Never insert a line break for line-length reasons, and remove such breaks when you edit a file that has them.

Line breaks are structural only: blank lines between blocks, one line per list item or table row, fenced/indented code kept verbatim. A deliberate hard break inside a paragraph (rare) uses a trailing backslash, not two spaces.

## American spelling

Use American English spelling everywhere: organization, artifact, behavior, customize, judgment, license, catalog. This is the ruling that ASD-STE100 rule 1.14 requires (decided 2026-08-10); do not reintroduce British forms. Documents under `docs/reviews/` are point-in-time records and stay as written.

## Avoid stock AI phrasing

Do not use the stock metaphors and intensifiers that AI-assisted prose overuses: "load-bearing", "first-class (citizen)", "battle-tested", "north star", "delve", "seamless", "holistic", "deep dive", "leverage" as a verb, "robust" as filler. Name the concrete thing the metaphor points at instead: who depends on it, what breaks without it, what it enforces. The list a check reads is `retired_terms` in `.headwater/overlay.yml`, and this paragraph is the statement of why. Add a term in both places, with the reason in the overlay.

Exception: documents under `docs/reviews/` are point-in-time review records and stay as written.

## How these rules are enforced on the shelves that carry this repository's prose

Every rule above is a check that `headwater check` runs, and the taxonomy of this repository is where each one is declared. Nothing enforces them by memory, and no second copy of any rule lives in a script.

**What declares them.** `.headwater/overlay.yml` binds `regimes.language.ste_house` to the kinds that carry this repository's own prose: `design_spec`, `decision`, `decision_register`, `evaluation`, `obligation_register`, `obligation_record`, `interface_contract`, `tutorial`, `requirement` and `acceptance_criterion`. The regime names the controlled language and the profile, fixes the source form as `one_line_per_block`, and lists the stock phrasing under `retired_terms` with the reason each term is retired. The base package declares the voice regime that forbids future intent, change narration and phased rollout. A document under `docs/reviews/` answers to none of the language rules, which is the exemption the sections above state.

**What runs them.** Six rules read prose. `language.controlled.not_met` holds a sentence to 25 words and refuses a contraction, a British spelling and a semicolon in running prose. `language.source_form.not_met` reports a block written over more than one line. `language.retired_term.used` reports a term the regime retired. `voice.forbidden_construction` reports the three voice categories. `section.required.missing` and `link.fragment.unresolved` read the body for other reasons.

**What blocks.** `.githooks/pre-commit` runs `headwater check --strict`, which fails on an error and reports everything else. It first runs `.githooks/change-manifest HEAD`, which turns the committed `HEAD` and your working tree into the manifest that `--change` reads, so a state movement the declared lifecycle refuses is refused here too. That producer runs every `git` command on the path from a commit to a verdict, because the engine runs none. It fails open, so no git and no `mktemp` costs you one rule and never the commit. `sh .githooks/fixtures.sh` drives the gate and the producer over a scratch copy of this corpus, including every refusal, and CI runs it. A rule is an error when its remediation is mechanical and total, which is the [fixability](docs/spec/12-check-layer.md#fixability) bar: a contraction, a British spelling, a hard-wrapped block, and a retired term that names its replacement. A rule is advisory when the remediation is a rewrite: a sentence past the word limit, a semicolon, a stock metaphor, and every voice category. Git does not install repository hooks by itself, and it will not take a merge driver from a repository at all, because a driver is an executable that a clone would otherwise run without being asked. So each clone needs these three once:

    git config core.hooksPath .githooks
    git config merge.headwater-regenerate.name "regenerate a derived artifact"
    git config merge.headwater-regenerate.driver ".githooks/merge-regenerate %O %A %B %P"

The last two are what `.gitattributes` needs to refuse a merge of a derived artifact rather than reconcile one, and the commit gate prints them when a clone has not set them. Read `.gitattributes` for which files that covers and why each is on the list, and read [what a check can know](docs/evaluations/what-a-check-can-know.md) under *The shapes a record takes* for the rule that decides which shape a record should have. One measured limitation belongs beside the command rather than only in the evaluation: **no attribute of any kind reaches two branches that write the same value**, because git compares blobs before it selects a merge strategy. That case is what the recorded fixtures are shaped to avoid rather than to catch.

`headwater check --fix` writes the corrections the engine can derive without judgment, in your working tree, and prints what it wrote on standard error. Today that is a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. It leaves every finding whose remedy is a rewrite, and it refuses a file rather than half-writing one. Read the diff before you commit it.

The hook needs a built engine and fails open with one printed line when there is none. Either profile builds one, and the newer of the two answers: `cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked` is the one to run here, because the shipped `--release` profile pays a single threaded `lto = true` link that costs minutes for a binary this hook runs for a fifth of a second. Build `--release` when what you want is the artifact rather than a checked commit. The CI job builds the engine and runs the same check on every pull request, so an unbuilt clone delays a finding rather than losing it. Run `headwater check` yourself to read the advisory findings, which the hook does not print. [DEVELOPING.md](DEVELOPING.md) carries the measurement and the reason the newer binary wins rather than the shipped one.

**What runs before commit time, in this harness.** `.claude/settings.json` registers three hooks in `.claude/hooks/`, one for each moment [spec 5](docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) names. Git does not install them and Claude Code loads them when the repository opens, which is the opposite of the line above.

| Position | Script | What it does |
|---|---|---|
| `UserPromptSubmit` | `intent.sh` | `headwater route` on your prompt, and nothing at all when the route is silent |
| `PreToolUse` on `Write`/`Edit` | `write.sh` | refuses a raw write of a document that does not exist yet, and names `headwater new`. An edit to an existing document passes |
| `PostToolUse` on `Write`/`Edit` | `write.sh` | names the documents that declare `governs` over the path you just edited. Advisory, and it blocks nothing |
| `Stop` | `review.sh` | runs `.githooks/pre-commit` and stops the turn on what would stop the commit |

Each one calls a verb that already ships, and none carries a rule of its own. The review hook invokes the commit hook rather than repeating it, so this repository still runs exactly one thing at commit time. Every one of them fails open: no built engine, or an input it cannot read, and the action proceeds. `sh` and that engine are the whole of what a session needs, because the harness payload is read by `headwater json` rather than by an interpreter ([HW-DR-0055](docs/decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md)). The review position needs the engine for its re-entry guard, so a host with no engine ends the turn rather than stopping it twice.

None of them binds. A `Bash` call that writes a file matches no matcher, `disableAllHooks` turns all of them off with no record anywhere, and `git commit --no-verify` skips the gate below them. What holds a change is the commit gate and the CI job. `sh .claude/hooks/fixtures.sh` runs all four positions against recorded input, including every refusal.

The same three scripts are registered a second and a third time, in `.codex/hooks.json` and `.github/hooks/*.json`, for the two harnesses [spec 16](docs/spec/16-harness-support.md) records. `sh .claude/hooks/fixtures-live.sh` is what holds that binding rather than the shape it assumes: a real `codex exec` and a real `copilot -p`, over a scratch clone, spending real AI credits against a real login. It skips a harness that is not installed or not authenticated rather than failing on it, nothing gates on it, and no CI job runs it, the same posture as `headwater probe`.

**The escape hatches, and what each one means.** A directive on the offending block marks a deliberate exception, with the reason in the source where a reader will find it:

    …the term the cited authors use. <!-- headwater allow=language.retired_term.used scope=block until=2027-12-31 reason=false_positive note=quoting Star and Griesemer -->

`reason` is `false_positive` where the rule is wrong and `accepted_deviation` where the rule is right and the text stands anyway. `scope` is `file` or `block`. `until` is required, so no exception is permanent, and the report counts an expired directive and names it.

The second hatch is for debt rather than for an exception. The `adoption` block of `.headwater/taxonomy.lock` holds `(document, rule)` pairs under a task with an owner and an expiry. A pending finding is reported with its task beside it and does not fail a strict run. `headwater infer --owner <name> --write` writes one, and `headwater check` reports how many pairs remain on every run. That write adds to the block rather than producing it: it proposes only the findings no open task holds, it mints an identifier past every identifier the block declares, and it prints what it carried through. A second run of it declares nothing and writes nothing. This repository declares one task, `AD-1`, and its one pair is closed. The pair carried the last entry of the retired linter's baseline, and that sentence was rewritten rather than waited out, so `headwater check` now reports the task as holding 0 findings.

**The on-ramp, and the suite that holds it.** `docs/tutorials/your-first-governed-corpus.md` takes a reader from an empty directory to a passing strict run, and it states after every step what the reader should now see. Each of those is a claim about this engine copied into prose, so `sh .claude/tutorial/fixtures.sh` is the verb that produces them: it reads the commands out of the document, runs them against a scratch repository under the temporary directory, and diffs each result against the block the document prints. It blocks in CI, and it writes nothing inside this checkout. Edit an output block only by running the command and taking what it printed.

**Writing governed prose.** Invoke the `ste-editor` skill before you rewrite a document under `docs/spec/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/` or `docs/tutorials/`, and run `headwater check` on the result. The skill carries the rules that no check reads: consistent terms, multi-word nouns kept to three words, and the judgment calls that a lexical rule cannot make.

**The seven skills, and when each one loads.** Nothing makes a skill load. A harness reads a description and a model picks, which [spec 5](docs/spec/05-ai-integration.md#how-a-skill-reaches-an-agent-and-what-nothing-does) states as a measurement rather than a property. This paragraph is the one mechanism stronger than a description, and it costs context on every session.

| skill | invoke it before |
|---|---|
| `headwater-orient` | you search `docs/` or open a specification part to find out what this repository already says |
| `headwater-authoring` | you add or revise any document under `docs/`, and whenever `headwater new` refuses |
| `headwater-taxonomy` | you touch `packages/`, `docs/taxonomies/` or `.headwater/overlay.yml` |
| `ste-editor` | you rewrite prose under `docs/spec/`, `docs/decisions/`, `docs/evaluations/`, `docs/obligations/`, `docs/interfaces/`, `docs/requirements/`, `docs/acceptance-criteria/` or `docs/tutorials/` |
| `headwater-engine` | the first cargo or CLI command of a session, and whenever a build fails on the toolchain |
| `repo-cleanup` | you retire a worktree, a local branch or a remote branch that a merged change left behind |
| `headwater-sweep` | you are asked to read a slice of the corpus for what no check can see |

The first two are ordered: orientation finds the document, and authoring writes one. A session that opens a specification part to discover it was the wrong part has paid for a read it did not need, and `headwater explain` answers from the graph the engine already built.

The last one is the only mechanism here that no engine performs. `headwater sweep plan` writes the briefing, you read the documents, and `headwater sweep report` says what the engine could confirm about what you wrote back. Nothing gates on it: the verb exits 0 whatever it finds, no gate and no CI job runs it, and no crate of this engine opens a socket.

`.claude/agents/headwater-maintainer.md` is the maintainer subagent: hand it a change and it reports what the change touched, what is now stale, what the corpus is owed, and what it could not decide. It proposes and never accepts.

`.claude/agents/headwater-product-owner.md` is the product-owner subagent, and it reads the whole board rather than a change: whether the milestones are being completed in order, which of them is finished and left open, which issues are filed against the wrong one, what an adopter still cannot do, and what needs a ruling. `/product-owner` runs it standalone at any time, and the build order also runs it at the top of a run and every fifth iteration inside one, never in the context of the agent doing the building.

It owns the structure of the board and never its scope. It closes a finished milestone, creates one, moves an issue between milestones, writes the `adopter-blocking` and `self-audit` labels, and files an issue only where it can quote the bar of an existing milestone that names work no open issue carries. It closes no issue and edits no Done-when, because what work is and when it is done is the owner's. That boundary is the reason it may hold a pen at all: closing the current milestones cleanly needs one, and deciding what the product should do next does not.

**The value rule that both of them serve.** Work must name a reader who is not this repository. `.claude/commands/next-run.md` states it once, under *The value rule*, and every other file cites it rather than restating it. Two labels carry it on the issue tracker: `self-audit` for work found by running the engine over this corpus with no reader outside it, and `adopter-blocking` for work an outside adopter cannot proceed without. `self-audit` means the work waits, not that it is wrong.

## Filing an issue

`.github/ISSUE_TEMPLATE/issue.md` is the shape, and it is the only copy of it. GitHub offers it in the web form and nowhere else, so anything filing through `gh` reads that file first and follows it by hand.

**An issue opens with an `## ELI5` section.** Two to five sentences of plain language, for a reader who has never opened this repository: what is wrong, what a person would notice because of it, and what the fix looks like from outside. No spec numbers, no rule identifiers, no verb or crate name that the same sentence does not define. The rest of the body then addresses a reader who does know the repository, which is what the bodies here have always done. The ELI5 is the part that was missing, and it is written last and read first.

Nothing checks this. No hook, no rule and no CI job reads an issue body, so the template and this paragraph are the whole mechanism.

`sh .claude/skills/fixtures.sh` holds every claim these files make about the engine. It is a blocking CI step, half of its cases are derived from the skill files rather than listed, and it writes only into a scratch copy of the corpus.

## The identifier claim store

`headwater new` mints under a `reconcile-first` scheme by reading the tree, and a tree holds no concurrency: every branch cut from one `main` reads the same corpus, so two branches mint the same number, each one passes `check --strict` alone, the file names differ, and the merge is silent. `.headwater/ids/<scheme>/<identifier>` is the answer, and [HW-DR-0054](docs/decisions/0054-the-upper-bound-of-a-reconcile-first-allocator-is-the-corpus-and-a-claim-store.md) is the ruling. Each file holds the path of the document that minted it, it is written once and never modified, and two branches that claim one value hit an `add/add` conflict on that one path.

Three things about it bind a change to this repository. **A claim file is never empty**, because two identical empty blobs merge clean and that is the whole mechanism gone. **`.gitattributes` sets no `merge` attribute on `.headwater/ids/**`**, because `merge=union` there concatenates two claimants and loses the only record of who minted what. **Both writers create and never overwrite**, so a claim you met is a claim you leave. `sh tools/id-store-fixtures.sh` holds all three and runs in CI. `headwater check --fix` writes a claim the store is owed, and it is what bootstraps the store of a corpus that minted before it had one.

## What holds the first screen, and what still does not

`docs/` is the corpus root, so no rule of this engine reads the root `README.md`. `docs_dir: docs`, so `mkdocs build --strict` never reads it either, and every `README` named in a workflow, a hook or a script under `tools/` is `engine/README.md` or `docs/interfaces/README.md`. A British spelling, a contraction, a hard-wrapped block, four retired terms, a dead link and a dead fragment planted in that file produce exit 0 and a byte-identical check report.

`sh tools/readme-fixtures.sh` closes two of those eight, and it is worth being exact about which. It runs in CI and enumerates each population from the page rather than listing it. It holds the referential integrity of the page — every relative link resolving to a file that exists, and every fragment resolving to a heading that exists — and it holds four claims the page makes about itself: the opening image exists at 1280 by 640 under 1 MB, every GitHub URL names the owner and repository `git remote get-url origin` names, the exclusive or below, and no count of milestones. It reads links per occurrence and not per line, because this repository forbids hard-wrapped Markdown and a line-shaped count of a page whose paragraphs are single long lines is off by a factor of three.

The case it exists for is the paste block. Either the tag the fence pins resolves against the remote, or the paragraph directly above the fence says the tag is not cut and tells the reader to omit the line — never both, and never neither. So the change that cuts that tag cannot merge until it deletes that sentence, and a change that deletes the sentence early cannot merge until the tag exists. The tag is read out of the fence rather than written into the suite, and the four states are driven over scratch pages and stub bare repositories.

**The prose on that page still answers to nobody, and this suite does not change that.** A British spelling, a contraction, a hard-wrapped paragraph, a retired term and `headwater` written lower case in running prose were planted on the page together, and the suite reported 37 of 37 passing. The language regime binds the kinds declared under the corpus root, and the README is not a governed document, so this suite deliberately reimplements none of those rules: a second copy of a rule living in a script is what every other paragraph here refuses. Whether the regime should reach a file outside `docs/` is a question about the taxonomy and not a gap in this suite.

Two further things on that page answer to nobody. An absolute URL is never fetched, so a dead external link stays green. The Status blockquote names the open milestones, and the suite refuses only a **count** of them, because a count is what went stale once and the authority for one is the GitHub API — which a gate here does not open a socket to, the same posture `headwater probe` takes.

**A false positive here costs more than a missed defect.** A required check on the most-read page in the project that reddens on correct Markdown is a check the first person it annoys turns off, after which it guards nothing. Two shapes reached the first cut of this suite: a CommonMark link title, `[a](p "t")`, parsed into the path and reported as a missing file, and a link printed as an example inside an inline code span, read as a link. Both are cases now, and removing either fix turns two cases red.

## What holds the command an outsider runs

`engine/README.md` carries two `docker run` recipes, and between 2026-08-26 and 2026-09-07 both exited 101 before compiling a line. `ordered-float` had entered `engine/Cargo.lock` under `saphyr` declaring a floor of 1.90, both recipes pinned a 1.85 image, and no crate of this workspace declared `rust-version`, so cargo never read the floor either. This repository went public with both commands dead, and a change on that day edited both of them without running either.

`[workspace.package]` in `engine/Cargo.toml` now declares `rust-version`, every crate declares `rust-version.workspace = true`, and `engine/clippy.toml` states the same number as its `msrv`. `sh tools/engine-readme-fixtures.sh` is what holds them: its first case compares the highest `rust-version` across `cargo metadata --locked` against the tag the recipes pin, so a `cargo update` that raises the floor reddens the same day. It runs no container and builds nothing, which is why it can be a required step. It also holds the four things those recipes explain and nothing else read — `--user` on the test half and root on the other, the `git rev-parse --show-toplevel` mount, the two blocks that a blank line alone would render as one `<pre>`, and the absence of `-D warnings` — and it holds the nine files that state the floor to one another, enumerated from the tree by shape rather than listed. It follows the shape of `tools/readme-fixtures.sh` and reads no cargo flag, because `tools/build-declaration-fixtures.sh` owns those.

It does not run the container, and nothing does. A recipe that is well formed, correctly pinned and still broken stays green here, so run both commands by hand after changing either.

## The two declarations the build makes about itself

`engine/Cargo.lock` is committed, and `--locked` is what holds a cargo run to it. Every cargo step of `.github/workflows/ci.yml` that resolves a manifest, both container commands of `engine/README.md`, and every copy of `cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked` carry the flag. A maintainer's own loop does not, and that is deliberate: there a rewritten lock is the intended result of adding a dependency, and the flag would refuse it. `cargo fmt` is exempt because it rejects the flag, and `cargo --version` because it accepts and ignores it.

`[workspace.package]` in `engine/Cargo.toml` declares `license = "Apache-2.0"` and every crate manifest declares `license.workspace = true`. The workspace key alone declares nothing a reader sees: it is an inheritance source, and a member that omits its own line reports `license: null` to `cargo metadata`. Delete the workspace key and no cargo command runs at all, so that direction needs no guard. A crate added later that forgets the member line is the direction that stays silent.

`sh tools/build-declaration-fixtures.sh` holds both, enumerates each population from the tree rather than listing it, and runs in CI. It reads the workflow by parsing every `run:` value, so a cargo step written behind a `cd`, an environment assignment, a nested shell or a block scalar is in the population too, and it judges the install command one occurrence at a time rather than one line at a time, because a line filter goes quiet on a paragraph carrying both forms. So a crate you add declares the license, and a cargo invocation you add to CI or a copy of the install command you write carries `--locked`. No count of either population is written down here, because a number no judge reads drifts.
