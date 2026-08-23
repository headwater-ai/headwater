---
id: HW-OBL-0127
status: draft
status_since: 2026-08-15
waiting_on: build
summary: "A path a change named that no census row holds keeps its prior version only where that version parsed, so a deleted document with malformed front matter reaches no rule."
last_verified: 2026-08-15
title: "A deletion is invisible where the version that stood there does not parse"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-check-layer
---

# A deletion is invisible where the version that stood there does not parse

## Context

`lifecycle.deletion.not_permitted` refuses a change that deletes a document standing at a terminal state of a regime that retains. The only account of what stood there is the version the caller named. The working tree holds no file at the path, and the census holds no row.

`Unbound::bind` decides what is left of such a path. Where the caller named a `prior` line and the bytes parsed as a document, the version is held. `Change::departed` then carries it to the rule. Where the bytes did not parse, `read_prior` has already recorded a reason. The bind replaces that reason with the arm that says only that no row holds the path. The reason is dropped, and the report counts the path among the ones that matched nothing.

That collapse is what makes the rule readable at all. Every source file a change edits reaches the same arm. A Rust file is named on a `prior` line, its bytes are no document, and it does not parse as one. A rule that skipped on an unreadable prior would skip on every real commit.

## Obligation

The corpus owes a reading that separates two files whose bytes did not parse. One is no document of any corpus. The other is a governed document whose committed front matter was malformed, and its deletion reaches no rule at all.

The engine states the trade in a doc comment on `Held::Unmatched`, and nowhere a reader of this corpus meets it. The argument there is that the case costs nothing. A document whose committed front matter does not parse fails the run that committed it. The argument is sound and it is not a measurement. It holds only where the commit that added the document ran the gate, and `git commit --no-verify` is one line.

[Spec 4](../spec/04-assurance-model.md#no-silent-passes-every-document-is-accounted-for) forbids a silent pass, and this one is silent in the strict sense. No instance is created over the path, so no skip is reported. The only line about it is a count of paths that matched nothing.

## Discharge

A reading that holds the two apart discharges this. `Held::Unmatched` would keep the reason beside the absent version. `Change` would publish the departed paths whose prior version did not read. The rule would then report a skip that names each one.

The cost is one arm and one report line. What that line would name today is every source file of a commit. So the discharge needs a second reading, which separates a document from a file, before the first one is worth writing.

What would not discharge it is a rule that refuses an unreadable prior. The engine reads the bytes a caller named, and it states no opinion about which of them are documents. A refusal there would make the honest wrapper the one thing this engine declines to accept.
