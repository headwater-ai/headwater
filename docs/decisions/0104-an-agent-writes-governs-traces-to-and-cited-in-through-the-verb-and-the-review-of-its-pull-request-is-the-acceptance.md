---
id: HW-DR-0104
status: current
status_since: 2026-10-02
summary: "`headwater new --relates` writes `governs`, `traces_to` and `cited_in` and refuses a target that binds to nothing. A reviewed pull request accepts each edge"
last_verified: 2026-10-02
title: "An agent writes governs, traces_to and cited_in through the verb, and the review of its pull request is the acceptance"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: draft
  evidence_basis: evidenced
relations:
  supersedes:
    - HW-DR-0083
  governs:
    - taxonomy-source/headwater-standard/taxonomy.yml
    - .headwater/overlay.yml
    - engine/crates/scaffold/src/lib.rs
  traces_to:
    - HW-OBL-0105
    - HW-OBL-0226
    - HW-SPEC-taxonomy-model
    - HW-EVAL-default-taxonomy-first-run
---

# An agent writes governs, traces_to and cited_in through the verb, and the review of its pull request is the acceptance

## Context

[HW-DR-0083](0083-governs-and-traces-to-are-created-by-an-agent-because-a-session-proposes-the-line-and-a-person-types-it.md) put `created_by: agent` on `governs` and `traces_to`. It ruled that a session proposes the line and a person accepts it by typing the line. So `headwater new --relates` refused every relation whose `created_by` was not `scaffold`, and every `governs` and `traces_to` edge of this corpus was hand entry.

On 2026-10-01 the owner ruled on [#1315](https://github.com/headwater-ai/headwater/issues/1315), in run `20261001-1107`: "Amend: PR review is acceptance". An agent may write a `governs` or `traces_to` edge in a pull request. The merge ruling on that pull request is the acceptance by a person.

On 2026-10-02 the owner widened [#1560](https://github.com/headwater-ai/headwater/issues/1560). The request: "We need to review HW-DR-0083 as I think we can allow agents to fill in these edges - they should be clear." The request came from [#1580](https://github.com/headwater-ai/headwater/issues/1580), where an agent drafts a requirement, a criterion and a verification. The verb wrote `verified_by` and `proven_by` of that chain, and it refused `governs`, `traces_to` and `cited_in`. The overlay declares `cited_in` with `created_by: agent`, from a verification to a `test_site` anchor.

[Spec 3](../spec/03-authoring-and-lifecycle.md) says that a changed decision gets a successor and an error about the present gets an edit in place. Here the decision changed, and the title of HW-DR-0083 states the old rule. So this record supersedes it.

## Decision

**The actor stays `agent` on all three relations.** [Spec 2](../spec/02-taxonomy-model.md#who-creates-each-edge) defines `created_by` as the actor that pays for the edge. For `governs`, `traces_to` and `cited_in`, that payment is a judgment about which file or which document. An agent still makes the judgment. The verb is the pen it writes with, and it decides no target. The value `scaffold` would claim that the taxonomy decides the edge, and it does not.

**Who writes each edge, and what accepts it:**

| relation | actor (`created_by`) | writer | acceptance |
|---|---|---|---|
| `governs` | `agent` | `headwater new --relates`, the `new` tool of `headwater mcp`, or a hand edit | the merge ruling on a reviewed pull request |
| `traces_to` | `agent` | `headwater new --relates`, the `new` tool of `headwater mcp`, or a hand edit | the merge ruling on a reviewed pull request |
| `cited_in` | `agent` | `headwater new --relates`, the `new` tool of `headwater mcp`, or a hand edit | the merge ruling on a reviewed pull request |

**An edge that an agent writes gets to `main` only through a reviewed pull request.** The acceptance is the merge ruling, and not a field. The verb writes no `accepted_by` on an edge or on a document, so [HW-OBL-0108](../obligations/0108-an-agent-writes-the-acceptance-stamp-of-every-document-in-this-corpus.md) and [HW-DR-0034](0034-q34-whether-acceptance-means-merged-to-main-and-what-an-agent-may-write-before-that.md) do not change.

**The verb's gate reads the actor and not the relation name.** `headwater new --relates` writes a relation whose `created_by` is `scaffold` or `agent`. It refuses `author`, `generator`, `hook` and `import`, because the verb is none of those actors. So the gate also opens `constrains` and `conflicts_with`, the other two `agent` relations of the base package. That agrees with the ruling, because each of them also gets to `main` only through a reviewed pull request. A gate on the relation name would be a second statement of who pays for an edge, beside the declaration.

**The verb binds an anchor target the way `headwater check` binds it.** Where the far end of a relation admits an anchor kind, the verb binds the target through `headwater_graph::edges::bind` and the resolvers of the check. A `governs` pattern that matches no entry is refused with `AnchorUnresolved`, and the verb writes nothing. That is the rule of [HW-DR-0074](0074-a-code-path-anchor-is-a-pattern-over-the-tree-and-it-binds-when-the-pattern-matches-at-least-one-entry.md), applied at write time, and no second definition of a tracked entry exists.

**`cited_in` has one stated gap.** The `comment-scan` resolver binds a file only where a comment in that file cites the identifier of the document that asserts the edge. The verb mints that identifier in the same run, so no file can cite it yet. So for an anchor kind whose resolver is `comment-scan`, the verb asks `source-tree` whether the path names one regular file. A path that names one file is written, and the report names the comment that the file still owes. `comment-scan` opens one named file and expands no pattern. So a pattern, a directory and a path that reaches nothing are refused, like a `governs` pattern that matches no entry. `headwater check` reports the edge as unbound until the comment is there.

**No declared value changes, so no release of `headwater-standard` is owed.** HW-DR-0083 made 4.7.0 a minor release because `headwater taxonomy audit` moved two relations to another row. Here no `created_by` value moves, no row of the audit moves, and a consumer's audit report is the same. The change is in the engine, and a package release follows a declaration change.

## Consequences

The verb now writes relations that cover 937 of 1144 declared edge halves of this corpus, against 64 before. The figures are from `headwater taxonomy audit` over 562 documents on 2026-10-02, with this record in the corpus. The `agent` row holds 873 of the 1144 halves and the `scaffold` row holds 64. No declaration changes, so no row of the audit moves to another actor.

The case table of the scaffolder holds the new behavior. A `governs` pattern that matches no entry is refused with `AnchorUnresolved`, and so is a `cited_in` path that reaches no file. A `governs` pattern that matches an entry is written, and a `cited_in` path that reaches a file is written with the owed comment named. The row where `traces_to` declares `created_by: hook` stays refused, so the gate did not open to every actor.

The section "Edges that a person types" of [HW-OBL-0226](../obligations/0226-six-corpus-gaps-from-run-20260929-1205-filed-together.md) rests on the premise that an agent may not write these edges. That premise is gone. The two missing `traces_to` edges that it names belong to their own issues, and this record does not write them.

[HW-OBL-0105](../obligations/0105-nothing-plays-the-hook-role-that-two-relations-name.md) stays discharged. The verb plays the role of the pen for an agent, and no relation declares `hook`.

HW-DR-0083 said that a verb that writes these edges, with no person between the proposal and the file, reopens its question. This record names that person: the reviewer of the pull request that carries the edge.

A path that puts an agent-written edge on `main` without a reviewed pull request reopens this decision. An example is a run that merges its own pull request with no merge ruling, or a verb that writes into `main` directly.
