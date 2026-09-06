---
name: headwater-engine
description: Build the engine of this repository and run its verbs. Use before the first cargo or CLI command of a session, when a build fails on the toolchain, when a test suite has to be named, and whenever a verb has to run against a corpus that is not the current directory. It carries the invocation and the toolchain floor, and it sends every rule about the taxonomy and the corpus to the skill that owns it.
---

# Headwater engine

The engine is a cargo workspace under `engine/`, and the corpus it reads is the repository above it. Two directories, so every command below states which one it means.

[engine/README.md](../../../engine/README.md) is the reference for the design of each crate and for what its tests hold. This file is the shorter thing: what to type, and the five mistakes that cost a session more than they should.

## The invocation

    cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked
    engine/target/release/headwater check --root .

Build from the repository root with `--manifest-path`, or from `engine/` with neither flag. Both write the same binary. Pick one form and keep it for the session, because a relative path that was right in one directory is a missing file in the other.

**Every verb takes `--root`.** It names the corpus to read, and it is what makes the binary runnable from anywhere. A verb with no `--root` reads the current directory, which is the engine workspace whenever the last command was a build. Pass it, and no command in a session needs a `cd` in front of it.

**This is the invocation for the binary itself, and not the default loop.** A session that is writing or checking a change stays on `cargo check` and `cargo test` — see the fifth mistake below.

## The toolchain floor

Rust 1.85 or later. `saphyr-parser` is on edition 2024, and an older cargo reports `feature edition2024 is required` and nothing else. Check the toolchain first when a clean checkout will not build, because that message names no crate and reads like a corrupt tree.

A machine that installed Rust from its distribution usually has neither rustfmt nor clippy. `engine/README.md` carries the container that supplies both and pins the floor at the same time. Run it before a change to the engine is proposed: CI runs `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`, and both are blocking. Clippy on the current stable knows lints that the pinned floor does not, so a clean container run is not a clean CI run.

It is two `docker run` commands there and not one. The first installs the components as root, which is the only user `rustup` can write for in that image. The second passes `--user` and runs `cargo test`, because three tests require a process that a `0444` file can stop and root is not one. Run both and read each exit status on its own, and never join them with a pipe. Do not add `-D warnings` to the container half: the workspace denies a clippy lint that 1.85 does not know, so the flag turns `unknown lint` into an error in every crate there.

## Which test suite to name

    cargo test                                   # from engine/, the whole workspace
    cargo test -p headwater-check                # one crate
    cargo test -p headwater-probe --test fixtures  # one file

CI runs the workspace with no filter, so the workspace is the bar. Name a crate to shorten a loop, and run the workspace before proposing the change.

A recorded fixture is re-recorded with `HEADWATER_BLESS=1` and never edited by hand. The digest of the lock reaches `.headwater/corpus.json` and several recorded fixtures, so a taxonomy change moves files that the change itself did not touch. Read that diff rather than blessing past it.

## The five mistakes

**`cargo test` from the repository root.** There is no manifest there. The workspace is under `engine/`, and the error names a missing `Cargo.toml` rather than the directory you are in.

**`--no-cache` on every run.** The cache is a correctness root: spec 12 requires that `headwater check` and `headwater check --no-cache` write the same bytes, and three test suites hold that equality. So the flag buys nothing during authoring. Reach for it when the question is whether the cache itself is wrong, and let CI carry it the rest of the time.

**Reading the corpus to answer a question about one document.** `headwater explain` prints the kind, the purpose, the summary, the warrant, the required facets and every edge in and out. That is the orientation, and [headwater-orient](../headwater-orient/SKILL.md) is the skill for it.

**A green run read as a green corpus.** `headwater check` exits 0 with findings on standard output, because the posture is advisory. `--strict` is the gate, and it is what `.githooks/pre-commit` runs. Read the findings.

**A `--release` build as a verification step.** `.github/workflows/ci.yml` already runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the whole test suite, `taxonomy resolve --check`, `generate --check` and `headwater check` on every pull request. A debug `cargo check` and `cargo test` prove the same fix, in seconds rather than the minutes `lto = true` and `codegen-units = 1` cost a release link, and a session that also builds `--release` and runs the binary by hand to double-check a passing test suite is spending real time and real machine load on evidence it already had. Push and read CI rather than reproducing it locally. Reach for `--release`, or `--profile dev-release` for a faster link at a smaller optimization cost, only when the session needs the binary itself: to hand it to somebody, to run it once by hand against a real corpus, or to measure a performance claim, which debug and release answer differently by roughly an order of magnitude.

`engine/.cargo/config.toml` names four aliases for the invocations on this page that cargo has no shorthand for, so a session reaches for the cheap one by name instead of retyping the flags that make it cheap: `test-crate <crate>` (`test -p <crate>`), `test-timings` (`test --workspace --timings`, an HTML report under `target/cargo-timings/`), `release-cli` and `dev-release-cli` (the two builds in the paragraph above). Each is a rename of a flag combination already explained here, not a new behavior, so nothing depends on a session using them. There is no alias for plain `check`/`test`: cargo already ships `c` and `t` for those, and this workspace has no `default-members`, so `cargo c`/`cargo t` from `engine/` already cover the whole workspace.

## Optional: a faster linker and compile cache

`tools/dev-fast-build-setup.sh` wires `mold` (linker) and `sccache` (compile cache) into `~/.cargo/config.toml` — the user's own, not `engine/.cargo/config.toml`, and never checked into a repo, because CI and a fresh clone have neither binary and must not start depending on them. It is idempotent, installs nothing itself (it names the two binaries and stops if either is missing, rather than writing a config that would break every cargo invocation on a rustc-wrapper it can't find), and `--remove` undoes exactly the block it wrote. On a host that also runs a self-hosted CI runner as the same OS user, that runner reads the same file, so check there before suspecting the repo if a runner build starts behaving differently.

The one gotcha worth knowing before setting this up: sccache cannot cache an incremental build, and `cargo check`/`cargo test` use incremental compilation by default, so sccache gives it roughly zero benefit there — cargo's own incremental cache already covers that case. sccache earns its place on the builds that already run without incremental: `--release`, `--profile dev-release`, and a clean or cross-branch rebuild. mold helps every link regardless of incremental.

## What this skill does not decide

A change to `packages/`, `docs/taxonomies/` or `.headwater/overlay.yml` is a taxonomy change, and [headwater-taxonomy](../headwater-taxonomy/SKILL.md) carries it. A new rule is declared there and implemented in `engine/crates/check/`, and the bar that a check without a failing fixture does not ship is stated there rather than here.
