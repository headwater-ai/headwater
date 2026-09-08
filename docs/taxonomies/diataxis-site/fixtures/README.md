# Fixtures for the Diátaxis documentation-site bundle

The worked corpus contains one document of each concrete kind. The fixture case table is the contract for a runner: an isolated scratch root selects `diataxis-site`, validates and resolves; a second root selects it with `design-spec`, `decision-record`, and `diataxis`; each `headwater new` invocation succeeds; and a root that also copies this repository's overlay refuses at its first shared address.

| Case | Expected result |
|---|---|
| Isolated selection | `taxonomy validate` and `taxonomy resolve` exit 0 after the fixture overlay supplies five identifier namespaces |
| Entry selection with three library entries | resolution exits 0 |
| Constructor sweep | `headwater new tutorial`, `how_to`, `reference`, and `explanation` succeed |
| Overlay collision | resolution exits 1 and first names `purposes.procedure.intent` |
| Planted defect | a document under `docs/tutorials/` with `kind: reference` reports a kind or shelf finding |

The isolated run on 2026-09-09 validated and resolved successfully, then constructed all four kinds successfully. The collision run exited 1 and named four collisions: `purposes.procedure.intent`, `identifier_schemes.tutorial_id.pattern`, `kinds.tutorial.is_a`, and `shelves.tutorials.title`. The first one is the discriminator.

The corpus is intentionally a small worked instance until #489 or #491 provides a commit-pinned external source. Neither issue has landed a corpus in this branch, so no byte-identical source, source commit, source path, check count, or planted-defect result can honestly be recorded yet. That blocks criterion 4 admission; this entry is a proposal, not an admitted bundle.
