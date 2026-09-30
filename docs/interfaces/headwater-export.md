---
id: HW-IFACE-headwater-export
status: current
status_since: 2026-09-06
summary: "How to emit a taxonomy export profile to a file or standard output with a declared loss census."
last_verified: 2026-08-25
title: "headwater export"
relations:
  governs:
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
    - engine/crates/generate/src/export.rs
---

# headwater export

## Synopsis

    headwater export [--profile <name>] [--format <target>] [--at <date>] [--check] [--json] [--root <path>]

The command emits declared export profiles, or one selected emitter target to standard output.

## Description

Without `--format`, it writes every declared profile to its taxonomy-declared path. `--check` compares those files without writing. With `--format`, it writes one artifact to standard output and reports its projection census to standard error. With `--format json`, that artifact is the graph export, and [The graph export, member by member](#the-graph-export-member-by-member) names each of its members.

The command refuses an ambiguous profile selection or an uncovered emitter loss.

**This command is the one writer of an export that declares `committed: false`.** Such an export is built at publish time, and the tree does not hold it. Without `--check`, the command writes it like every other declared export. With `--check`, it does not require the file and does not compare a copy that it finds. The report names the path with a line that is not an error. [`headwater generate`](headwater-generate.md) does not write the file.

**`--at` dates an export that declares `committed: false`, and only that declared export.** No gate compares such a file, so a date inside it fails nothing. Its marker states that `headwater export` builds it at publish time and that no gate compares it. A committed export carries no date, because `generate --check` compares it by byte. So when the selected profiles include a committed export, the command refuses `--at` before it writes anything. The refusal names each committed path. Name a target with `--format`, or name with `--profile` a profile whose declared exports all state `committed: false`.

**With `--check`, the command reads the producer identity before it compares the bytes of any declared export.** A declared export was written by an emitter set, in the way every other projection was. So this command and [`headwater generate --check`](headwater-generate.md) make the same read of the corpus descriptor at `.headwater/corpus.json`. When the emitter set it records is not this engine's, a byte difference has two possible causes. A corpus moved, or an emitter moved, and the command cannot tell which. It reports both numbers, it says nothing about a remedy, and it exits non-zero. It says this before the drift sentence, which asserts what a run in that state does not know.

The read is of the descriptor and never of an export, so it holds whether or not the selected profile writes anything. A repository that commits no descriptor records no emitter set, and a descriptor that records none is one an earlier engine wrote. Absence is not disagreement in either case, and the command then behaves as it always did.

## Preconditions

The repository must load its taxonomy, corpus and projection declarations. A stream export must select one profile when several are declared. A declared export must have a writable or checkable target path.

## Options

| Option | What it does |
|---|---|
| `--profile <name>` | Selects one declared export profile. |
| `--format <target>` | Emits one artifact to standard output using the target. |
| `--at <date>` | Adds a generation date to a stream artifact, and to a declared export that states `committed: false`. The command refuses it with `--check`, and it refuses a run that selects a committed export. |
| `--check` | Checks declared output files without writing, and first checks the emitter set that the committed corpus descriptor records against this engine's. |
| `--json` | Selects JSON output where the command supports a format choice. |
| `--root <path>` | Selects the repository to load. |
| `--no-color` | Force plain text on both streams: bold and dim weight plus glyphs, no escape sequence. The default already senses whether each stream is a terminal, and renders color only there. |
| `--no-banner` | Suppress the masthead: the line naming this binary and its version, that the root help screen alone prints. It is accepted here and does nothing, since only the root screen prints one. |

## Exit status

**0** means that declared outputs match, or that the stream artifact was emitted without an uncovered loss.

**1** means that loading, profile selection, emission or writing failed. It also means that `--at` selected a committed export, that `--check` found drift, or that `--check` found a producer difference. The two `--check` failures print different sentences, because only one of them has a remedy this command can name.

**A refusal writes nothing to standard output, and a run that reported and then failed still wrote its report.** Drift found by `export --check` is the second case, where the regeneration report prints and the run then exits 1. A refusal of the command line, of a profile name or of an emitter target prints nothing there at all. The account of a refusal is one English sentence on standard error, under `--json` and `--format json` alike, which is what [HW-DR-0043](../decisions/0043-q43-whether-a-refusal-under-json-is-a-json-document.md) rules.

**A refusal of this verb names the spelling of the target that the caller typed.** `--json` and `--format json` reach one value, so the two write one artifact byte for byte. A message a person reads is not an artifact. A message that named the other flag would send a reader to a flag nobody typed.

**1**, and never 101, when standard output or standard error cannot be written, and one sentence on standard error names a failed standard output.

## Environment

The command reads no environment variable.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/taxonomy.lock`, corpus and projections | Read to build the export. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. A pin with no digest binds no anchor. An absent file binds no anchor, and neither does a file that fails the pinned digest or is not an export. |
| `.headwater/corpus.json` | Read under `--check` for the emitter set it records, before any declared export is compared. This verb never writes it. |
| Declared export paths | Written without `--format` and `--check`, and an export that declares `committed: false` too. Under `--at`, only an export that declares `committed: false` is written, and it states the date. Under `--check`, an export that declares `committed: false` is not read. |
| Standard output | Receives a stream artifact with `--format`. |
| Standard error | Receives the stream projection census. |

### The graph export, member by member

`headwater export --format json` writes the graph export as one JSON object to standard output. The tree holds no copy of it. This table names every member that the object can hold, and a test holds the table to what the engine writes. A path uses `.` between a key and its parent, and `[]` for each element of an array. The table describes the `json` target.

| Member | When it is present | What it means |
|---|---|---|
| `headwater:generated` | Always | The marker that says an engine wrote this artifact. |
| `version` | Always | The version of this format, as `Major.Minor`. Read this member. [Raising the version](#raising-the-version) says when it moves. |
| `export_version` | Always | The same value as `version`, under its earlier name. |
| `profile` | Always | The export profile that this artifact is. |
| `profile.name` | Always | The name of the profile. |
| `profile.target` | Always | The emitter target, which is `json` here. |
| `profile.filtered` | Always | `true` when the profile declares a filter. A reader of a filtered artifact does not take an absence as a fact about the corpus. |
| `profile.tombstone` | When `profile.filtered` is `true` | The tombstone grain that the profile declares: `counted` or `sealed`. |
| `profile.generated_at` | With `--at` only | The date that `--at` supplied. The engine never reads a clock for it. |
| `loss_set` | Always | One element for each loss that the target declares. The `json` target carries the graph with no loss, so this array is empty. |
| `loss_set[].class` | In each loss | What the loss is about: `node`, `edge` or `attribute`. |
| `loss_set[].name` | In each loss | The name of what the target drops. |
| `loss_set[].reason` | In each loss | The declared reason for the loss. |
| `census` | Always | The projection census, which compares what the artifact carries with what the corpus holds. |
| `census.accounted for` | Always | One element for each group of omitted items that a declared reason covers. The key contains a space. |
| `census.accounted for[].class` | In each element | `node`, `edge` or `attribute`. |
| `census.accounted for[].name` | In each element | The name of the omitted group. |
| `census.accounted for[].count` | In each element | How many items the reason covers, as a number. |
| `census.accounted for[].reason` | In each element | The reason that covers the omission, for example the filter rule that withheld a document. |
| `census.unaccounted for` | Always | One element for each omitted item that no declared reason covers. The key contains a space. An element is a defect in the emitter, and the command then exits 1. |
| `census.unaccounted for[].class` | In each element | `node`, `edge` or `attribute`. |
| `census.unaccounted for[].name` | In each element | The name of the omitted item. |
| `census.unaccounted for[].at` | In each element | Where the omitted item is. |
| `tombstones` | When the profile is filtered and its grain is `counted` | One element for each filter rule that withheld at least one document. |
| `tombstones[].rule` | In each element | The filter rule. |
| `tombstones[].documents` | In each element | How many documents the rule withheld, as a number. |
| `graph` | Always | The property graph. |
| `graph.documents` | Always | One element for each document that the profile carries. |
| `graph.documents[].path` | Always | The path of the document from the repository root. |
| `graph.documents[].kind` | Always | The kind of the document. |
| `graph.documents[].id` | When the document has an identifier | The identifier of the document. |
| `graph.documents[].warrant` | When the document has a warrant | The warrant, repeated here for a reader that reads no taxonomy. |
| `graph.documents[].facets` | Always | The facets as the document wrote them. The keys come from the corpus, so this table does not name them. |
| `graph.anchors` | Always | One element for each anchor, such as a code path that a document governs. |
| `graph.anchors[].anchor_kind` | Always | The anchor kind. |
| `graph.anchors[].id` | Always | The normalized identity of the anchor. For an anchor that holds more than one pattern, it is a key and not a path. |
| `graph.anchors[].resolver` | Always | The resolver that bound the anchor. |
| `graph.anchors[].patterns` | When the anchor holds more than one pattern | The sorted, normalized patterns. Read this member for the files of a list anchor, and do not decode `id`. |
| `graph.anchors[].excluded_by` | When a corpus exclusion claims a single literal pattern | The exclusion pattern that claims the anchor. |
| `graph.edges` | Always | One element for each edge that the profile carries. |
| `graph.edges[].source` | Always | The path of the source document. |
| `graph.edges[].relation` | Always | The relation that the edge resolved to. |
| `graph.edges[].written_as` | Always | The relation name as the author wrote it. It can be the inverse of `relation`. |
| `graph.edges[].target` | Always | The target of the edge. The members it holds depend on `bound`. |
| `graph.edges[].target.bound` | Always | `document`, `anchor`, `withheld` or `nothing`. [What bound selects](#what-bound-selects) gives the members of each value. |
| `graph.edges[].target.id` | When `bound` is `document` or `anchor` | The identifier of the document, or the identity of the anchor. |
| `graph.edges[].target.path` | When `bound` is `document` | The path of the target document. |
| `graph.edges[].target.kind` | When `bound` is `document` | The kind of the target document. |
| `graph.edges[].target.anchor_kind` | When `bound` is `anchor` or `withheld` | The anchor kind. |
| `graph.edges[].target.resolver` | When `bound` is `anchor` | The resolver that bound the anchor. |
| `graph.edges[].target.patterns` | When `bound` is `anchor` and the anchor holds more than one pattern | The sorted, normalized patterns, as on the anchor. |
| `graph.edges[].target.rule` | When `bound` is `withheld` | The export profile whose filter withheld the target. |
| `graph.edges[].target.reason` | When `bound` is `nothing` | Why nothing bound the target, as one English sentence. Do not parse it. |
| `graph.edges[].attributes` | When the edge has instance attributes | The attributes as the author wrote them. The keys come from the corpus, so this table does not name them. |

#### What bound selects

| `bound` | Members of `target` | What it means |
|---|---|---|
| `document` | `id`, `path`, `kind` | A typed document is the target. |
| `anchor` | `anchor_kind`, `id`, `resolver`, and `patterns` for more than one pattern | An anchor is the target. |
| `withheld` | `anchor_kind`, `rule` | The source withheld the target under a declared export filter. This is not a defect. |
| `nothing` | `reason` | Nothing bound the target. The edge travels, so that a reader does not conclude that every relation resolved. |

### Raising the version

The current version is `1.2`. `version` and `export_version` carry this one value, from one constant in the engine. Read `version`. `export_version` stays because its removal is a major change, and that change costs every reader more than the duplicate costs.

These rules decide how a change to the members moves the version. This section is the one statement of them.

- **A member added moves the minor.** A member that the export writes in more cases than before also moves the minor, because a reader already handles it when it is present.
- **A member removed or renamed moves the major.** A change to the type or the meaning of a member also moves the major. So does a member that becomes conditional where it was always present, because a reader that relied on it then fails.
- **A reader that meets a major above the one it understands stops with a message.** A reader that meets a different minor warns and continues.

The history: `1.0` to `1.1` added `version` ([#343](https://github.com/headwater-ai/headwater/issues/343)). `1.1` to `1.2` added `patterns` to a list anchor and to an edge onto one ([#1247](https://github.com/headwater-ai/headwater/issues/1247)).

## See also

[`headwater generate`](headwater-generate.md) writes native projections. [`headwater taxonomy publish`](headwater-taxonomy.md) creates a package artifact.
