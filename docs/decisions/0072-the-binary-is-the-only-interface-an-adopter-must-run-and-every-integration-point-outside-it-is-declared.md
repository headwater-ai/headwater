---
id: HW-DR-0072
status: current
status_since: 2026-09-17
summary: "An adopter reaches the whole governed loop through the binary. One integration point sits outside it and the list is closed against growth. Git plumbing meets the necessity test, a network fetch left the list under HW-DR-0075, and a verb writes the change manifest."
last_verified: 2026-09-24
title: "The binary is the only interface an adopter must run, and every integration point outside it is declared"
provenance:
  warrant: accepted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  governs:
    - README.md
    - tools/headwater-bootstrap.sh
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

**One integration point sits outside the binary. This list is closed against growth.**

- **Git plumbing.** A merge driver, a merge attribute and a hook are things git runs. Git does not take an executable from a repository without the consent of the clone, so the engine cannot install one. [#892](https://github.com/headwater-ai/headwater/issues/892) rules on what ships inside this edge.

**A network fetch left this list.** `headwater taxonomy vendor` took a path and never a location, on the ground that no crate of this engine opened a socket. [HW-DR-0075](0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md) rules that the no-socket property serves the checking loop alone, that a bootstrap verb runs outside that loop, and that the digest in `--expect` already makes a fetch safe whoever performs it. `vendor` may accept a location once a distributor lands it, confined to a crate only the CLI links, so this is a correction of the entry rather than an exception granted to it.

**The test for the edge is necessity and never convenience.** An integration point is legal here only where the binary cannot do the work without the loss of a property the corpus depends on. Git plumbing meets the test, because consent of the clone is not a property this engine can grant itself. A point that fails this test is a defect, and a missing verb is a gap to file rather than an exception to grant.

**The change manifest is inside the binary.** `headwater change` writes the manifest that `headwater check --change` reads, from the two anchors that spec 12 fixes. So the manifest is not an integration point outside the binary, and it has no entry in this list. [#929](https://github.com/headwater-ai/headwater/issues/929) is the issue that built the verb.

**This list grows by a decision record and never by a script.** A contributor who reaches for a script in adopter-facing prose either finds the verb or files the gap.

## Consequences

An adopter who runs `headwater check --change` gets the manifest from `headwater change`. No script stands between the adopter and a change-scoped check. [.githooks/change-manifest](../../.githooks/change-manifest) only hands its two arguments to that verb, for the hooks of this repository.

[#892](https://github.com/headwater-ai/headwater/issues/892) gains a frame it did not have. Its question was whether merge-safety tooling ships at all. The question now is narrower. The edge is legal, and consent of the clone is the constraint on it. What remains is what ships inside it.

[#930](https://github.com/headwater-ai/headwater/issues/930) landed too. `README.md` and the tutorial now name the binary route first, and call the bootstrap script a convenience over it rather than the only route. Naming a location directly in `vendor`, once a distributor lands it under [HW-DR-0075](0075-the-vendor-verb-may-take-a-location-and-the-fetch-lives-only-in-a-crate-the-checking-loop-never-links.md), still leaves the path form worth documenting: an adopter on a mirror or an air-gapped host needs it either way.

The list is closed against growth and not against a correction. HW-DR-0075 removed the second entry, and a list of one is the stronger form of this ruling rather than a retreat from it.

[#933](https://github.com/headwater-ai/headwater/issues/933) answered what should check this record's one remaining claim. A corpus-wide rule was rejected: the list holds one necessity-based entry, so a rule that reads every document for a stock lexical pattern would report zero findings by construction, the failure shape a lexical rule already takes when its targets run out. `.claude/tutorial/adopter_interface.py` checks the narrower, real risk instead: it reads `README.md` and the tutorial for a script this repository ships standing in for a verb, the shape both #929 and #930 took, and it names the bootstrap script's network fetch as the one declared exception. `.claude/tutorial/fixtures.sh` runs it in CI, so a second edge added in silence now fails a check rather than waiting on this record staying short enough to read.

This record rules on what an adopter runs. It rules nothing about what this repository runs for itself. Principle 8 keeps `tools/` and `.githooks/` exactly as they are, because a corpus that governs itself needs producers that no adopter ever sees.
