# Developing Headwater

This page is the contributor loop: what to install, what to type, what CI will run against what you push, and the two failures that cost a newcomer an afternoon. [`CONTRIBUTING.md`](CONTRIBUTING.md) carries the terms you agree to and the sign-off every commit needs. [`CLAUDE.md`](CLAUDE.md) carries the authoring conventions that the prose under `docs/` answers to, and why each one is a check rather than a habit. This page carries neither of those, and it does not explain the design of the engine — [`engine/README.md`](engine/README.md) does that, crate by crate.

Read this before your first build. The gate list near the end is the part that goes stale fastest, so it is derived from the workflow by a suite rather than kept by hand.

## Before your first commit

Git does not install a repository's own hooks, so a fresh clone runs none of them. Enable them once:

    git config core.hooksPath .githooks

Keep that path relative. An absolute path makes every worktree of this repository run the main checkout's hook body instead of the one on its own branch, which is a defect that reads as the gate passing.

`.githooks/pre-commit` builds a change manifest from your working tree and then runs the engine over the corpus in strict mode. It refuses an error and reports everything else, so a commit that goes through is not a commit with nothing to answer for. Run the engine yourself to read the advisory findings.

`.githooks/pre-commit` needs a built engine and **fails open with one printed line when there is none**. A fresh worktree has no `engine/target`, so the hook there exits 0 without checking anything. Do not read that silence as a pass.

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

Every verb of that binary takes `--root`, which names the corpus to read. Without it a verb reads the current directory, and the current directory after a build is the engine workspace rather than the corpus.

`--locked` on that command is not decoration. `engine/Cargo.lock` is committed, and the flag is what holds a cargo run to it. Every cargo step in CI carries it, and so does every copy of the install command in this repository. A maintainer adding a dependency drops the flag deliberately, because there a rewritten lock is the intended result.

## A faster build, and the one failure it causes

`tools/dev-fast-build-setup.sh` wires `mold` and `sccache` into your own `~/.cargo/config.toml`. It is optional, it is idempotent, it installs nothing itself, and `--remove` undoes exactly the block it wrote. Nothing in CI or in a fresh clone depends on either binary.

Two things to know before you run it.

**sccache cannot cache an incremental build.** `cargo check` and `cargo test` use incremental compilation by default, so sccache does close to nothing for them; cargo's own incremental cache already covers that case. It earns its place on `--release`, on `--profile dev-release`, and on a clean or cross-branch rebuild. `mold` helps every link.

**sccache shares one object cache across every checkout on the machine, and some test binaries bake their own path in.** A test that resolves a fixture directory from `env!("CARGO_MANIFEST_DIR")` compiles that absolute path into the object. Compile it inside a worktree, delete the worktree, and a later run in the main checkout can get a cache hit on the stale object and fail on a path that no longer exists. **A cargo test failure that quotes a path under `.claude/worktrees/` is a cache hit, not a defect in your change.** The remedy is one command:

    cargo clean -p <crate>

Then run the test again. Because `cargo test` stops at the first failing target, one poisoned crate can hide the rest of the suite behind it.

## What CI runs

`.github/workflows/ci.yml` defines two jobs, `engine` and `headwater`, and a pull request triggers both. Everything below is blocking. This list is checked against the workflow by `sh tools/developing-fixtures.sh`, in both directions, so a gate added to CI and not written here fails, and a gate written here that CI does not run fails too.

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
    sh .claude/tutorial/fixtures.sh
    sh tools/assemble-site.sh --check
    sh tools/build-declaration-fixtures.sh
    sh tools/developing-fixtures.sh
    sh tools/engine-readme-fixtures.sh
    sh tools/id-store-fixtures.sh
    sh tools/readme-fixtures.sh
    sh tools/refresh-crawler-files.sh --check
    sh tools/refresh-site-tokens.sh --check
    sh tools/site-console-fixtures.sh
    sh tools/site-fragments-fixtures.sh
    python3 tools/check-site-console.py
    python3 tools/check-site-fragments.py

Each one holds an artifact that no rule of the engine reads: a workflow, a manifest, a page outside the corpus root, a hook, a skill file. They are cheap, they need no container, and running the ones your change touches before you push saves a round trip.

Two things are deliberately absent from that list. `.githooks/change-manifest` is a producer the workflow calls rather than a gate that can fail on its own. `.claude/hooks/fixtures-live.sh` spends real AI credits against a real login, so no job runs it and nothing gates on it.

## Where the rest lives

[`CLAUDE.md`](CLAUDE.md) is the fullest account of what each gate holds and why it exists at all. [`engine/README.md`](engine/README.md) is the reference for each crate and for what its tests cover. [`docs/tutorials/your-first-governed-corpus.md`](docs/tutorials/your-first-governed-corpus.md) takes a reader from an empty directory to a passing strict run, and a suite runs every command in it and diffs the output against the page.

If you work through an AI harness, `.claude/`, `.codex/` and `.github/hooks/` register the same three scripts for three harnesses. None of them blocks anything. What holds a change is the commit gate and the CI job above.
