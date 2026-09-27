---
id: HW-DR-0092
status: current
status_since: 2026-09-27
summary: "The per-identifier citation comment of spec 5 is a convention for an adopter's corpus. By the owner's ruling, the code of this repository carries none. Its comments that name an identifier are prose, and the checker does not read them."
last_verified: 2026-09-27
title: "The citation comment of spec 5 is for an adopter's corpus, and this repository does not practice it on its own code"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  governs:
    - to: tools/cite/check-citations-fixtures.sh
      verified_revision: sha256:d2733ba32d5f1032c7d76c694dbd95e3f9a9d9398612035c8a71e43dccc292c8
  traces_to:
    - HW-SPEC-ai-integration
---

# The citation comment of spec 5 is for an adopter's corpus, and this repository does not practice it on its own code

## Context

**Spec 5 states a convention for generated code.** The section [Generated artifacts cite what licensed them](../spec/05-ai-integration.md#generated-artifacts-cite-what-licensed-them) asks an agent to write a comment of the shape `# per <IDENTIFIER> (<path>)`. The comment goes above the line that the document licensed. The section did not say whose corpus the convention is for. So a reader could take it as a rule for every corpus, this repository included.

**This repository has a checker for the convention and no population for it.** `tools/cite/check-citations.py` reads each comment of that shape and asks the engine whether the identifier resolves and whether the document governs the file. Its CI step runs a fixture suite, `tools/cite/check-citations-fixtures.sh`, over a scratch copy of this corpus with planted citations. [#844](https://github.com/headwater-ai/headwater/issues/844) found that no code in this repository writes the comment, and that no file that an agent reads tells an agent to write one.

**The issue offered two outcomes.** One outcome was to practice the convention here: tell each agent to write the comment, and run the checker over this repository's own code. The other outcome was to state that the convention is for an adopter's corpus, and to record the measurement.

**The owner ruled on the issue.** The [ruling](https://github.com/headwater-ai/headwater/issues/844#issuecomment-5852797610) of 2026-09-27, in run `20260927-0443`, reads: "**adopter-only**. The spec 5 citation-comment convention is for an adopter's corpus; this repository does not practice it on its own code. Scope the issue to recording that ruling where spec 5 and the corpus meet it."

**The measurement, at commit `2590cc90`.** Each number below has its command, so that a later reader can run it again and does not measure from nothing.

| what was counted | result | how |
|---|---|---|
| lines of the citation shape outside `tools/cite/` | 1, in the whole tree | `grep -rEn "(//\|#) per [A-Z][A-Z-]*-?[0-9]+ \("`, with `target`, `.git` and worktrees excluded |
| where that one line is | `docs/spec/05-ai-integration.md` line 285 | the worked example of spec 5, which names an imagined corpus |
| citations in this repository's own code | 0 citations in 365 files, exit status 0 | the checker over `engine/crates/*/src`, `engine/crates/*/tests`, `.claude/hooks`, `.claude/agents`, `.claude/skills`, `.claude/commands`, `.githooks` and `.github` |
| comment lines that name an `HW-` identifier in any shape | 460 lines in 184 files | `grep -rEn` for a comment marker and `HW-[A-Z]+-[0-9]{4}` on one line, over `engine/crates`, `.claude/hooks`, `tools` and `.githooks`, with each `fixtures` and `target` directory excluded |
| instruction files for an agent that mention the convention | 0 | `CLAUDE.md` and each file under `.claude/agents`, `.claude/skills` and `.claude/commands` |

The population of the checker is `engine/crates` by crate, not whole. `engine/crates/census/fixtures/walk/broken-link.md` is a dangling link on purpose, and the checker exits 2 on it. The population is also not `tools/` whole, because of the planted fixtures, and not `.claude` whole, because `.claude/worktrees/` holds other checkouts.

## Decision

**The citation comment of spec 5 is a convention for an adopter's corpus.** An adopter who wants an answer to "why does the code do this?" writes the comment, and runs the checker over their own code.

**This repository does not practice the convention on its own code.** No instruction file for an agent tells an agent to write the comment, and no CI step runs the checker over this repository's own code. The CI step for the checker stays the fixture suite. Its case over the real spec 5 pins what this repository carries: one line of the shape, at line 285, which does not resolve.

**The comments that name an identifier are prose, and they are not a partial adoption.** The code of this repository names decisions and obligations in its comments, most often as a Markdown link. The checker reads only the `per <IDENTIFIER> (<path>)` shape, so it reads none of these. They tell a reader where to find the reason. They make no claim that the checker can hold, and nothing in this record asks for one.

**Spec 5 states the scope where its reader meets it.** The section on the convention says that it is for an adopter's corpus, and it links this record. A case in the fixture suite holds that the section names this record.

## Consequences

**#844 closes on this record.** Its clause about the instruction in the files for an agent does not apply. Its clause about a population that is not empty in CI does not apply either. The checker is not for this repository's own code. The spec 13 record that the product owner proposed for the issue is not written, because this record settles the question.

**The checker stays in `tools/cite/`.** [tools/README.md](../../tools/README.md) already says that the checker can move out as a companion that an adopter runs.

**Two events reopen the question.** The first event is that the checker moves out of `tools/` as a companion for an adopter. Then this repository can be its first user. The second event is that an adopter asks for this repository as a worked example of the convention. The fixture case over spec 5 fails when the scope paragraph stops linking this record. So the person who reopens the question reads this record first.

**The measurement can go stale.** The count of comment lines that name an identifier changes with each change to the code, and nothing holds it. The count of citations in this repository's own code is 0, and a citation that someone adds to that code does not fail a check. The case over spec 5 holds only that spec 5 states the scope.
