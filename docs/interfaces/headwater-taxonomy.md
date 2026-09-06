---
id: HW-IFACE-headwater-taxonomy
status: current
status_since: 2026-09-06
summary: "How to validate, resolve, audit, publish, vendor, compare and migrate taxonomy packages."
last_verified: 2026-09-05
title: "headwater taxonomy"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/resolve/src/lib.rs
    - engine/crates/audit/src/lib.rs
    - engine/crates/audit/src/reading.rs
    - engine/crates/compat/src/lib.rs
---

# headwater taxonomy

## Synopsis

    headwater taxonomy <validate|resolve|audit|publish|vendor|diff|migrate> [options] [--root <path>]

The grouped command validates taxonomy sources, resolves the lock, measures schema use, publishes or vendors packages, compares versions and applies migration payloads.

## Description

`validate` checks taxonomy sources without writing. `resolve` writes the validated `.headwater/taxonomy.lock`, or checks that the committed lock is current with `--check`. `audit` reports schema measurements and remains non-gating. `audit --record` also appends this run's adoption reading to `.headwater/adoption.jsonl`, which is the one write this word performs.

`publish` writes a release artifact and release record. Before it writes anything it resolves the base with every bundle the package ships, and it holds every template the package ships against that resolution. It refuses a bundle set that does not resolve, and a template a person could copy into a document that resolves to no kind.

`publish --json` writes the release record as one JSON document on standard output, in place of the paragraph a person reads. The document names the package, the output directory, the digest a consumer pins, and every member with its own digest. It names its own shape in a `version` member, so a consumer pins that rather than the version of this engine.

`vendor` reads a fetched artifact into the package area after digest validation. `diff` compares a fetched artifact with the current taxonomy. `migrate` reports migration steps and writes them only with `--apply`.

## Preconditions

The consumer declaration and package sources must be readable for source operations. Package and artifact paths must exist for `publish`, `vendor`, `diff` and `migrate`. A write operation must satisfy its directory, digest, version and migration preconditions.

## Options

| Word and options | What it does |
|---|---|
| `validate` | Validates taxonomy sources without writing. |
| `resolve [--check]` | Writes or checks `.headwater/taxonomy.lock`. |
| `audit [--now <date>] [--record]` | Measures the taxonomy against the corpus. `--record` appends one adoption reading to `.headwater/adoption.jsonl`, and refuses a reading the store already holds at this lock and this date. |
| `publish [--package <name>] [--from <dir>] [--assembly <name>] [--out <dir>] [--json]` | Writes a package artifact and release record, or a flattened artifact from the named assembly. `--json` writes the record as one JSON document on standard output. |
| `vendor <dir> [--expect <digest>]` | Installs a fetched artifact after digest validation. |
| `diff <dir> [--to <version>] [--now <date>]` | Compares a fetched artifact with the current taxonomy. |
| `migrate <dir> [--to <version>] [--apply] [--now <date>]` | Reports or applies migration steps. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

`validate`, `resolve`, `publish`, `vendor`, `diff` and `migrate` return **0** when their operation succeeds and **1** on refusal or write failure. `publish --json` moves no exit status. A refusal under it writes no document on standard output, and its account is one English sentence on standard error. That is the rule [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) states for every `--json` this binary takes. `resolve --check` returns **1** for a stale lock. `audit` returns **0** after it reports its measurements, and **1** where `--record` cannot write or read the store.

## Environment

The command reads the system date when a subcommand has `--now` and no date is supplied. It reads no other environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.yml`, package sources and overlay | Read by validation and resolution. |
| `.headwater/taxonomy.lock` | Written by `resolve` without `--check`. |
| `.headwater/adoption.jsonl` | Read by `audit`, and appended to by `audit --record`. |
| Package and artifact directories | Read by `vendor`, `diff` and `migrate`, and written by `publish` or `migrate --apply`. |

## See also

[`headwater init`](headwater-init.md) creates the consumer declaration. [`headwater generate`](headwater-generate.md) updates projections after resolution. [`headwater import`](headwater-import.md) consumes a pinned external snapshot.
