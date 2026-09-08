---
id: HW-DR-0060
status: draft
status_since: 2026-09-08
summary: "A binary's `--version` keeps naming one number, the `Cargo.toml` version a `requires_engine` range is read against. Which commit built it is a separate fact a build script now captures, kept out of that comparison and not yet on the command line."
last_verified: 2026-09-08
title: "The engine's version stays one number, and a build's exact commit is a separate, unwired fact"
---

# The engine's version stays one number, and a build's exact commit is a separate, unwired fact

## Context

Publishing headwater-cli to crates.io (#735) raised a question. Can a reader tell whether a `headwater` binary came from the exact commit a tag names, or from an arbitrary commit on `main`?

`headwater --version` already answers a narrower question. It prints `headwater_resolve::release::ENGINE`, the workspace version from `Cargo.toml`. A caller pastes one line into a bug report, and a reader compares it against a `requires_engine` range. Three tests hold that contract: one line, nothing on standard error, and the named constant read from source.

HW-DR-0042 fixes a second budget: every global flag costs one line of the first help screen, and the ceiling that follows is 60 lines. The screen already measures past that ceiling today, so a new global flag has no line to spend.

## Decision

A build script in `headwater-resolve` runs `git describe --always --dirty --tags` and stores the result in a new constant, `release::BUILD_COMMIT`. Nothing reads this constant today. `--version`, the banner, and every `requires_engine` comparison keep reading `ENGINE` alone, unchanged.

When the build runs outside a git checkout, `BUILD_COMMIT` is `None` rather than a build failure. A crate that `cargo publish` uploads carries no `.git` directory. A build from the crates.io tarball is exactly this case. Every crate this bootstrap still needs to publish depends on that build's success with no `.git` present.

A test in `headwater-resolve` checks that a value, when present, holds one token with no whitespace.

## Consequences

A caller cannot yet ask which commit built a binary. That capability exists in the library. It waits for a CLI surface a later decision spends first-screen budget on, or for a reader who compiles `headwater-resolve` directly.

`ENGINE` still means one thing: the version a range is read against. A future flag that prints `BUILD_COMMIT` reads a second, independent fact. The two values never merge into one string a test has to parse apart again.

The build script adds no dependency and reaches no network. It shells out to `git`, already required to build this workspace from a checkout, and a failure to run it is silent rather than fatal.
