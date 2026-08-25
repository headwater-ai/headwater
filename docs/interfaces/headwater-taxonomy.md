---
id: HW-IFACE-headwater-taxonomy
status: draft
status_since: 2026-08-25
summary: "How to validate, resolve, audit, publish, vendor, compare and migrate taxonomy packages."
last_verified: 2026-08-25
title: "headwater taxonomy"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/resolve/src/lib.rs
    - engine/crates/audit/src/lib.rs
    - engine/crates/compat/src/lib.rs
---

# headwater taxonomy

## Synopsis

    headwater taxonomy <validate|resolve|audit|publish|vendor|diff|migrate> [options] [--root <path>]

The grouped command validates taxonomy sources, resolves the lock, measures schema use, publishes or vendors packages, compares versions and applies migration payloads.

## Description

`validate` checks taxonomy sources without writing. `resolve` writes the validated `.headwater/taxonomy.lock`, or checks that the committed lock is current with `--check`. `audit` reports schema measurements and remains non-gating.

`publish` writes a release artifact and release record. `vendor` reads a fetched artifact into the package area after digest validation. `diff` compares a fetched artifact with the current taxonomy. `migrate` reports migration steps and writes them only with `--apply`.

## Preconditions

The consumer declaration and package sources must be readable for source operations. Package and artifact paths must exist for `publish`, `vendor`, `diff` and `migrate`. A write operation must satisfy its directory, digest, version and migration preconditions.

## Options

| Word and options | What it does |
|---|---|
| `validate` | Validates taxonomy sources without writing. |
| `resolve [--check]` | Writes or checks `.headwater/taxonomy.lock`. |
| `audit [--now <date>]` | Measures the taxonomy against the corpus. |
| `publish [--package <name>] [--from <dir>] [--out <dir>]` | Writes a package artifact and release record. |
| `vendor <dir> [--expect <digest>]` | Installs a fetched artifact after digest validation. |
| `diff <dir> [--to <version>] [--now <date>]` | Compares a fetched artifact with the current taxonomy. |
| `migrate <dir> [--to <version>] [--apply] [--now <date>]` | Reports or applies migration steps. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Confirms color-free output. |

## Exit status

`validate`, `resolve`, `publish`, `vendor`, `diff` and `migrate` return **0** when their operation succeeds and **1** on refusal or write failure. `resolve --check` returns **1** for a stale lock. `audit` returns **0** after it reports its measurements.

## Environment

The command reads the system date when a subcommand has `--now` and no date is supplied. It reads no other environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml`, package sources and overlay | Read by validation and resolution. |
| `.headwater/taxonomy.lock` | Written by `resolve` without `--check`. |
| Package and artifact directories | Read by `vendor`, `diff` and `migrate`, and written by `publish` or `migrate --apply`. |

## See also

[`headwater init`](headwater-init.md) creates the consumer declaration. [`headwater generate`](headwater-generate.md) updates projections after resolution. [`headwater import`](headwater-import.md) consumes a pinned external snapshot.
