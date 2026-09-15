---
id: HW-IFACE-headwater-generate
status: current
status_since: 2026-09-06
summary: "How to write the projections declared by the taxonomy and detect stale generated files."
last_verified: 2026-09-12
title: "headwater generate"
relations:
  governs:
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
    - engine/crates/generate/src/lib.rs
---

# headwater generate

## Synopsis

    headwater generate [--check] [--root <path>]

The command writes every generated projection that the resolved taxonomy declares.

## Description

Without `--check`, it computes projections and writes marked generated files. With `--check`, it writes nothing and compares committed projections with the same plan.

It refuses to overwrite a file that lacks the generated-file marker. A stale projection or a marked file that the plan does not write is an error.

**The command also refuses a declaration whose kind requires what the generated file cannot carry, and it writes no file for that declaration.** Two refusals have this shape. The first reads the `facets.require` list of the declared kind. The front-matter block states the identifier, the discriminator of a heterogeneous shelf, and the facet in the `name` role. The command derives the state, the two dates and the summary. A required facet outside that set is one that no author can add, because the only writer of a generated document is this engine.

The second reads the `sections.require` list of the same kind against the body that the emitter composes. No emitter of this engine reads a section contract. The headings of a generated body are the name of a shelf and the names of the documents on it. So a required section that no heading answers is also one that no author can add. Each refusal names the kind and the requirement, and the run prints it under *what this verb does not write, and why*.

**With `--check`, the command reads the producer identity before it compares the bytes of any projection.** The corpus descriptor at `.headwater/corpus.json` records the emitter set of the engine that wrote it. When that number is not this engine's, the two runs do not agree about what the projections are. A byte difference then has two possible causes. A corpus moved, or an emitter moved, and the command cannot tell which. It reports both numbers, it withholds the instruction to regenerate for every projection in the run, and it exits non-zero. When the numbers agree, or when the committed descriptor records no emitter set, the command behaves as it always did.

A descriptor that records no emitter set is a descriptor an earlier engine wrote. Absence is not disagreement, so the command reports no producer difference for it. The first run of this engine over such a repository writes the member, and the pair agrees from then on.

**An emitter set is a number a person raises, and the raise moves `.headwater/corpus.json` in every repository.** An emitter that changes the bytes it produces for a corpus it produced other bytes for before raises the number. The next `headwater generate` in an adopter's repository then writes a new descriptor. That cost is the point: a regenerate is deliberate, and the taxonomy lock format already behaves this way.

Under *what this verb does not write, and why*, a run reports every projection kind this engine does not emit. It reports a kind the taxonomy declared at the output path of that declaration, and a kind no declaration named at `no declaration names one`. A reason is a property of this engine rather than of the corpus, so a reader gets it before writing the declaration.

## Preconditions

The repository must have a readable consumer declaration, taxonomy lock, corpus and generated projection declarations. Existing generated targets must satisfy their write preconditions.

## Options

| Option | What it does |
|---|---|
| `--check` | Writes nothing and exits non-zero when a projection is stale, or when the committed corpus descriptor records an emitter set that is not this engine's. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that all declared projections match or were written.

**1** means that loading, planning or writing failed, that `--check` found drift, or that `--check` found a producer difference. The two `--check` failures print different sentences, because only one of them has a remedy this command can name.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock`, corpus and configuration | Read to build the projection plan. |
| `.headwater/corpus.json` | Read under `--check` for the emitter set it records, before any projection is compared. Written like any other projection. |
| Generated projection files | Written without `--check` when their marker permits it. |

## See also

[`headwater taxonomy resolve`](headwater-taxonomy.md) refreshes the lock. [`headwater export`](headwater-export.md) emits a selected profile.
