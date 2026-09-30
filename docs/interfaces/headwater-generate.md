---
id: HW-IFACE-headwater-generate
status: current
status_since: 2026-09-06
summary: "How to write the projections declared by the taxonomy and detect stale generated files."
last_verified: 2026-09-30
title: "headwater generate"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/generate/src/lib.rs
    - .headwater/corpus.json
---

# headwater generate

## Synopsis

    headwater generate [--check] [--root <path>]

The command writes every generated projection that the resolved taxonomy declares.

## Description

Without `--check`, it computes projections and writes marked generated files. With `--check`, it writes nothing and compares committed projections with the same plan.

It refuses to overwrite a file that lacks the generated-file marker. A stale projection or a marked file that the plan does not write is an error.

**A run that refuses writes nothing.** The command finds the verdict of every output before its first write. When one of these six refusals applies, the command writes no file, and it exits 1:

- more than one transcript on one side of a pair of arms
- a pair of arms that holds a refused session that the recorder or the probe declaration caused
- a refused transcript whose state holds the refusal
- a marked file that no declaration writes
- an unmarked file at an output path
- an output whose marker the census does not read

The report lists each output that it did not write with the line `not written: this run refused before its first write, and it wrote nothing`. A refused transcript whose state releases the refusal is not a refusal of the run, so the command writes its result and continues.

Three failures can still come after a write, because no check before the first write can find them. The operating system can refuse a write. A projection can read a value that another projection writes in a cycle, and only a write shows the cycle. A later pass can find a new refusal, and only a defect in this engine causes that, because the first pass writes no transcript. These are failures of the disk or of the engine, and not refusals of the corpus.

**A `graph_export` that declares `committed: false` is not written and not compared.** The tree does not hold that file, because `headwater export` builds it at publish time. The run names the path with the line `built at publish time by headwater export, and not written or compared here`, and that line is not an error. The command does not read a copy that a local export left at the path, and it does not report that copy as orphaned.

**The command also refuses a declaration whose kind requires what the generated file cannot carry, and it writes no file for that declaration.** Two refusals have this shape. The first reads the `facets.require` list of the declared kind. The front-matter block states the identifier, the discriminator of a heterogeneous shelf, and the facet in the `name` role. The command derives the state, the two dates and the summary. A required facet outside that set is one that no author can add, because the only writer of a generated document is this engine.

The second reads the `sections.require` list of the same kind against the body that the emitter composes. No emitter of this engine reads a section contract. The headings of a generated body are the name of a shelf and the names of the documents on it. So a required section that no heading answers is also one that no author can add. Each refusal names the kind and the requirement, and the run prints it under *what this verb does not write, and why*.

**With `--check`, the command reads the producer identity before it compares the bytes of any projection.** The corpus descriptor at `.headwater/corpus.json` records the emitter set of the engine that wrote it. When that number is not this engine's, the two runs do not agree about what the projections are. A byte difference then has two possible causes. A corpus moved, or an emitter moved, and the command cannot tell which. It reports both numbers, it withholds the instruction to regenerate for every projection in the run, and it exits non-zero. When the numbers agree, or when the committed descriptor records no emitter set, the command behaves as it always did.

A descriptor that records no emitter set is a descriptor an earlier engine wrote. Absence is not disagreement, so the command reports no producer difference for it. The first run of this engine over such a repository writes the member, and the pair agrees from then on.

**An emitter set is a number a person raises, and the raise moves `.headwater/corpus.json` in every repository.** An emitter that changes the bytes it produces for a corpus it produced other bytes for before raises the number. The next `headwater generate` in an adopter's repository then writes a new descriptor. That cost is the point: a regenerate is deliberate, and the taxonomy lock format already behaves this way.

Under *what this verb does not write, and why*, a run reports every projection kind this engine does not emit. It reports a kind the taxonomy declared at the output path of that declaration, and a kind no declaration named at `no declaration names one`. A reason is a property of this engine rather than of the corpus, so a reader gets it before writing the declaration.

A `consumer_surface` declaration writes one page from the `surface` block of the taxonomy. The page lists the archive, the integration points with what each one needs, the prerequisites, the companions and the commands. It does not list `adopter_documents` or `local_roots`, because they configure a check and an adopter does not receive or run them. A taxonomy with no `surface` block writes no page, and the run reports the declaration under *what this verb does not write, and why*. A change to the block makes the page stale, and `--check` reports it.

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

**1** means that loading, planning or writing failed, that `--check` found drift, or that `--check` found a producer difference. The two `--check` failures print different sentences, because only one of them has a remedy this command can name. Without `--check`, a refusal leaves the tree as the command found it.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock`, corpus and configuration | Read to build the projection plan. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| `.headwater/corpus.json` | Read under `--check` for the emitter set it records, before any projection is compared. Written like any other projection. |
| Generated projection files | Written without `--check` when their marker permits it and the run refuses nothing. A `graph_export` output that declares `committed: false` is not written and not read. |

## See also

[`headwater taxonomy resolve`](headwater-taxonomy.md) refreshes the lock. [`headwater export`](headwater-export.md) emits a selected profile.
