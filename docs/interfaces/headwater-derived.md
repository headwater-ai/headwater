---
id: HW-IFACE-headwater-derived
status: current
status_since: 2026-09-11
summary: "How the set of derived artifacts is computed from the producers that write them, and why no file of this repository holds that set as a list."
last_verified: 2026-09-11
title: "headwater derived"
relations:
  governs:
    - engine/crates/census/src/derived.rs
    - engine/crates/cli/src/lib.rs
    - engine/crates/cli/src/main.rs
---

# headwater derived

## Synopsis

    headwater derived [--root <path>]

The verb takes no argument. It reports the derived artifacts of the tree in front of it, and the two directions in which that tree disagrees with the report.

## Description

`headwater derived` answers one question: which files of this repository does a producer write? It asks each producer for its own output set, and the union of those sets is the answer. No file of this repository holds the set as a list, so a new producer output changes the answer with no edit to the engine.

Four producers answer, and each one has its own rule. `headwater generate` claims a file that carries the generated-file marker. `headwater taxonomy resolve` claims the committed lock. `tools/site/refresh-figures.sh` claims a page under `site/` that carries a `data-figure` element. A blessing run of the test suite claims a recorded corpus fixture whose opening states a fold.

The fourth rule reads the shape of the artifact and not the name of the file. [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) decomposed two recorded fixtures into one record for each entity, so that they merge correctly. A decomposed artifact must not declare the merge driver. What separates the two groups is the fold, and a fold shows in the first lines of the artifact as a count over the corpus or as a digest over the whole canonical text.

The verb then holds the computed set against the `merge=headwater-regenerate` attribute in `.gitattributes`, and it reports both directions. A producer output that carries no attribute merges as an ordinary file, and two branches that move it to one value merge it in silence. A path that carries the attribute and no producer writes refuses a merge of text that a person now edits. `.gitattributes` names the second failure the worse of the two.

The verb reads the tree and nothing else. It reads no lock, resolves no taxonomy and runs no producer. So it answers on a tree whose lock is stale, and on a tree in the middle of a merge, which are the two moments a caller asks the question.

**The report renders the palette [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules on, when standard output is a terminal.** The opening count is a heading. Each producer command carries the verb color, because it is the command a reader retypes. Every path carries the path color, on both sides of the answer. The two headings that report a disagreement carry the error color, and their agreement counterparts stay plain: the verb exits non-zero on exactly those two conditions, so the color says what the exit status says.

## Preconditions

The repository root must hold a readable `.gitattributes`. A root without one reports every producer output as undeclared.

No producer has to run first. The verb reads what each producer last wrote, and it does not compare that against what a producer would write now. `headwater generate --check` and `headwater taxonomy resolve --check` are the verbs that ask the second question.

## Options

| Option | What it does |
|---|---|
| `--root <path>` | Select the repository whose tree is read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. This report paints the opening count, each producer command, every path on both sides of the answer, and the two headings that report a disagreement. |
| `--no-banner` | Suppress the masthead. It is accepted here and does nothing, because only the root help screen prints one. |

The verb takes no option of its own. It computes one answer about one tree, and the tree is every input it has. `--format` and `--json` are not options of this verb.

## Exit status

**0** means that the computed set and the declared set agree in both directions.

**1** means that at least one direction disagrees. The report prints the whole answer, on standard output, under either status.

## Environment

No environment variable reaches this verb. The root comes from the command line, and every other input comes from the tree.

## Files

| Path | How this verb treats it |
|---|---|
| `.gitattributes` | Read for every path that declares `merge=headwater-regenerate`. |
| `.headwater/taxonomy.lock` | Claimed as the output of `headwater taxonomy resolve`. Its content is not read. |
| Every other file of the tree | Read, to ask each producer rule whether the file is its own. |

The verb writes no file. It changes no artifact and no cache.

## See also

[HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that a corpus-wide fold is derived and never stored, and it is the decision this verb computes the population of.

[`headwater generate`](headwater-generate.md) writes the largest of the four producer output sets, and `--check` holds every byte of it.

[Spec 6](../spec/06-engine-architecture.md#cli) lists the command line.
