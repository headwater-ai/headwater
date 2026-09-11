# Developing Headwater

This page is the contributor loop: what to install, what to type, what CI will run against what you push, and the two failures that cost a newcomer an afternoon. [`CONTRIBUTING.md`](.github/CONTRIBUTING.md) carries the terms you agree to and the sign-off every commit needs. [`CLAUDE.md`](CLAUDE.md) carries the authoring conventions that the prose under `docs/` answers to, and why each one is a check rather than a habit. This page carries neither of those, and it does not explain the design of the engine — [`engine/README.md`](engine/README.md) does that, crate by crate.

Read this before your first build. The gate list near the end is the part that goes stale fastest, so it is derived from the workflow by a suite rather than kept by hand.

## Before your first commit

Git does not install a repository's own hooks, so a fresh clone runs none of them. Enable them once:

    git config core.hooksPath .githooks

Keep that path relative. An absolute path makes every worktree of this repository run the main checkout's hook body instead of the one on its own branch, which is a defect that reads as the gate passing.

`.githooks/pre-commit` builds a change manifest from your working tree and then runs the engine over the corpus in strict mode. It refuses an error and reports everything else, so a commit that goes through is not a commit with nothing to answer for. Run the engine yourself to read the advisory findings.

`.githooks/pre-commit` needs a built engine and **fails open with one printed line when there is none**. A fresh worktree has no `engine/target`, so the hook there exits 0 without checking anything. Do not read that silence as a pass.

Either profile builds an engine the gate accepts, and so does every hook under `.claude/hooks/`. `release` is what CI builds and what a release artifact ships. `--profile dev-release` is the same optimization level without the `lto = true` and `codegen-units = 1` link, and on an eight core host a one crate relink under it costs 25 seconds of CPU against 153 for the shipped profile. Nothing here runs the engine for longer than a fifth of a second, so the cheaper binary is the one to build when what you want is a checked commit rather than an artifact:

    cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked

When both are built the **newer** one answers, rather than the shipped profile by name. A stale `release` binary beside a `dev-release` one built after it would check your change through an engine that predates it, which is a failure that reads as a pass. `sh .claude/hooks/fixtures.sh` and `sh .githooks/fixtures.sh` hold that rule on both sides.

## The toolchain floor

**Rust 1.90 or later.** `[workspace.package]` in `engine/Cargo.toml` declares it and every crate inherits it, so cargo refuses an older toolchain and names the crate that raised the floor. Below 1.85 the message is worse: a dependency is on edition 2024, and a cargo older than that reports `feature edition2024 is required`, names no crate, and reads like a corrupt tree. Check your toolchain first when a clean checkout will not build.

A machine that installed Rust from its distribution packages usually has neither `rustfmt` nor `clippy`. [`engine/README.md`](engine/README.md) carries two `docker run` recipes that supply both and pin the floor at the same time. They are two commands with two exit statuses, and joining them with a pipe reports only the second one.

The engine is a cargo workspace under `engine/`, and the corpus it reads is the repository above it. Two directories, so every command below says which one it means. `cargo test` from the repository root finds no manifest at all.

## The loop

From `engine/`, in debug:

    cargo test

That is the whole workspace, which is the bar CI holds you to. Shorten a loop by naming one crate, or one test file inside it:

    cargo test -p headwater-check
    cargo test -p headwater-probe --test fixtures

Run the workspace again before you propose the change.

`cargo fmt` must be run **from `engine/`**. `cargo fmt --manifest-path engine/Cargo.toml` is not a quieter way to say the same thing: `engine/` is a virtual workspace root, so that form exits 1 with `Failed to find targets` and formats nothing. It looks silent only to a caller that ignores the exit status, which is how it went unnoticed here for a while. Read `$?`.

A recorded fixture is re-recorded rather than edited by hand:

    HEADWATER_BLESS=1 cargo test -p headwater-check

The digest of the resolved taxonomy lock reaches several recorded fixtures and the committed corpus descriptor, so a taxonomy change moves files your diff never named. Read that diff instead of blessing past it.

`cargo test` stops at the first failing target, so a poisoned run reports a fraction of the suite and the smaller number reads like a collapse rather than like an early stop.

## The release build, and what it is not for

    cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked

This builds the binary a person runs by hand. It is **not** a verification step. `lto = true` and `codegen-units = 1` cost minutes on the link, and a debug `cargo check` and `cargo test` prove the same fix in seconds. Reach for `--release` when the session needs the binary itself: to hand it to somebody, to run a verb against a real corpus, or to measure a performance claim, which debug and release answer differently by roughly an order of magnitude. `--profile dev-release` links faster at a smaller optimization cost.

That link is **single threaded**, which is what makes it expensive rather than merely slow. Measured on an eight core host: a one crate relink costs 153 seconds of CPU under `release` and 25 under `dev-release`, and the `release` figure is one core held for two and a half minutes rather than eight cores held for twenty seconds. On a machine running anything else at the time, that is the difference the other work feels. `lto = "thin"` is not the answer either, and it was measured before it was ruled out: 287 seconds of CPU against 153, spread over more cores, so it halves the wall clock and nearly doubles the load.

**A hook or the commit gate is not a reason to build this profile.** Both accept a `dev-release` binary, which is what the first section of this page says to build.

Every verb of that binary takes `--root`, which names the corpus to read. Without it a verb reads the current directory, and the current directory after a build is the engine workspace rather than the corpus.

`--locked` on that command is not decoration. `engine/Cargo.lock` is committed, and the flag is what holds a cargo run to it. Every cargo step in CI **that resolves a manifest** carries it, and so does every copy of the install command in this repository. Two are exempt and neither is an omission: `cargo fmt` rejects the flag, and `cargo --version` accepts and ignores it. Neither reads a manifest, so neither can rewrite a lock. A maintainer adding a dependency drops the flag deliberately, because there a rewritten lock is the intended result. `sh tools/engine/build-declaration-fixtures.sh` is what holds all of that, one occurrence at a time.

## A faster build, and the one failure it causes

`tools/engine/dev-fast-build-setup.sh` wires `mold` and `sccache` into your own `~/.cargo/config.toml`. It is optional, it is idempotent, it installs nothing itself, and `--remove` undoes exactly the block it wrote. Nothing in CI or in a fresh clone depends on either binary.

Two things to know before you run it.

**sccache cannot cache an incremental build.** `cargo check` and `cargo test` use incremental compilation by default, so sccache does close to nothing for them; cargo's own incremental cache already covers that case. It earns its place on `--release`, on `--profile dev-release`, and on a clean or cross-branch rebuild. `mold` helps every link.

**sccache shares one object cache across every checkout on the machine, and some test binaries bake their own path in.** A test that resolves a fixture directory from `env!("CARGO_MANIFEST_DIR")` compiles that absolute path into the object. Compile it inside a worktree, delete the worktree, and a later run in the main checkout can get a cache hit on the stale object and fail on a path that no longer exists. **A cargo test failure that quotes a path under `.claude/worktrees/` is a cache hit, not a defect in your change.** The remedy is one command:

    cargo clean -p <crate>

Then run the test again. Because `cargo test` stops at the first failing target, one poisoned crate can hide the rest of the suite behind it.

**`-p <crate>` is the whole command, and the bare `cargo clean` is not a stronger version of it.** The bare form removes the build output of every crate in the workspace and every dependency under it, so the next `cargo test` recompiles from nothing and the next `--release` build pays the link again. One session here ran it three times in an afternoon, once immediately before a `cargo test`, and bought a full rebuild each time to clear one crate's stale object. Name the crate. Reach for the bare form when the question is whether the build directory itself is corrupt, which is rare enough that it has not happened here yet.

**A host that runs several sessions at once wants `tools/hw-cargo` too.** It is optional and machine-local in the same way, and nothing in this repository or in CI calls it. It takes one of three slots, points cargo at that slot's own pooled target directory, and then runs the cargo command you handed it:

    sh tools/hw-cargo build --profile dev-release -p headwater-cli --manifest-path engine/Cargo.toml --locked
    sh tools/hw-cargo test --workspace

Two measurements set both halves, taken here on 2026-09-11 while a build-order run held fifteen worktrees. Nine cargo invocations were running at once and the run queue sat at 34 against 8 cores, because the throttle in use then wrapped `cargo build` and nothing else. Separately, 1441 dependency rlibs sat on disk under 621 distinct names. So 57 percent of that output repeated a compilation already finished in another worktree, including 34 separate copies of `syn`. Pooling the target directory is what shares that work. sccache cannot, because it declines a proc-macro and declines an incremental build, and those two reasons covered 1621 of 2961 compile requests.

After a build it copies the engine back to `engine/target/<profile>/headwater` inside your checkout, which is where the commit gate, every hook under `.claude/hooks/` and several scripts under `tools/` look for it. It writes only the profile you built, because the gate takes the newer of `release` and `dev-release`. `--status` prints the pool and the size of each slot, and `--prune` removes all of it.

## What CI runs

`.github/workflows/ci.yml` defines two jobs, `engine` and `headwater`, and a pull request triggers both. Everything below is blocking. This list is checked against the workflow by `sh tools/repo/developing-fixtures.sh`, in both directions, so a gate added to CI and not written here fails, and a gate written here that CI does not run fails too.

**What that check holds is the name of each gate, and not the flags printed beside it.** The commands below are written as CI writes them so that you can copy a line and run it, but no case compares a flag: `--check` deleted from a line here moves nothing, and `--check` deleted from the workflow moves nothing either. The reason is that the flags on these commands are a pinned clock, a change manifest, an output shape and a check-versus-write switch, and a suite that compared them would redden on a reordering that changed no gate. `--locked` is the one flag anything in this repository holds, and `sh tools/engine/build-declaration-fixtures.sh` is what holds it.

The cargo commands. The `engine` job runs the first four; the `headwater` job runs the build, because it needs the binary before it can run a verb over the corpus:

    cargo --version
    cargo fmt --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --locked
    cargo build --release -p headwater-cli --locked

A green clippy on your host is not evidence about CI. Three toolchains disagree about this source — the pinned container, your host, and CI's current stable — and a host clippy has reported zero warnings on a tree that CI rejected with two errors. Push and read CI rather than reproducing it locally.

The engine's own verbs, run over this corpus in the `headwater` job:

    headwater taxonomy resolve --check
    headwater generate --check
    headwater export --check
    headwater conformance --level L0
    headwater check

The first three run in `--check` form, which refuses a committed artifact that the sources no longer produce. `check` runs several times rather than once: with the cache and without it, scoped to the change the pull request carries, and once per output format, because the cache must not be able to change a verdict and no format may disagree with another. CI also pins the clock with `--now`, so a rule that reads a date gives the same answer on a rerun.

The fixture suites, which are shell and Python rather than cargo, and which you can run yourself from the repository root:

    sh .githooks/fixtures.sh
    sh .claude/hooks/fixtures.sh
    sh .claude/skills/fixtures.sh
    sh .claude/agents/fixtures.sh
    sh .claude/tutorial/fixtures.sh
    sh tools/taxonomy/n8n-fixtures.sh
    sh tools/site/assemble-site.sh --check
    sh tools/engine/build-declaration-fixtures.sh
    sh tools/engine/color-fixtures.sh
    sh tools/repo/developing-fixtures.sh
    sh tools/engine/engine-readme-fixtures.sh
    sh tools/repo/id-store-fixtures.sh
    sh tools/repo/library-index-fixtures.sh
    sh tools/probe/probe-record-fixtures.sh
    sh tools/repo/readme-fixtures.sh
    sh tools/repo/retire-worktree-fixtures.sh
    sh tools/site/refresh-crawler-files.sh --check
    sh tools/site/refresh-figures.sh --check
    sh tools/site/refresh-site-tokens.sh --check
    sh tools/run/run-census-fixtures.sh
    sh tools/run/run-dir-fixtures.sh
    sh tools/site/check-site-footer.sh .headwater/site-deploy
    sh tools/site/site-canonical-fixtures.sh
    sh tools/site/site-console-fixtures.sh
    sh tools/site/site-footer-fixtures.sh
    sh tools/site/site-fragments-fixtures.sh
    python3 tools/site/check-site-canonical.py .headwater/site-build
    python3 tools/site/check-site-console.py
    python3 tools/site/check-site-fragments.py
    python3 tools/site/render-tutorial.py --check

Each one holds an artifact that no rule of the engine reads: a workflow, a manifest, a page outside the corpus root, a hook, a skill file. They are cheap, they need no container, and running the ones your change touches before you push saves a round trip.

Two things are deliberately absent from that list. `.githooks/change-manifest` is a producer the workflow calls rather than a gate that can fail on its own. `.claude/hooks/fixtures-live.sh` spends real AI credits against a real login, so no job runs it and nothing gates on it.

One of them is the only thing in this repository that runs the engine under a terminal. `tools/engine/color-fixtures.sh` attaches a pseudo-terminal with util-linux `script` and asserts that a verb whose interface contract promises to sense its stream emits an escape sequence there, and none through a pipe, under `NO_COLOR` or under `--no-color`. Everything else in this workflow reaches the engine through a pipe, where [HW-DR-0045](docs/decisions/0045-coloring-the-cli-and-where-the-banner-goes.md)'s refusal of a `--color=always` flag makes the painter the identity, so a renderer wired to plain output forever is green in every other gate. On a host with no `script` it prints one line saying every case is unrun and exits 0, so read its output and not only its status.

## What holds this repository

[`CLAUDE.md`](CLAUDE.md) states the rules every agent obeys, and this section is where each of them is enforced, what holds the pages outside the corpus root, and what the build declares about itself. It moved here from `CLAUDE.md` because that file loads into every dispatched agent and this narration is for a developer who wants the mechanism ([HW-PD-0001](docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md)).

### How the language rules are enforced

Every rule in `CLAUDE.md` is a check that `headwater check` runs, and the taxonomy of this repository is where each one is declared. Nothing enforces them by memory, and no second copy of any rule lives in a script.

**What declares them.** `.headwater/overlay.yml` binds `regimes.language.ste_house` to the kinds that carry this repository's own prose: `design_spec`, `decision`, `decision_register`, `evaluation`, `obligation_register`, `obligation_record`, `interface_contract`, `tutorial`, `requirement`, `acceptance_criterion` and `process_decision`. The regime names the controlled language and the profile, fixes the source form as `one_line_per_block`, and lists the stock phrasing under `retired_terms` with the reason each term is retired. The base package declares the voice regime that forbids future intent, change narration and phased rollout. A document under `docs/reviews/` answers to none of the language rules, which is the exemption `CLAUDE.md` states.

**What runs them.** Six rules read prose. `language.controlled.not_met` holds a sentence to 25 words and refuses a contraction, a British spelling and a semicolon in running prose. `language.source_form.not_met` reports a block written over more than one line. `language.retired_term.used` reports a term the regime retired. `voice.forbidden_construction` reports the three voice categories. `section.required.missing` and `link.fragment.unresolved` read the body for other reasons.

**What blocks.** `.githooks/pre-commit` runs `headwater check --strict`, which fails on an error and reports everything else. It first runs `.githooks/change-manifest HEAD`, which turns the committed `HEAD` and your working tree into the manifest that `--change` reads, so a state movement the declared lifecycle refuses is refused here too. That producer runs every `git` command on the path from a commit to a verdict, because the engine runs none. It fails open, so no git and no `mktemp` costs you one rule and never the commit. `sh .githooks/fixtures.sh` drives the gate and the producer over a scratch copy of this corpus, including every refusal, and CI runs it. A rule is an error when its remediation is mechanical and total, which is the [fixability](docs/spec/12-check-layer.md#fixability) bar. A rule is advisory when the remediation is a rewrite. Git does not install repository hooks by itself, and it will not take a merge driver from a repository at all, because a driver is an executable that a clone would otherwise run without being asked, which is why each clone runs the three `git config` commands `CLAUDE.md` lists once.

**The second thing that blocks is `.githooks/commit-msg`, and it blocks a merge rather than a commit.** It does nothing at all unless `MERGE_HEAD` is present, and then it runs `.githooks/merged-fold-check`, which runs `headwater generate --check`, `headwater taxonomy resolve --check` and `sh tools/site/refresh-figures.sh --check` over the tree the merge produced. Those are the three CI runs against `refs/pull/N/merge`, and until it existed the local path ran none of them. It needs the same one `git config core.hooksPath .githooks` and no line of its own. It fails open with a printed line on a clone with no built engine, exactly as `.githooks/pre-commit` does, so a merge that goes through silently on a fresh worktree is a merge nothing checked. `git merge --no-verify`, `git merge --squash`, a fast-forward and a rebase reach it not at all, and the hook file enumerates that. `engine/crates/census/tests/merge_driver.rs` drives the real hook over a scratch repository with planted producers, and every one of its gate cases fails against a body that refuses nothing.

The two merge-driver lines are what `.gitattributes` needs to refuse a merge of a derived artifact rather than reconcile one, and the commit gate prints them when a clone has not set them. Read `.gitattributes` for which files that covers and why each is on the list, and read [what a check can know](docs/evaluations/what-a-check-can-know.md) under *The shapes a record takes* for the rule that decides which shape a record should have. One measured limitation belongs beside the command rather than only in the evaluation: **no attribute of any kind reaches two branches that write the same value**, because git compares blobs before it selects a merge strategy. That is the case the hook above exists for, and `.gitattributes` says which of the declared paths two branches can actually write alike, measured over all of them rather than assumed.

`headwater check --fix` writes a British spelling, a contraction whose expansion is one word, a retired term that names a replacement, and a missing reciprocal link. The CI job builds the engine and runs the same check on every pull request, so an unbuilt clone delays a finding rather than losing it. *A faster build* above carries the measurement and the reason the newer binary wins rather than the shipped one.

### What runs before commit time, in this harness

`.claude/settings.json` registers three hooks in `.claude/hooks/`, one for each moment [spec 5](docs/spec/05-ai-integration.md#the-hook-contract-and-what-a-hook-cannot-bind) names. Git does not install them and Claude Code loads them when the repository opens, which is the opposite of the line above.

| Position | Script | What it does |
|---|---|---|
| `UserPromptSubmit` | `intent.sh` | `headwater route` on your prompt, and nothing at all when the route is silent |
| `PreToolUse` on `Write`/`Edit` | `write.sh` | refuses a raw write of a document that does not exist yet, and names `headwater new`. An edit to an existing document passes |
| `PostToolUse` on `Write`/`Edit` | `write.sh` | names the documents that declare `governs` over the path you just edited. Advisory, and it blocks nothing |
| `Stop` | `review.sh` | runs `.githooks/pre-commit` and stops the turn on what would stop the commit |

Each one calls a verb that already ships, and none carries a rule of its own. The review hook invokes the commit hook rather than repeating it, so this repository still runs exactly one thing at commit time. Every one of them fails open: no built engine, or an input it cannot read, and the action proceeds. `sh` and that engine are the whole of what a session needs, because the harness payload is read by `headwater json` rather than by an interpreter ([HW-DR-0055](docs/decisions/0055-a-hook-reads-a-wire-format-through-the-engine-and-not-through-an-interpreter.md)). The review position needs the engine for its re-entry guard, so a host with no engine ends the turn rather than stopping it twice.

None of them binds. A `Bash` call that writes a file matches no matcher, `disableAllHooks` turns all of them off with no record anywhere, and `git commit --no-verify` skips the gate below them. What holds a change is the commit gate and the CI job. `sh .claude/hooks/fixtures.sh` runs all four positions against recorded input, including every refusal.

The same three scripts are registered a second and a third time, in `.codex/hooks.json` and `.github/hooks/*.json`, for the two harnesses [spec 16](docs/spec/16-harness-support.md) records. `sh .claude/hooks/fixtures-live.sh` is what holds that binding rather than the shape it assumes: a real `codex exec` and a real `copilot -p`, over a scratch clone, spending real AI credits against a real login. It skips a harness that is not installed or not authenticated rather than failing on it, nothing gates on it, and no CI job runs it, the same posture as `headwater probe`.

### The adoption block, and what a strict run does with it

The `adoption` block of `.headwater/taxonomy.lock` holds `(document, rule)` pairs under a task with an owner and an expiry. A pending finding is reported with its task beside it and does not fail a strict run. `headwater infer --owner <name> --write` writes one, and `headwater check` reports how many pairs remain on every run. That write adds to the block rather than producing it: it proposes only the findings no open task holds, it mints an identifier past every identifier the block declares, and it prints what it carried through. A second run of it declares nothing and writes nothing. This repository declares one task, `AD-1`, and its one pair is closed. The pair carried the last entry of the retired linter's baseline, and that sentence was rewritten rather than waited out, so `headwater check` now reports the task as holding 0 findings.

### The on-ramp, and the suite that holds it

`docs/tutorials/your-first-governed-corpus.md` takes a reader from an empty directory to a passing strict run, and it states after every step what the reader should now see. Each of those is a claim about this engine copied into prose, so `sh .claude/tutorial/fixtures.sh` is the verb that produces them: it reads the commands out of the document, runs them against a scratch repository under the temporary directory, and diffs each result against the block the document prints. It blocks in CI, and it writes nothing inside this checkout. Edit an output block only by running the command and taking what it printed.

`sh tools/taxonomy/n8n-fixtures.sh` does the same for the three n8n fixture corpora under `docs/taxonomies/*/fixtures/n8n/`. Each README prints the commands that assemble its corpus and then states what the run reports, and the suite reads both out of the page: it runs the recipe verbatim and diffs every figure against the paragraph that states it. It carries no second copy of a number. It blocks in CI, it writes nothing inside this checkout, and it fails unless it finds three corpora, unless an edited figure would have been caught, and unless a version pin the vendored package does not carry would have failed. Those pages went stale unseen for four minor versions of `headwater/standard`, because `docs/taxonomies/**` is outside this repository's own corpus and no check read either scalar.

`sh .claude/agents/fixtures.sh` holds the build-order agents under `.claude/agents/hw-*.md`, the two commands that dispatch them and the doctrine both carry: every `subagent_type` a command names is a definition, every skill an agent invokes exists, every ruling cited under `.claude/` is on the graph and not superseded, the two agents that must not write carry neither `Edit` nor `Write`, each check of the verification bar is stated once, `CLAUDE.md`, `.claude/commands/next-run.md` and `.claude/run/doctrine.md` hold the byte ceilings the suite declares, and the doctrine block in the command is byte-identical to the file. [HW-PD-0001](docs/process/decisions/0001-orchestration-prose-has-one-owner-per-sentence.md) is why each of those files holds what it holds.

`sh tools/run/run-dir-fixtures.sh` holds `tools/run/run-dir.sh`, the ledger of a build-order run: a directory under the git common dir with the doctrine copied in, `log.jsonl` at one line per iteration, and every total derived by `run-dir.sh net` and never stored ([HW-PD-0005](docs/process/decisions/0005-the-ledger-is-split-its-tabular-parts-are-jsonl-and-its-totals-are-derived.md)). The same tool holds the run's claims: a claim on an artifact is a file under the run directory opened with `set -C`, which the shell implements as O_EXCL, never empty, so two parents claiming one artifact at once cannot both succeed and the second reads `WAITS-ON`. Not `mkdir`: on a host whose coreutils are the uutils rewrite, two racing `mkdir` calls on one path both succeeded in 17 of 20 races, and the suite's race case is what found it ([HW-PD-0004](docs/process/decisions/0004-coordination-is-a-create-only-claim-and-authority-stays-on-the-tree.md)). The suite provokes the three refusals, a reused id, a line missing a key and a line carrying a total, races two claimants for one artifact ten times and holds that exactly one owns each time, and it needs `jq`, which CI treats as a skip when absent.

`sh tools/repo/retire-worktree-fixtures.sh` holds `tools/repo/retire-worktree.sh`, the sweep that retires a worktree and a branch a merge finished. The tool exists because ownership of a tree and knowledge of its merge sit in different agents: `hw-build` makes the tree and exits when the pull request opens, at which point the branch is unmerged and no rule permits deleting it, and `hw-integrate` is the first stage that knows it merged. Merged is decided by the pull request rather than by ancestry, because a squash merge leaves no commit of the branch reachable from `origin/main` and ancestry therefore calls every finished branch unmerged forever. The suite builds its own repository, produces that exact shape, and asserts that ancestry gets it wrong before asserting that the tool gets it right. Four guards are each provoked: a locked tree, a tree holding an uncommitted change, a tree whose pull request is open, and a tree a live process has its working directory in, which is the guard with a cost behind it, because removing such a tree pins the directory on disk and the next cargo run there exits 101 naming the binary under test. The report arm is held to retiring nothing, since the action is destructive and a forgotten flag must not read as a clean sweep, and the suite runs the tool twice to hold it idempotent. It stands in for `gh` through `HEADWATER_RETIRE_PR_STATE` and needs nothing but git.

`sh tools/probe/probe-record-fixtures.sh` holds the two halves of a recorder, `tools/probe/probe-transform.sh` and `tools/probe/probe-record.sh`. [Spec 15](docs/spec/15-the-recorder-contract.md) keeps both outside this engine, and [HW-DR-0059](docs/decisions/0059-a-transform-over-a-harness-session-log-is-an-observed-transcript-when-the-log-arrives-by-a-channel-the-model-cannot-write-to.md) admits the shape on one condition: the session log has to reach the transform through a channel the model holds no handle on, which is the standard output of `claude -p --output-format stream-json --verbose` and never a file under `~/.claude/projects/`. The suite feeds the transform a harness log whose `thinking` and `text` blocks carry content shaped like transcript fields, and asserts that no byte of any block the harness did not tag as a call or a result survives. A case that only asserts that the thinking block is gone passes a filter by position, a filter by index and a filter over the tags in one sample, so it asserts the bytes. It also holds spec 15's three-state rule, where `calls: []` is a watched session that made no call and an absent `calls` key is a session nothing watched, and the transform refuses a stream carrying no `system`/`init` line rather than claiming an observation nobody made. Two properties are held by mutation rather than by a needle, because both went unheld under a suite that looked complete: the content digest, where replacing `sha256sum` with a constant has to fail, and the encoder, where weakening it to a line-based one has to fail at every entry point that takes a scalar. It needs `jq`, which CI treats as a skip when absent.

`sh tools/run/run-census-fixtures.sh` holds `tools/run/run-census.sh`, which reads one session transcript and prints what its turns were spent on: one usage record per message id, and every `Bash` call grouped by its leading verb, with the turns and the cache reads each group cost. It then reads the agent transcripts the harness writes beside the session file and prints the fleet: the share of the span with no agent in flight, the mean number in flight, the gap around each compaction, and the parent's turns per agent of each type. It is the measurement behind [HW-PD-0003](docs/process/decisions/0003-a-dispatch-pays-when-it-retires-more-parent-turns-than-it-costs.md) as a tool, so a run is held to the numbers the evaluation records rather than to a memory of them. It gates nothing.

`sh .claude/skills/fixtures.sh` holds every claim the skill and agent files make about the engine. It is a blocking CI step, half of its cases are derived from the files rather than listed, and it writes only into a scratch copy of the corpus. `headwater sweep` is the one mechanism among the skills that no engine performs: `headwater sweep plan` writes the briefing, a model reads the documents, and `headwater sweep report` says what the engine could confirm about what came back. Nothing gates on it, and no crate of this engine opens a socket.

### What holds the first screen, and what still does not

`docs/` is the corpus root, so no rule of this engine reads the root `README.md`. `docs_dir: docs`, so `mkdocs build --strict` never reads it either, and every `README` named in a workflow, a hook or a script under `tools/` is `engine/README.md` or `docs/interfaces/README.md`. A British spelling, a contraction, a hard-wrapped block, four retired terms, a dead link and a dead fragment planted in that file produce exit 0 and a byte-identical check report.

`sh tools/repo/readme-fixtures.sh` closes two of those eight, and it is worth being exact about which. It runs in CI and enumerates each population from the page rather than listing it. It holds the referential integrity of the page — every relative link resolving to a file that exists, and every fragment resolving to a heading that exists — and it holds five claims the page makes about itself: the opening image exists at 1280 by 640 under 1 MB, every GitHub URL names the owner and repository `git remote get-url origin` names, the exclusive or below, no count of milestones, and the command the page tells a newcomer to run. It reads links per occurrence and not per line, because this repository forbids hard-wrapped Markdown and a line-shaped count of a page whose paragraphs are single long lines is off by a factor of three.

The fifth claim is a different kind from the other four, and it is the only place in this repository where a gate runs a command out of a page rather than reading one. The page told a newcomer to run `headwater taxonomy vendor --expect <digest>`, which names no directory. `vendor` takes the path of a package somebody already fetched, so that command refused on its grammar before it read `--expect`: it exited 1 for every reader, and it printed the same bytes whether the digest they pasted matched or not. Every gate on that page read what the page said and none of them ran it, so nothing saw it. Case group 6 extracts the invocation from the page, runs it against the newer of the two engine profiles with the digest read from `.headwater/packages/headwater-standard/release.yml`, and then again with that digest zeroed, and judges two properties together: the stated digest is accepted, and the two reports differ. The second is the half a pair of exit statuses cannot see. That group needs a built engine, takes neither on faith and goes red rather than skipping when there is none, and it runs over a copy of the pin and the package under `mktemp -d`, because the arm that succeeds installs a package over the directory it is handed.

The case it exists for is the paste block. Either the tag the fence pins resolves against the remote, or the paragraph directly above the fence says the tag is not cut and tells the reader to omit the line — never both, and never neither. So the change that cuts that tag cannot merge until it deletes that sentence, and a change that deletes the sentence early cannot merge until the tag exists. The tag is read out of the fence rather than written into the suite, and the four states are driven over scratch pages and stub bare repositories.

**The prose on that page still answers to nobody, and this suite does not change that.** A British spelling, a contraction, a hard-wrapped paragraph, a retired term and `headwater` written lower case in running prose were planted on the page together, and the suite reported 37 of 37 passing. The language regime binds the kinds declared under the corpus root, and the README is not a governed document, so this suite deliberately reimplements none of those rules: a second copy of a rule living in a script is what every other paragraph here refuses. Whether the regime should reach a file outside `docs/` is a question about the taxonomy and not a gap in this suite.

Two further things on that page answer to nobody. An absolute URL is never fetched, so a dead external link stays green. The Status blockquote names the open milestones, and the suite refuses only a **count** of them, because a count is what went stale once and the authority for one is the GitHub API — which a gate here does not open a socket to, the same posture `headwater probe` takes.

**A false positive here costs more than a missed defect.** A required check on the most-read page in the project that reddens on correct Markdown is a check the first person it annoys turns off, after which it guards nothing. Two shapes reached the first cut of this suite: a CommonMark link title, `[a](p "t")`, parsed into the path and reported as a missing file, and a link printed as an example inside an inline code span, read as a link. Both are cases now, and removing either fix turns two cases red.

### What holds the command an outsider runs

`engine/README.md` carries two `docker run` recipes, and between 2026-08-26 and 2026-09-07 both exited 101 before compiling a line. `ordered-float` had entered `engine/Cargo.lock` under `saphyr` declaring a floor of 1.90, both recipes pinned a 1.85 image, and no crate of this workspace declared `rust-version`, so cargo never read the floor either. This repository went public with both commands dead, and a change on that day edited both of them without running either.

`[workspace.package]` in `engine/Cargo.toml` now declares `rust-version`, every crate declares `rust-version.workspace = true`, and `engine/clippy.toml` states the same number as its `msrv`. `sh tools/engine/engine-readme-fixtures.sh` is what holds them: its first case compares the highest `rust-version` across `cargo metadata --locked` against the tag the recipes pin, so a `cargo update` that raises the floor reddens the same day. It runs no container and builds nothing, which is why it can be a required step. It also holds the four things those recipes explain and nothing else read — `--user` on the test half and root on the other, the `git rev-parse --show-toplevel` mount, the two blocks that a blank line alone would render as one `<pre>`, and the absence of `-D warnings` — and it holds the nine files that state the floor to one another, enumerated from the tree by shape rather than listed. It follows the shape of `tools/repo/readme-fixtures.sh` and reads no cargo flag, because `tools/engine/build-declaration-fixtures.sh` owns those.

It does not run the container, and nothing does. A recipe that is well formed, correctly pinned and still broken stays green here, so run both commands by hand after changing either.

### The two declarations the build makes about itself

`engine/Cargo.lock` is committed, and `--locked` is what holds a cargo run to it. Every cargo step of `.github/workflows/ci.yml` that resolves a manifest, both container commands of `engine/README.md`, and every copy of `cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked` carry the flag. A maintainer's own loop does not, and that is deliberate: there a rewritten lock is the intended result of adding a dependency, and the flag would refuse it. `cargo fmt` is exempt because it rejects the flag, and `cargo --version` because it accepts and ignores it.

`[workspace.package]` in `engine/Cargo.toml` declares `license = "Apache-2.0"` and every crate manifest declares `license.workspace = true`. The workspace key alone declares nothing a reader sees: it is an inheritance source, and a member that omits its own line reports `license: null` to `cargo metadata`. Delete the workspace key and no cargo command runs at all, so that direction needs no guard. A crate added later that forgets the member line is the direction that stays silent.

`sh tools/engine/build-declaration-fixtures.sh` holds both, enumerates each population from the tree rather than listing it, and runs in CI. It reads the workflow by parsing every `run:` value, so a cargo step written behind a `cd`, an environment assignment, a nested shell or a block scalar is in the population too, and it judges the install command one occurrence at a time rather than one line at a time, because a line filter goes quiet on a paragraph carrying both forms. So a crate you add declares the license, and a cargo invocation you add to CI or a copy of the install command you write carries `--locked`. No count of either population is written down here, because a number no judge reads drifts.

## Where the rest lives

[`CLAUDE.md`](CLAUDE.md) is the short list of conventions every agent obeys, and *What holds this repository* above is the fullest account of what each gate holds and why it exists at all. [`engine/README.md`](engine/README.md) is the reference for each crate and for what its tests cover. [`docs/tutorials/your-first-governed-corpus.md`](docs/tutorials/your-first-governed-corpus.md) takes a reader from an empty directory to a passing strict run, and a suite runs every command in it and diffs the output against the page.

If you work through an AI harness, `.claude/`, `.codex/` and `.github/hooks/` register the same three scripts for three harnesses. None of them blocks anything. What holds a change is the commit gate and the CI job above.
