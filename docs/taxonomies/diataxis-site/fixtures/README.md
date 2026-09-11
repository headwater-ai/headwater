# Fixtures for the Diátaxis documentation-site bundle

The worked corpus contains one document of each concrete kind. The fixture case table is the contract for a runner: an isolated scratch root selects `diataxis-site`, validates and resolves; a second root selects it with `design-spec`, `decision-record`, and `diataxis`; each `headwater new` invocation succeeds; and a root that also copies this repository's overlay refuses at its first shared address.

| Case | Expected result |
|---|---|
| Isolated selection | `taxonomy validate` and `taxonomy resolve` exit 0 after the fixture overlay supplies five identifier namespaces |
| Entry selection with three library entries | resolution exits 0 |
| Constructor sweep | `headwater new tutorial`, `how_to`, `reference`, and `explanation` succeed |
| Overlay collision | resolution exits 1 and first names `purposes.procedure.intent` |
| Planted defect | a document under `docs/tutorials/` with `kind: reference` reports a kind or shelf finding |
| n8n skills corpus, vendored | `fixtures/n8n/corpus/.agents/skills/` holds every file the pin holds under `.agents/skills/`, each body byte-identical below an added front-matter block |
| n8n skills corpus, typed | `headwater check --root` over the assembled scratch root types every one of those files, reports 0 untyped, and prints typed and excluded denominators that sum to the file count under the root |

The isolated run on 2026-09-09 validated and resolved successfully, then constructed all four kinds successfully. The collision run exited 1 and named four collisions: `purposes.procedure.intent`, `identifier_schemes.tutorial_id.pattern`, `kinds.tutorial.is_a`, and `shelves.tutorials.title`. The first one is the discriminator.

The fixture vendors byte-identical source files beside the worked corpus. `sysl` commit `d34f35389ac019db7c37300ac62c2306bfa766b2` supplies `docs/docs/tutorial.md` for the tutorial, `docs/docs/best-practices/intro.md` for the how-to guide, and `docs/docs/lang-spec.md` for reference. `cockroach` commit `8812064a015d2faf99d3fc7e15880f94042954b0` supplies `docs/tech-notes/life_of_a_query.md` for explanation. Their SHA-256 digests are recorded in the paths under `fixtures/sources/`; no source body was edited.

The four short documents under `fixtures/corpus/` are executable shape fixtures rather than the external corpus. Their front matter and section contracts make the constructor and resolver arms run without altering a vendor source body. A future fixture runner must assemble each selected source with front matter only and run the same case table. The clause this sentence used to carry, that the denominators must be recorded *before this entry can be admitted*, is stale: `docs/taxonomies/README.md` already lists this entry as admitted, and [the n8n corpus](n8n/README.md) records the denominators now. A denominator is owed and admission does not wait on it.

[`n8n/`](n8n/README.md) holds the real corpus the last two rows of the table name: the 35 files of `.agents/skills/` at `b0550cb3cb4d1752546a69056c55eccfb9111a12`, all 35 typed as `how_to` on one homogeneous shelf, with 0 untyped and 0 excluded.

The planted input is `kind: reference` on `docs/tutorials/get-started.md`. The check run exits 1 and reports `shelf.placement_is_primary`: the `tutorials` shelf carries `tutorial`, and the document restates it as `kind: reference`. The rule treats an explicit `kind` field as a conflicting restatement on every homogeneous shelf.
