---
id: HW-IFACE-headwater-derived
status: current
status_since: 2026-09-11
summary: "How the derived artifacts and the shape of each one are computed from the producers, rather than declared in any file."
last_verified: 2026-09-11
title: "headwater derived"
relations:
  governs:
    - engine/crates/census/src/derived.rs
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
---

# headwater derived

## Synopsis

    headwater derived [--root <path>]

The verb takes no argument. It reports the derived artifacts of the tree in front of it, the shape of each one, and every disagreement it finds.

## Description

`headwater derived` answers one question: which files of this repository does a producer write? It asks each producer for its own output set, and the union of those sets is the answer. No file of this repository holds the set as a list, so a new producer output changes the answer with no edit to the engine.

Four producers answer, and each one has its own rule. `headwater generate` claims a file that carries the generated-file marker. `headwater taxonomy resolve` claims the committed lock. A script local to this repository claims a page of its hand-built site that carries a `data-figure` element. A blessing run of the test suite claims a recorded corpus fixture whose opening states a fold.

The fourth rule reads the shape of the artifact and not the name of the file. [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) decomposed two recorded fixtures into one record for each entity, so that they merge correctly. A decomposed artifact must not declare the merge driver. What separates the two groups is the fold, and a fold shows in the first lines of the artifact as a count over the corpus or as a digest over the whole canonical text.

The verb then holds each fold of the computed set against the `-merge` and `merge=headwater-regenerate` attributes, and it reports both directions. A fold that carries neither merges as an ordinary file, and two branches that move it to one value merge it in silence. A path that carries `merge=headwater-regenerate` and no producer writes refuses a merge of text that a person now edits. A literal `-merge` line of the root `.gitattributes` is held in this direction too. The `binary` macro and a pattern are not, because they are how a repository marks a file that a person makes, such as an image. `.gitattributes` names the second failure the worse of the two.

The verb reads the tree. It also asks git two questions about the tree: which paths git ignores, and which merge attribute git gives each path. It reads no lock, resolves no taxonomy and runs no producer. So it answers on a tree whose lock is stale, and on a tree in the middle of a merge, which are the two moments a caller asks the question.

### Where the merge attribute comes from

**Inside a git repository, git gives the merge attribute.** The verb runs `git check-attr merge` one time, for every file of its walk, every literal path of the root `.gitattributes`, and the lock. Git applies its own precedence. A `.gitattributes` in a deeper directory wins over one nearer the root, `$GIT_DIR/info/attributes` wins over every `.gitattributes`, and `core.attributesFile` applies below all of them. Git also expands its own patterns and macros, so the `binary` macro and `-merge` both give a path no merge attribute. The answer of the verb is the answer that a merge gets.

The verb asks about a literal path of the root file without its leading `/`, because git refuses a path that begins with one. It does not ask about a path that climbs out of the tree through `..`. Git refuses that path too, and a pattern that names it matches no file of the tree.

**Where git refuses the question inside a repository, the report says so.** The verb then uses the root-file reader for the whole tree. The report prints what git printed, and it states that nested files, `info/attributes` and `core.attributesFile` were not read. It also names each nested `.gitattributes` that the walk finds, as it does outside a repository. The verb exits 1.

**Outside a git repository, the verb reads the root `.gitattributes` alone.** It reads that file as a list of literal paths. A leading `/` of a line anchors the pattern to the root and is not part of the path, so this reader also removes it. The reader does not expand a pattern, and the report names each pattern that carries a merge attribute. The reader does not read a nested `.gitattributes`. The report names each nested file that the walk finds, under its own heading, and the verb exits 1. A nested file can set or unset a merge attribute. A report that passed over it would state agreement for a layout that the verb did not read. No merge reads the attributes of such a tree, so this reader is for a tree that is not a repository yet.

The verb is not on the check-evaluation path. [Spec 12](../spec/12-check-layer.md) keeps version control commands off that path, and `headwater check` does not call this verb.

### The shape of a record, and the treatment that shape takes

The verb also names the shape of each reported path, and it holds that shape against the merge attribute the path carries. [*The shapes a record takes*](../evaluations/what-a-check-can-know.md#the-shapes-a-record-takes) states the five shapes and the treatment of each one. Four of the five name a file, and the report uses that table's own words for them.

**A shape is computed from the artifact and never declared.** No path of this population carries a shape in front matter. Most members are generated documents, which carry no front matter at all, and the rest are not documents. A path-to-shape table would be a hand-written statement of this population under a second name, and every hand statement of this population has been wrong. So the shape has the same source as the membership: the structure of the artifact, and the rule of its producer.

Three rules give the shape, and they are read in this order:

| The shape | How the verb computes it | The treatment |
|---|---|---|
| A line that depends on nothing | Every line of the file is one complete record, which is the shape `headwater capture` reads | `merge=union` |
| One record for each entity, in a fixed order | A generated file or a recorded corpus fixture whose opening states no fold. The recorded one is the artifact [HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) decomposed so that it merges | No merge attribute, because an ordinary conflict is the correct report |
| A fold over those records, or a fold over everything | A producer writes it, and for `headwater generate` the opening of the file states a count or a digest after the marker | `-merge` in `.gitattributes`, or `merge=headwater-regenerate` from the `info/attributes` of a configured clone |

**A generated file whose opening states no fold is one record per entity.** It lists one row for each document, and it takes no merge attribute. [#1058](https://github.com/headwater-ai/headwater/issues/1058) measured each generated file of this repository. Every one of them merged as text to the bytes that `headwater generate` writes over the merged tree. The verb reads the opening of the file rather than a list. So a generated file that starts to state a count is a fold again, and the verb reports it until it is declared.

**A fold takes either of two attributes, because git gives a clone the one that it can honor.** The committed `-merge` keeps the current side and conflicts in every clone ([#1058](https://github.com/headwater-ai/headwater/issues/1058)). `headwater init --git --git-config` writes the driver into the `info/attributes` of the clone, which wins. Both answers agree, so this verb exits 0 in CI and in a configured clone.

**The third rule covers two rows of the table, and the verb says both rather than guessing one.** Nothing in the structure of an artifact separates a fold over the records from a fold over everything. The separation costs nothing here, because the two rows take one merge attribute. A fold of either kind is derived rather than merged. The evaluation gives the two rows different cures rather than different attributes.

**The reported set is wider than the computed population.** The population holds what a producer writes. The shape report adds every path that has a merge attribute, and every decomposed recorded fixture. A row of the table with no member in the report stops being read in silence. Over this repository the population is the smaller of the two.

### A disagreement between a shape and a treatment

**A path whose merge attribute is not the treatment of its shape is a finding, and each one names what it costs.** The verb enumerates them in code rather than in prose, and the table below is the reader's copy:

| The shape | The attribute | What the merge does |
|---|---|---|
| A line that depends on nothing | None | Two branches that each appended a reading conflict, on every parallel append |
| A line that depends on nothing | `merge=headwater-regenerate` | No producer rewrites an append store, so the driver refuses the merge that `union` resolves correctly |
| One record for each entity | `merge=headwater-regenerate` | The driver refuses the merge the record was decomposed to take |
| One record for each entity | `merge=union` | Two record streams interleave out of the fixed order, into a file no producer writes |
| One record for each entity | `-merge` | The merge keeps one side of every record and conflicts, which refuses the merge the record was decomposed to take |
| A line that depends on nothing | `-merge` | Two branches that each appended a reading conflict, and the merge keeps the readings of one side alone |
| A fold | None | Two branches that move the fold to one value merge it in silence, into a value true of neither |
| A fold | `merge=union` | Two folds interleave into a value true of nothing, which is the worst of them and which nothing else of this repository reports |

**A merge driver that this verb does not know is reported by its name.** Git can give a path a driver such as `merge=ours`. The verb cannot say what that driver does with a shape. So the report names the path and the driver, and a path with a shape and such a driver is a disagreement. The verb does not read the driver as no attribute, because a driver that nothing reads looks the same as a path that agrees.

**Outside a git repository, a merge attribute that this verb cannot read is reported too.** The root-file reader reads `.gitattributes` as a list of paths. A pattern that carries a merge attribute and a glob character reaches files this reader cannot enumerate. The verb names such a line rather than passing over it. A nested `.gitattributes` is also a declaration that this reader does not read, and the report names it under a second heading. Both are disagreements. Inside a repository, git expands every pattern and reads every nested file, and neither finding occurs.

**The check layer does not carry this rule, and the reason is the grain.** A check of this repository runs at the grain of a document or of the corpus. [Spec 12](../spec/12-check-layer.md) gives a finding a document to hang on. Most paths here are not documents: a lock, a site page, a record of a test run, and a store of readings. A rule whose subject is a path that no shelf claims has no document grain to run at. So it lives in the verb that already computes the population.

**The report renders the palette [HW-DR-0045](../decisions/0045-coloring-the-cli-and-where-the-banner-goes.md) rules on, when standard output is a terminal.** The opening count is a heading. Each producer command carries the verb color, because it is the command a reader retypes. Every path carries the path color, on both sides of the answer. The two headings that report a disagreement carry the error color, and their agreement counterparts stay plain: the verb exits non-zero on exactly those two conditions, so the color says what the exit status says.

## Preconditions

Inside a git repository, the `git` executable must be on the `PATH`. The verb already needs it to read the ignore rules. A repository with no file that declares `-merge` or `merge=headwater-regenerate` reports every producer output as undeclared.

Outside a git repository, the root must hold a readable `.gitattributes`. A root without one reports every producer output as undeclared.

No producer has to run first. The verb reads what each producer last wrote, and it does not compare that against what a producer would write now. `headwater generate --check` and `headwater taxonomy resolve --check` are the verbs that ask the second question.

## Options

| Option | What it does |
|---|---|
| `--root <path>` | Select the repository whose tree is read. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. This report paints the opening count, each producer command, every path on both sides of the answer, and the two headings that report a disagreement. |
| `--no-banner` | Suppress the masthead. It is accepted here and does nothing, because only the root help screen prints one. |

The verb takes no option of its own. It computes one answer about one tree, and the tree is every input it has. `--format` and `--json` are not options of this verb.

## Exit status

**0** means that the computed set and the declared set agree, and that every reported path carries the merge attribute its shape takes.

**1** means that at least one of those disagrees. Five things give this status. A producer output carries no attribute. A declared path has no producer. A shape carries an attribute that is not its treatment. Outside a git repository, a merge attribute is one this verb cannot read. Inside one, git refused to give the merge attributes. The report prints the whole answer, on standard output, under either status.

## Environment

The verb reads no environment variable itself. The root comes from the command line, and every other input comes from the tree. Inside a git repository, git reads the configuration of the clone and its own environment, such as `GIT_DIR`. So `core.attributesFile` and `$GIT_DIR/info/attributes` can change the merge attribute of a path, as they change what a merge does.

## Files

| Path | How this verb treats it |
|---|---|
| `.gitattributes`, in any directory | Inside a git repository, git reads every one of them, with `$GIT_DIR/info/attributes` and `core.attributesFile`, and the verb takes the answer of `git check-attr`. Outside a repository, the verb reads the root file alone. A path has `merge=headwater-regenerate`, `merge=union`, no merge attribute, or a driver this verb does not know. The verb holds each answer against the shape of that path. |
| `.headwater/taxonomy.lock` | Claimed as the output of `headwater taxonomy resolve`. Its content is not read. |
| `.headwater/capture-cost.jsonl`, `.headwater/adoption.jsonl` | Read for their shape. No producer writes either one, and both carry `merge=union`. |
| Every other file of the tree | Read, to ask each producer rule whether the file is its own, and to compute the shape of the ones the report names. |

The verb writes no file. It changes no artifact and no cache.

## See also

[HW-DR-0049](../decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md) rules that a corpus-wide fold is derived and never stored, and it is the decision this verb computes the population of.

[`headwater generate`](headwater-generate.md) writes the largest of the four producer output sets, and `--check` holds every byte of it.

[*What a check can know*](../evaluations/what-a-check-can-know.md#the-shapes-a-record-takes) is where the five shapes and their treatments are stated. This verb reads that table and does not restate it.

[Spec 6](../spec/06-engine-architecture.md#cli) lists the command line.
