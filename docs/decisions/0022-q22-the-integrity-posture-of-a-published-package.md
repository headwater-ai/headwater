---
id: HW-DR-0022
status: current
status_since: 2026-08-14
summary: A package digest checks a fetched artifact against a pin that a person committed, and it is never a signature. The in-house SHA-256 is held against a second implementation rather than replaced by a dependency.
last_verified: 2026-08-14
title: "Q22 — The integrity posture of a published package"
provenance:
  warrant: proposed
  agency: mixed
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-SPEC-distribution-and-federation
---

# Q22 — The integrity posture of a published package

## Context

[Spec 7](../spec/07-distribution-and-federation.md#publishing) states that a release carries an integrity digest, and that the engine requires only that it can fetch a version and check that digest. Until [#77](https://github.com/headwater-ai/headwater/issues/77) nothing published a package and nothing checked one, so the sentence named a mechanism that did not exist.

**The engine already writes digests, and the module that writes them refused to let this one be inherited.** `engine/crates/hash` implements SHA-256 in about eighty lines rather than taking a dependency with six transitive crates behind it. Its own argument names the limit: every digest this engine writes is an integrity mark inside a repository that already holds the bytes it was computed from. The module then stated that the package digest of a fetched artifact is a different job, and that it should reach for a vetted implementation.

**Two questions were open and only one of them was being asked.** The question in front of the reader was which implementation of SHA-256 to use. The question that decides it is what the digest is a check against, because a digest with no answer to the second question is a number that proves nothing whichever library computed it.

## Decision

**A package digest is an integrity check against a pin that a person committed, and this repository does not claim it is a signature.**

The consumer writes the publisher's digest into `.headwater/taxonomy.yml`, which is authored, committed and read in a diff. `headwater taxonomy vendor` recomputes the digest from the bytes on disk and refuses an artifact that does not match the pin. It also refuses to run when no pin exists, rather than recording the artifact in front of it. A digest that the engine copied out of the thing it had just received would be a pin against itself.

**The engine never fetches, and the verb is what enforces that.** `vendor` takes the path of a directory that the caller already has. [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) forbids a network dependency at check time, no crate of this engine depends on the network, and a verb that takes a path has no code path that could open a socket. The fetch belongs to whatever moves a directory in the organization that runs it.

**The in-house SHA-256 is reused, and a second implementation is what makes that safe to do.** A vetted crate buys one thing here, which is a correct implementation. It buys no authentication at all, because the strength of the check above comes from a pin that a reviewer read rather than from the authorship of the compression function. So the only risk a dependency answers is an implementation defect, and a defect is answerable by measurement. `engine/crates/hash/tests/oracle.rs` holds this implementation against `sha256sum` over sixteen subjects, and the continuous integration job makes the absence of that second implementation a failure rather than a skip.

## Consequences

**What the check now refuses, measured on 2026-08-14.** A file changed inside a published artifact, a file added that the record does not name, a file the record names that is gone, and a whole artifact republished with a record that agrees with itself. The first three are named file by file in the message. The fourth is the case a record cannot catch on its own, and the pin is what refuses it. `engine/crates/resolve/tests/publish.rs` provokes each one, and each of seven regressions of the implementation was caught by the test named for it.

**What it does not refuse, and the record that says so.** The first fetch, which is the one that produced the pin. Nothing here authenticates a publisher, and closing that gap needs a key, a channel that distributes the key, and a rule for revocation. None of the three is decided, so the gap is [HW-OBL-0115](../obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md) rather than a field named `signature` that holds a digest.

**A sentence in the engine is falsified rather than left standing.** `engine/crates/hash/src/lib.rs` and `engine/README.md` both said that this digest should reach for a vetted implementation. Both now state the argument above and cite this record, because a reader who meets the old sentence would conclude that the reuse was an oversight.

**The count of third-party crates in the engine stays at six.** That number is a claim the engine makes about itself in two files, and a seventh crate would have moved it for a property that no test could have shown.
