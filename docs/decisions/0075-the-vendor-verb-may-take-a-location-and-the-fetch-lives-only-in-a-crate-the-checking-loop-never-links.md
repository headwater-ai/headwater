---
id: HW-DR-0075
status: current
status_since: 2026-09-24
summary: "`headwater taxonomy vendor` may accept a location, ruled yes with a boundary. The fetch lives in a crate only the CLI links, so the checking loop's crates keep the no-socket property that serves it."
last_verified: 2026-09-24
title: "The vendor verb may take a location, and the fetch lives only in a crate the checking loop never links"
relations:
  constrains:
    - HW-DR-0072
    - HW-DR-0022
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-sonnet-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# The vendor verb may take a location, and the fetch lives only in a crate the checking loop never links

## Context

[HW-DR-0072](0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md) names a network fetch as the weaker of two integration points an adopter reaches outside the binary, and holds it open rather than settled. The entry rests on one property: "no crate of this engine opens a socket." [#932](https://github.com/headwater-ai/headwater/issues/932) was filed against that entry itself, because the record cites the property rather than arguing that a bootstrap verb must inherit it. It also asks whether `headwater taxonomy vendor` may take a location as well as a path.

The property serves the checking loop. [Spec 0](../spec/00-vision-and-scope.md#non-negotiables) states the actual non-negotiable at line 117: "No network dependency at check time." `headwater check` has to answer the same way on every run and on a machine with no network. That is because a corpus owner may run it disconnected and expects one verdict. `vendor` runs once, at setup or at an upgrade, never inside that loop. The sentences that generalize the checking-loop property to the whole engine are broader than what spec 0 requires. That gap is what #932 was filed to name.

`--expect` already carries the digest that would make a fetch safe. A fetch the engine performs and a fetch `tools/headwater-bootstrap.sh` performs are checked by the same comparison against the same pinned value. So the safety property does not distinguish who dials out.

An exhaustive grep for the property's wording, run against this branch, finds it stated in ten places across seven files, not the three #932 names. The grep finds it at `engine/crates/probe/src/lib.rs:95`, `engine/crates/import/src/lib.rs:86` and `:88`, `engine/crates/import/src/snapshot.rs:27`, `engine/crates/resolve/src/package.rs:32` and `:3490`, `engine/crates/resolve/src/release.rs:17` and `:20`, and `engine/crates/verbs/src/lib.rs:339`. It also finds it in doc comments on four functions in `engine/crates/cli/src/main.rs`: `vendor`, `taxonomy diff`, `probe plan` and `probe grade`. The `vendor` implementation itself lives in `engine/crates/resolve/` (`package.rs`, `release.rs`), which `cli` and `verbs` dispatch to. `probe` and `import` do not implement `vendor` at all. Their sentences restate an engine-wide claim rather than state a fact their own crate depends on.

## Decision

**A bootstrap verb may reach the network. `headwater taxonomy vendor` may accept a location as well as a path, once a distributor lands it.** The determinism argument that grounds the no-socket property does not reach a verb that runs outside the checking loop. The digest already pins what a fetch, by any hand, is checked against.

**The capability is confined to a crate only the CLI binary links.** `vendor`'s existing contract, that it checks bytes it is handed against a pinned digest, stays exactly what `engine/crates/resolve/` does. That crate keeps taking a path and stays socket-free. A location on the CLI's `vendor` invocation resolves through a new crate that `engine/crates/cli/` alone depends on. That crate fetches to a local path and hands the result to `resolve` unchanged. `probe` and `import` call neither `resolve`'s vendor path nor the new crate. So their sentences describe a fact that stays true of their own crate without correction. This is the option the issue names as a separate crate that only the CLI links. It is preferred over a location on `vendor` with no such boundary, because only this shape leaves the library crates' own claims true. The other shape requires a rewrite of an unrelated crate's documentation, for a change that crate has no part in.

**This ruling settles denotation, not the build.** Whoever lands the fetch makes an engineering choice about the new crate's HTTP and TLS dependency. It can compile into every CLI build by default, or only under a feature a distributor opts into. That person weighs the choice against the cost that the issue's case against names. That cost is a dependency that falls on every build of the CLI, which today has none. That choice does not change which crates may claim to be socket-free, so it is out of scope for this record.

**Implementing the fetch is out of scope for this record and for #932.** This is a ruling-and-correction change: it corrects [HW-DR-0072](0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md)'s second entry and the sentences that overstate the engine-wide property. A new dependency, a new crate and a CLI grammar change are a materially larger, separately reviewable surface, and land as [#959](https://github.com/headwater-ai/headwater/issues/959) instead. That issue moves [the `vendor` grammar row](../interfaces/headwater-taxonomy.md) and the CLI grammar block spec 6 carries, alongside the crate and the code. Neither moves here, because `vendor`'s contract takes only a path until that issue lands.

**HW-DR-0072's closed list falls to one entry.** The network-fetch bullet leaves the list of integration points an adopter reaches outside the binary. Git plumbing is what remains. A list of one is the stronger form of that ruling, as HW-DR-0072 itself already anticipated.

## Consequences

**HW-DR-0072 is edited to remove its network-fetch bullet and to cross-reference this record.** The edit does not reword the bullet with a narrower reason, because the ruling is yes.

**The ten sentences the grep found are corrected in the same change**, not the three #932 names. A broader sweep for the same claim in the same run found and corrected several more the named grep did not reach. `probe/src/lib.rs`, `import/src/lib.rs` and `import/snapshot.rs` keep stating that their own crate opens no socket, because that stays true. Only the claim that generalizes to "no crate of this engine" is removed or narrowed to the crates it still describes. `resolve/package.rs` and `resolve/release.rs` are corrected the same way: the crate that implements `vendor` today stays socket-free. Its doc comments say so without claiming the property for crates it does not constrain. `verbs/lib.rs:339`'s `vendor` description is corrected, and so are the doc comments and `#[arg(help = …)]` strings in `cli/src/lib.rs` and `cli/src/main.rs`. The corrected text states that a location is a future capability confined to a crate the CLI alone links. It does not state that no crate of the engine ever will. **Some of what moved is real `--help` and `--about` text, confirmed live against the built binary, not only doc comments.** `cli/src/lib.rs`'s `#[arg(help = …)]` strings reach a caller directly, and `verbs/lib.rs`'s `description` field reaches one through `.about(word.description)`. `docs/reviews/the-sixty-four-restored-help-strings-checked-against-the-binary.md` rows 27 and 60 quote the pre-edit wording of exactly the two strings this change touches (the `diff --to` help and the `vendor` description). That document is a point-in-time record by this repository's own convention, and it stays as written. Nothing mechanically breaks, because no fixture asserts any of the 64 strings byte-for-byte outside three unrelated cases in `wiring.rs`. From this commit on, however, a reader who wants the current wording reads the source, not that record.

**The probe harness's argument is checked rather than assumed to survive.** `probe/src/lib.rs:95` lists "No socket" as one of the properties the harness's own claims rest on. That property is unchanged for `probe` itself. The harness never calls `vendor`, `resolve`, or the future fetch crate, so its argument holds under this ruling without amendment.

**[#930](https://github.com/headwater-ai/headwater/issues/930), the documentation half, is unaffected by this record alone.** It can still close the moment `vendor` itself accepts a location. Until then, adopter-facing prose points to the path form, on a proxy, a mirror or an air-gapped host as much as anywhere else. This follows HW-DR-0072's own note that a location is additive and never a replacement for the path form.

**[#959](https://github.com/headwater-ai/headwater/issues/959), the follow-up issue that implements the fetch, inherits this record's boundary as a constraint, not as a suggestion.** A `resolve`, `probe` or `import` crate that grows a network dependency to satisfy that issue is the defect this record was written to prevent. A reviewer of that issue checks the crate graph against it.

**[HW-DR-0022](0022-q22-the-integrity-posture-of-a-published-package.md) is narrowed by this record, from #959 on.** Its sentence "The engine never fetches" held while `vendor` took a path alone. Now the crates of the checking loop never fetch, and `headwater-fetch` fetches for `taxonomy vendor <location>` alone. The integrity posture of that record does not change, because the digest check is the same for either form.

**#959 decided the build question that this record left open: the fetch is a default-on cargo feature, `fetch`, on `headwater-cli`.** The 0.2 bar is adoption from the binary alone, so the released binary fetches, and `release.yml` builds it with the default features. `--no-default-features` is the opt-out for a distributor, and that binary refuses a location with one line that names the path form. The crate is `headwater-fetch`. `engine/crates/cli/tests/network_boundary.rs` reads `engine/Cargo.lock` and fails when any crate other than `headwater-cli` reaches it or its client.
