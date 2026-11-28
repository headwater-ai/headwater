---
id: HW-OBL-0127
status: discharged
status_since: 2026-11-29
waiting_on: build
summary: "The deletion of a governed document whose prior front matter did not parse reached no rule. The retention rule now skips its instance and names each such path, and it still refuses none."
last_verified: 2026-11-29
title: "A deletion is invisible where the version that stood there does not parse"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft+revise
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# A deletion is invisible where the version that stood there does not parse

## Context

`lifecycle.deletion.not_permitted` refuses a change that deletes a document at a terminal state of a regime that retains. The only account of that document is the version that the caller names. The working tree holds no file at the path, and the census holds no row.

`Unbound::bind` decides what is left of such a path. Where the caller names a `prior` line and the bytes parse as a document, the bind keeps the version. `Change::departed` then carries it to the rule. Where the bytes do not parse, `read_prior` records a reason. Until this change, the bind dropped that reason. The report counted the path among the paths that matched nothing, and no rule read it.

The record said that this gap cost nothing. Its reason was this: "a document whose committed front matter does not parse fails the run that committed it". The engine repeated that sentence in the rustdoc of `Change::departed`. A measurement on 2026-11-28 showed that it is false. A document with an unterminated quoted scalar in its front matter, in the printed fixture tree of the CLI, passed `headwater check --strict` with exit status 0. The census counted it as unreadable, and no finding named it. No rule reads an unreadable census row. So a malformed document can reach `main` through a green gate, and no `--no-verify` is necessary. Its deletion then reached no rule.

## Obligation

The corpus owed a reading that separates two files whose bytes do not parse. One file is not a document of any corpus. The other file is a governed document whose front matter was malformed, and its deletion reached no rule.

[The assurance model](../spec/assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids a silent pass, and this pass was silent. The instance passed, and the only line about the path was a count.

## Discharge

**`Change::unread` and version 2 of the retention rule discharge this record ([#1491](https://github.com/headwater-ai/headwater/issues/1491), the HW-OBL-0127 slice).**

- The bind keeps the reason for each departed path whose prior version did not read. It also keeps a flag that says whether the bytes held no block of front matter at all. `Change::unread` carries each such path, with the reason, the flag and the digest of the bytes where they opened.
- `Change::departed` does not change. Each entry there carries front matter, and `scope.ratchet` reads it.
- A corpus-scoped check that declares `NEEDS_PRIOR` receives the unread departures. The bytes of each one that may be a document go into the read set. So a warm run does not serve the verdict from before the change.
- The rule applies the second reading that this record asked for. A candidate passes three tests. It is a Markdown path by the census's own extension test. A shelf of the taxonomy claims it. Its bytes held a block of front matter. A source file fails the first test. A Markdown file outside every shelf fails the second. A shelf `README.md` with no front matter declares no state, so it fails the third.
- When the change has a refusal, the instance fails with the refusal, because the refusal already blocks the change. When it has no refusal and a candidate, the instance is skipped. The reason names each candidate path and why its prior version did not read. Otherwise, the instance passes.

The residual stays on the terms of this record. A departure whose prior version does not read is reported as a skip, and it is never refused. The engine reads the bytes that the caller names and states no opinion about the state of a document that it cannot read. A refusal there would block a deletion that the regime permits.

A document whose front matter does not load still passes a strict run while it stands in the corpus. That gap is a different defect, and this record does not close it.
