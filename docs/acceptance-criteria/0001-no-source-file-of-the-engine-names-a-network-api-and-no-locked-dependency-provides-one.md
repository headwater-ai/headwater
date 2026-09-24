---
id: HW-AC-0001
status: current
status_since: 2026-09-06
summary: "Two facts settle whether the engine can reach a network: no source file under engine/crates names a network API, and no locked package provides one."
last_verified: 2026-08-25
title: "No source file of the engine names a network API and no locked dependency provides one"
verification_method: inspection
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  verifies:
    - HW-REQ-0001
---

# No source file of the engine names a network API and no locked dependency provides one

## Fit criterion

Two facts, and both are true of the tree at the commit that carries this document. Neither is a standing guarantee. The method below is `inspection`, so it establishes the state of one tree at one commit. Nothing re-establishes it on the next commit, and a reader who needs the property today reads it again today.

**No source file outside `headwater-fetch` names one of six network APIs, except in two test files.** `engine/crates/fetch/` is excluded by name, because it is the one crate that carries a client ([HW-DR-0075](../decisions/0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md)). This command returns two lines, and both are in tests. `engine/crates/cli/tests/publish.rs` binds a `std::net::TcpListener` on `127.0.0.1` to serve an artifact to `taxonomy vendor`. `engine/crates/cli/tests/network_boundary.rs` names `ureq` as a package that no other crate may reach.

    grep -rn "std::net\|reqwest\|hyper\|ureq\|tokio::net\|TcpStream" engine/crates/ --include=*.rs --exclude-dir=fetch

Measured on 2026-09-24 on the branch of #959, which returned those 2 lines. On 2026-08-25 against `9a87b75`, the grep with no exclusion returned 0 lines. The six names are the standard library module, the three common client crates, the async transport, and the raw socket type.

**The six are the surfaces this criterion checks, and they are not every route to a socket.** A crate reaches one through `socket2`, through `mio`, through a `libc` call, or through `std::process::Command` running a program that does. This grep sees none of those four. Two things bound what that omission can hide. The second fact below reads the whole locked set by name, so a crate that arrives to supply one of those routes is visible there. And a workspace that took one of them would still have to name the crate that carries it. Widening the grep to those routes is work this criterion does not do, and it names the work rather than hiding it.

**No locked dependency that a crate of the checking loop reaches provides one.** `engine/Cargo.lock` names 223 packages, and 27 of them are crates of this workspace. Measured on 2026-09-24, on the branch of #959. The count was 192 and 26 before that change, and this criterion had stated 53 and 23 since 2026-08-25. #959 added 30 packages. The client is `ureq` and `ureq-proto`. The TLS stack is `rustls`, `rustls-webpki`, `rustls-pki-types`, `ring`, `untrusted`, `subtle`, `zeroize` and `webpki-roots`. The archive reader is `zip`, with `arbitrary` and `derive_arbitrary`. The others are `http`, `httparse`, `base64`, `percent-encoding`, `utf8-zero`, `getrandom`, `wasi` and ten `windows` shims. All of them are reached through `headwater-fetch`, and only `headwater-cli` reaches that crate. The other 166 were in the lock before, and none of them opens a socket. A lock file holds the whole transitive closure rather than the direct set. So a crate that arrives to open a socket is visible in it by name.

The criterion fails on either fact. A new line from the grep fails it. A new lock entry that a reader cannot place on the list above fails it too. So does a network package that a crate other than `headwater-cli` or `headwater-fetch` reaches.

## Method

Inspection, and the guidance of the facet is the reason: read the artifact against the fit criterion, and run nothing.

Two commands produce what a reader inspects, and neither is a test.

    grep -rn "std::net\|reqwest\|hyper\|ureq\|tokio::net\|TcpStream" engine/crates/ --include=*.rs --exclude-dir=fetch
    grep '^name = ' engine/Cargo.lock

The first produces the first fact. The second produces the second, as a list a person reads. The judgment is on that list, which is why the method is inspection rather than test. A person decides whether a package name opens a socket, and no command decides it.

One test holds a part of the second fact. `engine/crates/cli/tests/network_boundary.rs` reads the lock. It fails when a crate other than `headwater-cli` reaches `headwater-fetch`, `ureq`, `rustls`, `ring`, `webpki-roots` or `zip`. It does not read the names of the other packages, so the list above stays a judgment by inspection. A criterion whose method is `test` declares a `traces_to` edge onto a `code_path`, and this one does not. That is the route the requirement block of `.headwater/overlay.yml` records for the test member of 29148's four verifying artifacts.

No check rule holds either fact. Spec 4's control block binds a rule to an invariant through `mechanism: check:<rule-id>`. No rule of the 32 this engine ships reads a Rust source file or a lock file for a network API, so nothing here has one to name. A relation can reach one now: `verified_by` admits the anchor kind `check_rule`, and [HW-REQ-0002](../requirements/0002-every-document-of-a-kind-that-requires-sections-carries-all-of-them.md) is the requirement in this corpus that uses it.
