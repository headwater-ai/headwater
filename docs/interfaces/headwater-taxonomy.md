---
id: HW-IFACE-headwater-taxonomy
status: current
status_since: 2026-09-06
summary: "How to validate, resolve, audit, publish, vendor, compare and migrate taxonomy packages."
last_verified: 2026-09-07
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

`publish --json` writes the release record as one JSON document on standard output, in place of the paragraph a person reads. The document names the package, the output directory, the digest a consumer pins, and every member with its own digest. A `delivery` member says how the artifact reached the output directory. The value is `renamed` where one rename moved the assembled artifact into place, and `direct` where the artifact went in file by file. A `direct` publish is not atomic, and a run stopped part way leaves files there with no release record. The member is present under both values, so a consumer tells the two apart. The reason for a `direct` value is one line of prose on standard error, under both output modes. The document names its own shape in a `version` member, so a consumer pins that rather than the version of this engine.

`publish --clear-killed` removes what a publish that was killed part way left at the output directory. It then publishes into that directory in the same run. It removes one state and no other. That state has two halves. The first half is files at the output directory with no release record. The second half is a directory beside it, named for the output directory and the suffix `~staging`. That directory holds two files. One is the file a publish writes to claim the directory. The other is the file a publish writes when it cannot move the artifact into place. Only a killed publish leaves those two files together, and the second file names the output path it was writing. The flag removes nothing where the output directory carries a release record. It removes nothing where those two files are not together beside it. The publish then refuses as it refuses without the flag. The run reports on standard error what it removed.

`vendor` reads a fetched artifact into the package area after digest validation. Where a package is already installed at the same version and a different digest, the run reports the pair. It names both digests and both member counts, and it says that this version number now names two sets of bytes. It installs the artifact and exits 0. Nothing refuses a second artifact under a version already published, so this report is the one place an adopter sees it. `diff` compares a fetched artifact with the current taxonomy. `migrate` reports migration steps and writes them only with `--apply`.

`diff` reads the lock and it re-resolves no source. It states two caveats where they hold, and it gates on neither. The first caveat is a base that resolved to the same text beside a broken `addressability`. A lock that lost its founding record produces that pair. So does a release that moves a declaration between two bundles whose operations commute, and that lock is current. The run separates neither, so the caveat states both readings and refuses nothing. The second caveat names the source files the lock records that have since changed on disk.

`diff` does not apply the test that `resolve --check` applies. That test refuses the publisher who edits a package source in place, which is the correct run this verb is written for. It also passes a lock that a person resolved after the candidate was installed, where `diff` reports the wrong answer.

## Preconditions

The consumer declaration and package sources must be readable for source operations. Package and artifact paths must exist for `publish`, `vendor`, `diff` and `migrate`. A write operation must satisfy its directory, digest, version and migration preconditions.

## Options

| Word and options | What it does |
|---|---|
| `validate` | Validates taxonomy sources without writing. |
| `resolve [--check]` | Writes or checks `.headwater/taxonomy.lock`. |
| `audit [--now <date>] [--record]` | Measures the taxonomy against the corpus. `--record` appends one adoption reading to `.headwater/adoption.jsonl`, and refuses a reading the store already holds at this lock and this date. |
| `publish [--package <name>] [--from <dir>] [--assembly <name>] [--out <dir>] [--clear-killed] [--json]` | Writes a package artifact and release record, or a flattened artifact from the named assembly. `--clear-killed` removes the files a killed publish left at the output directory, and publishes in the same run. `--json` writes the record as one JSON document on standard output. |
| `vendor <dir> [--expect <digest>]` | Installs a fetched artifact after digest validation. It reports a second artifact installed under the version already there, with both digests and both member counts. |
| `diff <dir> [--to <version>] [--now <date>]` | Compares a fetched artifact with the current taxonomy. |
| `migrate <dir> [--to <version>] [--apply] [--now <date>]` | Reports or applies migration steps. It reads the version it migrates from out of the lock header against no pin, and it refuses a transition that is not forward. |
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
| `.headwater/taxonomy.lock` | Written by `resolve` without `--check`. Read by `diff`, which also re-hashes the source files it records. |
| `.headwater/adoption.jsonl` | Read by `audit`, and appended to by `audit --record`. |
| Package and artifact directories | `publish` and `migrate --apply` write a package directory, and `diff` and `migrate` read one. `vendor` reads the artifact it installs, and it writes `.headwater/packages/<name>`, `.headwater/packages/~staging/<name>` and `.headwater/packages/<name>~aside`. It writes no other path in the consumer tree. |
| `packages/`, the root before engine 0.2.0 | Read by no verb. A lookup that finds no package, on a tree that carries a directory there, names that directory and says to move what is under it. |

## See also

[`headwater init`](headwater-init.md) creates the consumer declaration. [`headwater generate`](headwater-generate.md) updates projections after resolution. [`headwater import`](headwater-import.md) consumes a pinned external snapshot.
