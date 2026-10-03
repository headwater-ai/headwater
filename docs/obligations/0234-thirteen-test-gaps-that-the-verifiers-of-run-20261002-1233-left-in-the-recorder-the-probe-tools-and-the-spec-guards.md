---
id: HW-OBL-0234
status: current
status_since: 2026-10-03
summary: "Verifiers of run 20261002-1233 left thirteen surviving mutants and untested branches. They sit in the recorder, the egress proxy, the probe suites and the spec guard tests."
last_verified: 2026-10-03
title: "Thirteen test gaps that the verifiers of run 20261002-1233 left in the recorder, the probe tools and the spec guards"
waiting_on: build
provenance:
  warrant: asserted
---

# Thirteen test gaps that the verifiers of run 20261002-1233 left in the recorder, the probe tools and the spec guards

## Context

The verifiers of run `20261002-1233` reported mutants that survive and branches that no case reaches. Each names a reader inside this repository: the next writer of a transcript, the next campaign, or the next change to a spec guard. So the product owner ruled each one RECORD and not an issue. Intake lines 33, 37, 39, 44, 50 and 121 were ruled during the run. Lines 79, 86, 97, 104, 108, 109 and 132 were ruled at the top of run `20261003-1026`. This is one of three spec 13 records for that run, as `hw-run-policy` caps them. Each item was checked on `a46bd95e` by the file and the symbol it names. A mutant was not re-run.

## Obligation

**The recorder and the probe tools.**

- `confines` in `tools/probe/ci-confine.sh` probes `bwrap` with `--unshare-user` alone. `tools/probe/probe-record.sh` also needs `--unshare-net`. A runner that makes a user namespace and no network namespace runs the recorder suite and fails its cases, where it should skip with one line.
- Mutant V10 survives `tools/probe/probe-record-fixtures.sh`. A transcript that gains "The session had no access to GitHub." keeps the suite green, because the `absent` checks match only "reachable" and "could not read".
- Mutant R6 survives: a killed driver under an ancestor that sets `PR_SET_CHILD_SUBREAPER`. The script the verifier used is `v2-subreaper.py` in its scratch directory.
- The committed transcripts record an empty result for every tool call. So "reached GitHub" by `gh api`, `curl` and `WebFetch` is inferred from the calls that followed, and no case shows it. No live case tests that `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` leaves no telemetry host and no update host.
- The guard against an absolute path in `tools/probe/probe-transform.sh` has no case, and its mutant survives.
- `tools/probe/egress-proxy.py` builds its TLS context with `ssl.create_default_context`. The suite cannot hold the hostname check alone, because a self-signed certificate fails the chain first, and a mismatch case needs a locally trusted authority. A mutant that drops a bare CR from the control-character check survives, though the branch refuses a bare CR. A folded sole `Content-Length` line is forwarded unchecked.
- With a dirty tree, the staggered block of `tools/probe/probe-record-fixtures.sh` skips, and the suite still exits 0. So an uncommitted mutation reads as green.
- The per-shape mutants S3 to S6 of #1552 come from its build note and were not re-run. A wiring case in `engine/crates/cli/tests/wiring.rs` and a case in `tools/probe/probe-record-fixtures.sh` still declare one probe.

**The guard tests of spec 6, spec 7 and the check.**

- `engine/crates/cli/tests/spec_six_inbound_links.rs` does not flag an uppercase anchor such as `#CI-adapters`, a bare link to spec 6 or a `#taxonomy-validate` link.
- `no_comment_quotes_moved_cli_text_as_spec_6` in `engine/crates/cli/tests/spec_six_cli_rules.rs` misses a block comment, a credit by bare path, a quote in curly quotes, single quotes or backticks, "Section 6 of the spec says", and a credit after its quote.
- `engine/crates/cli/tests/spec_six_overview.rs` passed three paraphrases after slice 4b of #1572. A wording that says the change-scoped mode is unimplemented passes the negation list. A reworded contradiction 5 that adds a claim passes. A reworded copy of the old Checks sentence is held by the word bar alone. Slice 4d added cases for G1 to G5 and two paraphrase shapes, and no verifier has re-run these three against it.
- The non-claims count test in `engine/crates/generate/tests/spec_seven_export.rs` reads number words, from "one" to "ten", and not digits. So "5 non-claims" in a current decision passes.
- After a Sonnet verify of #1631, no mutant reached the branches "another document declares" and "not a scalar" of `unwritable` in `engine/crates/check/src/suspect.rs`. The Markdown and SARIF render of the pairs that name no edge are untested, and only the CI `interface_page` test holds the JSON version bump from 1.5 to 1.6.

## Discharge

Each item discharges alone, when a case kills the mutant or reaches the branch it names, or when a change records why the gap stays. The record discharges when every item has. An item that gains a reader outside this repository leaves this record for an issue, and the record says where it went.
