---
id: HW-AC-0001
status: draft
status_since: 2026-08-25
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

Two facts, and both are true of the tree at the commit that carries this document.

**No source file names a network API.** This command returns no line.

    grep -rn "std::net\|reqwest\|hyper\|ureq\|tokio::net\|TcpStream" engine/crates/ --include=*.rs

Measured on 2026-08-25 against `9a87b75`, which returned 0 lines. The six names are the transport surfaces a Rust crate reaches a socket through. They are the standard library module, the three common client crates, the async transport, and the raw socket type.

**No locked dependency provides one.** `engine/Cargo.lock` names 53 packages, and 23 of them are crates of this workspace. The other 30 are `clap` and its `anstyle` family, `pulldown-cmark`, `saphyr-parser`, `thiserror` and `unicase`. The rest are `memchr`, `bitflags`, `arraydeque`, `strsim`, `heck`, the two `windows` shims, and the proc-macro crates that the two derive macros need. None of the 30 opens a socket. None of them has a transitive dependency that does either, because a lock file holds the whole transitive closure rather than the direct set.

The criterion fails on either fact. A new line from the grep fails it, and so does a new lock entry that a reader cannot place on the list above.

## Method

Inspection, and the guidance of the facet is the reason: read the artifact against the fit criterion, and run nothing.

Two commands produce what a reader inspects, and neither is a test.

    grep -rn "std::net\|reqwest\|hyper\|ureq\|tokio::net\|TcpStream" engine/crates/ --include=*.rs
    grep '^name = ' engine/Cargo.lock

The first produces the first fact. The second produces the second, as a list a person reads. The judgment is on that list, which is why the method is inspection rather than test. A person decides whether a package name opens a socket, and no command decides it.

No test in this workspace holds either fact, so this criterion declares no `traces_to` edge onto a `code_path`. A criterion whose method is `test` declares one instead. That is the route the requirement block of `.headwater/overlay.yml` records for the test member of 29148's four verifying artifacts.

No check rule holds either fact. Spec 4's control block binds a rule to an invariant through `mechanism: check:<rule-id>`. No rule of the 26 this engine ships reads a Rust source file or a lock file for a network API. No relation here could reach such a rule if it did. [#411](https://github.com/headwater-ai/headwater/issues/411) holds that gap.
