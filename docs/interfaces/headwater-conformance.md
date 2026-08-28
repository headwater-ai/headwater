---
id: HW-IFACE-headwater-conformance
status: draft
status_since: 2026-08-25
summary: "How a package's rules measure adoption, how waivers affect a requested rung, and what exits non-zero."
last_verified: 2026-08-25
title: "headwater conformance"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/conformance/src/lib.rs
    - engine/crates/conformance/src/render.rs
    - engine/crates/conformance/src/json.rs
---

# headwater conformance

## Synopsis

    headwater conformance [--level <name>] [--now <date>] [--json] [--root <path>]

The verb evaluates the repository against the conformance rules in its selected package. It takes no operand.

## Description

`headwater conformance` reads the package rule set and evaluates the four readings this engine holds: `pin.current`, `lock.current`, `corpus.classified` and `projections.current`.

The report identifies the package, version, digest and date. It lists every rule, each level and the highest level reached by met rules. A level is cumulative, so it includes the rules of earlier levels.

A waiver covers a gap only for a requested level and only until its inclusive expiry date. It never changes the reported level, because the reported level reads met rules alone. A rule with no engine reading, an orphan waiver or malformed conformance data ends the run rather than being skipped.

Without `--level`, the verb measures and exits successfully even when gaps exist. With `--level`, it exits successfully only when every rule under that rung is met or covered by a live waiver. Text goes to standard output. JSON carries the same report and adds a `gate` member only when a level was requested.

## Preconditions

The repository must carry a readable `.headwater/taxonomy.lock` and consumer declaration. The lock supplies the package identity and the corpus configuration.

The selected package must exist under `packages/`, declare a conformance file and carry a conformance format this engine reads.

The projections must read from the resolved taxonomy. The host must provide a date, or `--now` must provide one in `YYYY-MM-DD` form.

## Options

| Option | What it does |
|---|---|
| `--level <name>` | Ask whether the named cumulative rung passes. A live waiver can cover a gap under this rung. |
| `--now <date>` | Evaluate expiry and dated readings against this date. |
| `--json` | Write the machine-readable conformance report. |
| `--root <path>` | Select the repository to evaluate. |
| `--no-color` | Confirm the binary's color-free output. It changes no byte. |

`--format` is not an option of this verb. `--wide` is refused because the verb prints a report rather than help. Global `--help` and `--version` are answered before the verb runs.

## Exit status

**0** means that evaluation completed. Without `--level`, this includes gaps, expired waivers and undecided attestations.

**1** means that the command line, taxonomy, package, conformance file, waiver set or date was refused, or that the requested level did not pass. An ordinary report is printed before a failed requested gate is reported on standard error.

**A level that did not pass still wrote its whole document, and a refusal wrote none.** Take `conformance --json --level L1` over a corpus that does not reach L1. It exits 1 with the whole document on standard output, because the gate is a member of that document. A refusal writes nothing there, and its account is one English sentence on standard error. So the property is that a refusal writes no document, and not that a non-zero exit writes none, which is what [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules.

**A rung the package does not declare is a refusal, and the two formats print it in a different order.** The text report is written before the gate is asked, so an undeclared rung is refused under the gaps it is about. The document cannot take that order, so `--json` writes nothing at all on that run.

## Environment

No environment variable reaches this verb. The package, taxonomy, repository and date come from the tree and command line.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock` | Read for the resolved taxonomy and lock digest. |
| `.headwater/taxonomy.yml` | Read through the consumer loader. |
| `packages/` and the selected package directory | Read for the package manifest, release record and conformance rules. |
| The corpus | Read for classification and projection checks. |

The verb writes no file.

## See also

[`headwater taxonomy vendor`](../spec/06-engine-architecture.md#the-verbs) installs the package whose conformance rules this verb reads.

[Spec 7](../spec/07-distribution-and-federation.md#conformance) defines conformance as wired adoption rather than a copied taxonomy.

[`headwater check`](headwater-check.md) reports findings over the corpus. Its `--strict` option is a separate check gate.
