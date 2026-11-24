---
id: HW-IFACE-headwater-observe
status: current
status_since: 2026-11-24
summary: "The one writer of a verification entry in .headwater/observations.yml: it records the commit and the census digest of the one criterion that reaches the verification."
last_verified: 2026-11-24
title: "headwater observe"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5-5
  activity: draft
  evidence_basis: evidenced
relations:
  governs:
    - engine/crates/check/src/observation.rs
    - [engine/crates/cli/src/lib.rs, engine/crates/cli/src/main.rs]
---

# headwater observe

## Synopsis

    headwater observe <verification>... --commit <sha> [--root <path>] [--no-color] [--no-banner]

The verb takes one or more verification identifiers as operands, and the commit they passed at as `--commit`. Both are required.

## Description

`headwater observe` records that each named verification passed at one commit. For each identifier, it writes one `kind: verification` entry into `.headwater/observations.yml`, with three values: the verification, the commit, and `criterion_digest`. [`headwater check`](headwater-check.md) reads that entry. It shows the verification as `observed at <commit>` while the digest agrees with the criterion on disk, and as `suspect` after the criterion changes. [HW-DR-0073](../decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) ruling 3 states that comparison. The entry records no outcome, because that ruling gives the snapshot no outcome column.

**The digest comes from the census that `check` reads.** The verb loads the corpus in the same way as `check`. It finds the criterion through the relations whose `to` names `verification`, which is the same walk that the verification block of `check` makes. It records the census digest of that criterion: `sha256:` and the hexadecimal SHA-256 of the file bytes. So the reader accepts each digest that this verb writes, and a CI job does not calculate a digest of its own.

**An entry records one criterion.** The snapshot shape holds one `criterion_digest` in each entry. When two or more criteria reach a verification, no entry can state it, and the verb refuses that identifier with the names of the criteria. It also refuses a verification that no criterion reaches, because there is then no digest to record. This is a stated limit of the snapshot shape, and the verb does not invent a second shape.

**Every other byte of the file stays.** The verb replaces the entry of the same identifier, or it appends a new entry after the last one. A control entry, a comment and a blank line keep their bytes. The comment lines and blank lines after an entry belong to the next entry, so a replacement keeps them. When an entry already reads with the same commit and digest, the verb does not touch it. So a second run with the same input writes nothing, and the file stays byte-identical. The verb writes each value in double quotes, so that a commit of digits only reads as text.

**The verb does not ask version control about the commit.** It checks the syntax only: 4 to 40 hexadecimal digits, which is the test that `check` applies when it reads the entry. The commit is provenance. [HW-OBL-0199](../obligations/0199-an-observation-snapshot-records-a-commit-and-nothing-reads-it-back.md) records why no verb holds it against the history of the clone.

## Preconditions

**The corpus must load.** The verb reads the taxonomy lock and builds the census and the graph, in the same way as `check`. A corpus that `check` refuses, this verb refuses too, with the same message.

**A snapshot that is present must read.** When `.headwater/observations.yml` exists and does not parse, or its top level is not a mapping, or a key occurs two times, the verb writes nothing. It does not write over a file that it cannot read.

**A key that the verb replaces must start its line.** The verb replaces an entry by its lines. A snapshot written as one flow mapping, `{...}`, has no lines for one entry, and the verb refuses to replace an entry in it.

## Options

| Option | What it does |
|---|---|
| `<verification>` | The identifier of a verification that passed. Give one or more. An identifier given two times gets one entry. |
| `--commit <sha>` | The commit the verifications passed at, such as `$GITHUB_SHA` in a CI job. It must be 4 to 40 hexadecimal digits. The option is required. |
| `--root <path>` | The repository to read and write. It defaults to the working directory. |
| `--no-color` | Force plain text on both streams. The default senses whether each stream is a terminal, and shows color only there. |
| `--no-banner` | Suppress the masthead. It is accepted here and does nothing, since only the root help screen prints one. |

`--format` and `--json` are not options of this verb. Its one artifact is the snapshot file.

## Exit status

**0** means that the snapshot holds an entry for each identifier as asked. Standard output has one line for each entry, in the form `<verification> observed at <commit>, <criterion> at <digest>`. When no byte had to move, one line on standard error says so.

**1** means one of these:

- The command line named no identifier, or no `--commit`.
- The commit is not 4 to 40 hexadecimal digits. The verb writes nothing.
- The corpus did not load. The verb writes nothing.
- The snapshot is present and does not read, or a key it would replace does not start its line. The verb writes nothing.
- An identifier has no entry. It names no verification of the corpus, or no criterion reaches it, or two or more do. Or the census read no bytes of its criterion. The verb writes the entries of the other identifiers, and it names each refused identifier with the reason on standard error.

## Environment

No environment variable reaches this verb. The identifiers, the commit and the repository all come from the command line.

## Files

| Path | How this verb treats it |
|---|---|
| `.headwater/observations.yml` | Read, then written. The verb writes the new bytes to a temporary file beside it, and reads them back through the parser of `check`. Then it renames the file over the old one. A failed write leaves the old file as it was. |
| The corpus under `--root` | Read, through the census and the graph. Never written. |
| `.headwater/taxonomy.lock` | Read. Never written. |
| `.headwater/imports/` | Read where `.headwater/taxonomy.yml` declares an import, for the anchors that an imported snapshot supplies. An `imports` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |
| the path each `harvests.<name>.at` names | Read where `.headwater/taxonomy.yml` declares a pinned corpus export, for the anchors that export supplies. A `harvests` entry that does not read stops the verb with exit 1, and so does an `at` path outside the repository root. |

The verb touches no cache and no claim store.

## See also

[`headwater check`](headwater-check.md) reads the entries that this verb writes, and its row for `.headwater/observations.yml` states the shape of each entry.

[Record a verification from CI](../how-to/record-a-verification-from-ci.md) shows a pytest or JUnit step that calls this verb.

[HW-DR-0073](../decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) states what a verification is, and when one is `suspect`.
