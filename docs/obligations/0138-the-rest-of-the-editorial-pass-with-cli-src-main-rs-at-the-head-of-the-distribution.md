---
id: HW-OBL-0138
status: current
status_since: 2026-09-06
summary: "The editorial pass measured 180,255 words of comment prose and rewrote two files, leaving cli/src/main.rs, the largest, untouched."
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: draft
  accepted_by: j.baxter
  evidence_basis: evidenced
last_verified: 2026-08-26
title: "The rest of the editorial pass, with cli/src/main.rs at the head of the distribution"
waiting_on: adopter
---

# The rest of the editorial pass, with cli/src/main.rs at the head of the distribution

## Context

Issue #191 measured a habit in the engine's comment prose: sentences that say what a thing is not before they say what it is.

It tracked the habit with six phrases: `rather than`, `is what`, `and it is`, `and never`, `which is what`, and `it is not`.

#191 ruled, in [Q24](../decisions/0024-q24-readability-and-what-a-sweep-can-be-asked-about.md), that no mechanism performs this pass.

Readability is not a sweep class, it gets no verb, and the work falls to a person using the `ste-editor` skill.

Q24 states the bar the pass must clear: fewer words, same claims.

A first hand pass rewrote the module headers of `engine/crates/check/src/scope.rs` and `engine/crates/check/src/cache.rs`, plus their five worst blocks each.

That pass moved `scope.rs`'s header from a mean of 27.1 words to 19.5, and its longest sentence from 70 words to 41.

## Obligation

The corpus owes an editorial pass over 180,255 words of comment prose across `engine/**/*.rs`, counted by the same six-phrase tally #191 used.

`cli/src/main.rs` carries the largest concentration, 111 tics over 8,343 words, and it is the file #191 named first.

Nine more files carry the rest of the count the first pass did not reach.

- `check/tests/fixtures.rs`: 51 tics, 3,505 words.
- `query/src/mcp.rs`: 47 tics, 3,538 words.
- `check/src/scope.rs`: 45 tics remain beneath its already-passed header, over 5,079 words.
- `check/src/register.rs`: 44 tics, 3,720 words.
- `audit/src/lib.rs`: 43 tics, 3,056 words.
- `generate/src/lib.rs`: 43 tics, 3,055 words.
- `check/src/lib.rs`: 38 tics, 3,195 words, the fourth file #191 named.
- `scaffold/src/lib.rs`: 37 tics, 3,056 words.
- `census/src/census.rs`: 35 tics, 3,134 words.

The pass owes every citation and every reason in the rewritten prose surviving intact, checked by reading the diff rather than by a count.

It owes doctests and any fixture that quotes passed prose left unbroken.

`cargo test` runs doctests, and a moved comment needs a deliberate re-bless naming the moved bytes.

It owes not turning a single tic into a rule, a `retired_terms` entry, or a threshold.

`rather than` is correct English, and the count only locates the work.

## Discharge

Discharge is a pass over `cli/src/main.rs`, with a before-and-after measurement stated in the change description rather than claimed.

It also needs `check/src/lib.rs` passed, the fourth file #191 named.

Both passes must leave `cargo test`, `cargo fmt --check`, and `cargo clippy --all-targets -- -D warnings` green, along with the five gates.

`headwater_check::fragment::comment_links` must still hold every link in the rewritten prose.

No tic construction may gain a `retired_terms` entry, a new rule, or a threshold.

This obligation waits on an adopter, since no mechanism performs the pass and no verb reports it complete.
