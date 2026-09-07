---
id: HW-DR-0014
title: Q14 — Discovery surface
status: current
status_since: 2026-08-11
last_verified: 2026-08-11
summary: The corpus descriptor is a generated projection at `.headwater/corpus.json`, and registration went to Q16.
provenance:
  warrant: accepted
  agency: mixed
  drafted_by: claude-opus-5
  activity: draft+revise
  accepted_by: j.baxter
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-EVAL-the-serving-boundary
---

# Q14 — Discovery surface

## Context

One [evaluation](../evaluations/the-serving-boundary.md) settles this with [Q17](0017-governed-access-and-the-solution-layer.md) and [Q7](0007-scope-of-the-mcp-surface.md), because the three describe one boundary from three sides.

**The open-question entry bundled two questions, and only one of them is ours.** It asked how a machine "discovers that a corpus exists, what taxonomy governs it, what version, and where to start to read". **Registration** is how a machine learns of a corpus when it holds no pointer at all. No file inside a corpus answers that, and none ever has. Every convention in the field presumes a client that already resolved a name. Three package ecosystems put a capability document inside an index and discover the index in none of them. Registration went whole to [Q16](0016-public-presence.md), which closes it on a definition: registration is publication into a channel whose reader is already obliged. **Resolution** is what closes here: a machine holds a location and learns what governs it.

## Decision

The **corpus descriptor** is a projection ([spec 7](../spec/07-distribution-and-federation.md#arriving-at-a-corpus-cold)). `generate --check` holds it to regeneration. It names every corpus root in the repository, with the taxonomy identity, the version, the lock hash, the entry points, and each export profile. One descriptor serves several transports. A file at a fixed path *plus* a separate MCP statement is two copies of one fact ([principle 2](../spec/00-vision-and-scope.md#design-principles)).

## Consequences

**Its path is the engine's and not the taxonomy's, and the entry did not notice that it had to be.** Every other projection takes its output path from the schema. Apply that rule here and the descriptor is unreachable, because a reader who must read the taxonomy to find it already knows what it says. So the descriptor is engine-defined and non-optional, at `.headwater/corpus.json` relative to the repository root. The [register projection](../spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition) already holds that standing for a different reason.

**The canonical location is inside the repository, and that is a ruling.** A reserved path at the root of an origin fixes one service to one site. It also misdescribes a host that serves several publishers, and it needs control of the apex. A corpus meets all three objections, so a pointer reaches the served copy instead.

**Three rules make it usable.** A version carries a stated client behavior, because a version with no rule attached is a string. A success response that does not parse to the declared shape means **absent** rather than malformed. The reason is that a host which answers every path with a default page is the ordinary case. And a filter reaches the descriptor first, which is the real dependency that this decision had on Q17.

**The descriptor is a disclosure, not only a convenience.** It names roots, entry points and profiles, which is organizational structure. The robots convention states the same thing about itself in its own standard. The file grants no authorization, and a path becomes discoverable by being named.

**What the prior art contributes, including against us.** Four conventions that resemble this keep the index bare and put identity on each collection. They do so to avoid a central file that describes roots which somebody else edits. Headwater centralizes anyway, and may, because the descriptor is generated and regeneration catches drift. The sharper warning is `llms.txt`: about 137,000 domains measured, 97% of valid files unread in a month, and no provider obliged to read one. A descriptor is worth what its obliged consumer is worth, and Headwater's first consumer is its own tooling ([HW-EVAL-adjacent-work §O](../evaluations/adjacent-work.md#o--the-serving-boundary-descriptors-redaction-and-the-write-path)).
