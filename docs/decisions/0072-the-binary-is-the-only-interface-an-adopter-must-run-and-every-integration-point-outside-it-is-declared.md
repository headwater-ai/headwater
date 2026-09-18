---
id: HW-DR-0072
status: current
status_since: 2026-09-17
summary: "An adopter reaches the whole governed loop through the binary. Two integration points sit outside it and the list is closed against growth. Git plumbing meets the necessity test, a network fetch is held open, and the change manifest fails it and is a defect."
last_verified: 2026-09-17
title: "The binary is the only interface an adopter must run, and every integration point outside it is declared"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
---

# The binary is the only interface an adopter must run, and every integration point outside it is declared

## Context

An adopter installs this engine to govern a corpus that is not this one. What that adopter has to run has never been written down. Everybody assumed the answer was `headwater` and nothing else. Three measurements show that the tree does not hold the assumption.

**No verb writes a change manifest.** `headwater check --change` reads one, and [the contract](../interfaces/headwater-check.md) names `.githooks/change-manifest` as "the producer this repository uses". That shell script is the only producer that exists, and it belongs to this repository. A run without the flag states the cost. At `e4a63da1`, 729 rule instances of this corpus report `change-scoped-only`, which means that no rule reads the prior version. The report calls them skipped, so an absence reads as a posture.

**`headwater init` scaffolds a consumer declaration and an overlay, and nothing else.** The merge driver that protects a derived artifact needs a `git config` line and an executable in the adopter tree. [HW-DR-0049](0049-a-corpus-wide-fold-is-derived-and-never-stored.md) states that an adopter inherits the rule rather than the artifacts, and [#892](https://github.com/headwater-ai/headwater/issues/892) asks whether that answer still holds.

**The front door instructs a script.** `README.md` and step 3 of [the tutorial](../tutorials/your-first-governed-corpus.md) both name `tools/headwater-bootstrap.sh`, and the tutorial states that the script is not a part of this engine. Here the binary already reaches the same result, because `headwater taxonomy vendor` takes the path of a fetched artifact and an expected digest.

No document states the boundary. The eleven principles in [spec 0](../spec/00-vision-and-scope.md#design-principles) do not name it. Principle 8 says that the system governs itself, which is why this repository carries scripts at all, and it draws no line for an adopter. [The command surface](../interfaces/README.md) indexes the verbs and never claims that they are the whole surface. So the rule was a shared assumption, and an assumption that nothing holds is one that prose drifts past.

## Decision

**An adopter runs the binary for every operation of the governed loop.** The loop is what reads the corpus, checks it, writes it and resolves its taxonomy. A corpus owner reaches all of it through `headwater`, and this repository proposes no script to an adopter for any part of it.

**Two integration points sit outside the binary. This list is closed against growth.**

- **Git plumbing.** A merge driver, a merge attribute and a hook are things git runs. Git does not take an executable from a repository without the consent of the clone, so the engine cannot install one. [#892](https://github.com/headwater-ai/headwater/issues/892) rules on what ships inside this edge.
- **A network fetch.** `headwater taxonomy vendor` takes a path and never a location, because no crate of this engine opens a socket. This entry is the weaker of the two, and it is held open rather than settled. The no-socket property serves the checking loop, which has to answer the same way on every run and on a machine with no network. A bootstrap verb runs at setup and never inside that loop, so that reason does not reach it. The digest in `--expect` is what makes a fetch safe, whoever performs it. [#932](https://github.com/headwater-ai/headwater/issues/932) rules on it. A ruling that the verb may take a location removes this entry and leaves one.

**The test for the edge is necessity and never convenience.** An integration point is legal here only where the binary cannot do the work without the loss of a property the corpus depends on. Git plumbing meets the test, because consent of the clone is not a property this engine can grant itself. The network fetch has not met it yet. The entry above cites a property of the engine rather than a reason that a bootstrap verb must carry it. A point that fails this test is a defect, and a missing verb is a gap to file rather than an exception to grant.

**The change manifest fails the test.** Nothing stops a verb from writing one, and spec 12 already fixes the two anchors it needs. [#929](https://github.com/headwater-ai/headwater/issues/929) carries it as a defect of the command surface.

**This list grows by a decision record and never by a script.** A contributor who reaches for a script in adopter-facing prose either finds the verb or files the gap.

## Consequences

Those instances stay out of reach until [#929](https://github.com/headwater-ai/headwater/issues/929) lands. Naming the boundary does not close the hole, and the count is the measure of what an adopter loses in the meantime.

[#892](https://github.com/headwater-ai/headwater/issues/892) gains a frame it did not have. Its question was whether merge-safety tooling ships at all. The question now is narrower. The edge is legal, and consent of the clone is the constraint on it. What remains is what ships inside it.

[#930](https://github.com/headwater-ai/headwater/issues/930) follows from the same rule. The bootstrap script is a convenience over a verb that already exists, and adopter-facing prose presents it as the only route. That is worth correcting whatever [#932](https://github.com/headwater-ai/headwater/issues/932) rules, because an adopter on a mirror or an air-gapped host needs the path form either way.

The list is closed against growth and not against a correction. [#932](https://github.com/headwater-ai/headwater/issues/932) can remove the second entry, and a list of one is the stronger form of this ruling rather than a retreat from it.

Nothing checks any of this. No rule reads adopter-facing prose for a script invocation, and the two entries above live in this record alone. So a later contributor can add a third edge in silence, and the only guard is that this record is short enough to read. [#933](https://github.com/headwater-ai/headwater/issues/933) weighs what should assert the boundary, including the answer that nothing should.

This record rules on what an adopter runs. It rules nothing about what this repository runs for itself. Principle 8 keeps `tools/` and `.githooks/` exactly as they are, because a corpus that governs itself needs producers that no adopter ever sees.
