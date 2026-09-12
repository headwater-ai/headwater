# Fixtures for the Diátaxis documentation-site bundle — the n8n corpus

The 35 files of `.agents/skills/` from [`n8n-io/n8n`](https://github.com/n8n-io/n8n), typed against this entry. Nothing here is invented. The [other fixtures of this entry](../README.md) hold four short documents written to exercise the four kinds, which is what criterion 4 calls a *realistic* corpus. This one is what criterion 4 calls a *real* one.

This is the third corpus of the same repository at the same pin. The [design-spec fixture](../../../design-spec/fixtures/n8n/README.md) types n8n's architecture documents and the [standards-spec fixture](../../../standards-spec/fixtures/n8n/README.md) types its review rules. This one types its skills. Three trees rather than one wider tree, for the reason those two give: `corpus.root` is a single scalar string, and one root cannot reach `packages/` and `.agents/` at once. This corpus and the review-rules corpus share the root `.agents` and differ only in the shelf the overlay adds.

Every number below came from the run this file quotes. Nothing here is predicted.

## Modification notice

**These 35 files are modified copies of n8n software.** The modification is the addition of Headwater front-matter keys, and nothing else. **The body of every copy, from the first byte after the front-matter block to the last byte, is identical to the pinned upstream file.** No word, no heading, no link, no spelling, no contraction and no line break was changed.

n8n's [Sustainable Use License](LICENSE.md) requires a prominent notice on a modified copy, and this section is it. The same license requires that anyone who receives any part of the software also receives a copy of the terms, so [`LICENSE.md`](LICENSE.md) beside this file is a verbatim copy of n8n's own, and not a link to it. It is a third copy rather than a link to either earlier fixture's, because a third set of copied files is a third receipt of part of the software.

## The pin

| | |
|---|---|
| Upstream | https://github.com/n8n-io/n8n |
| Branch | `master`. n8n's license states that content of branches other than `master` is not licensed, so every citation here names `master` and nothing else |
| Commit | `b0550cb3cb4d1752546a69056c55eccfb9111a12`, committed 2026-09-01T16:27:59Z |

**None of the 35 is under the Enterprise License.** n8n excludes a file with `.ee.` in its name and a file under a directory with `.ee` in its name from the Sustainable Use License. No path under `.agents/skills/` at the pin matches `.ee` in either form.

## What the tree holds

`.agents/skills/` at the pin holds **22 skill directories, 34 files inside them, and one top-level `AGENTS.md` of 1,952 bytes**. That is 35 files and 425,316 bytes of upstream body. Twenty-two of the 35 are a `SKILL.md`, twelve are reference files under a skill directory, and one is `AGENTS.md`.

| Path under `.agents/skills/` | Files | Upstream bytes |
|---|---|---|
| `AGENTS.md` | 1 | 1,952 |
| `community-pr-readiness-check/` | 5 | 45,164 |
| `content-design/` | 1 | 13,959 |
| `conventions/` | 1 | 3,035 |
| `create-community-node-lint-rule/` | 2 | 10,111 |
| `create-instance-ai-eval/` | 4 | 121,441 |
| `create-issue/` | 1 | 12,134 |
| `create-pr/` | 1 | 7,143 |
| `create-skill/` | 1 | 6,764 |
| `db-migrations/` | 1 | 45,829 |
| `design-system/` | 3 | 14,118 |
| `experiments/` | 2 | 10,622 |
| `gh-stack/` | 1 | 43,529 |
| `human-like-code-review/` | 1 | 14,545 |
| `linear-issue/` | 1 | 11,105 |
| `loom-transcript/` | 1 | 3,641 |
| `nathan/` | 1 | 5,695 |
| `node-add-oauth/` | 1 | 11,537 |
| `protect-endpoints/` | 1 | 7,482 |
| `public-api/` | 2 | 21,811 |
| `reproduce-bug/` | 1 | 5,223 |
| `spec-driven-development/` | 1 | 3,229 |
| `telemetry/` | 1 | 5,247 |

**Byte identity was verified mechanically.** The blob hash of each upstream body was recomputed from the bytes and compared against the blob the tree API reports for that path at the pin: 35 comparisons and 35 matches. The byte length of each was compared against the same tree entry's `size` field as well.

## What the front matter does here, and why it differs from the other two fixtures

The design-spec and standards-spec fixtures each add a Headwater front-matter block above a file that carries none. **Twenty-two of these 35 files already carry Agent Skills front matter**, a YAML block with `name` and `description`. A second block above the first would not be read, because a parser takes the first block it meets. So the Headwater keys are merged into the block that is already there, below n8n's own keys, and no upstream key is changed or removed. The 13 files with no upstream front matter get a new block. Under both shapes every byte below the front matter is the file at the pin.

**The `summary` facet came from the corpus rather than from an author.** `kinds.governed_document` requires `summary`. For the 22 files with upstream front matter the summary is n8n's own `description` line, copied. For the other 13 it is the document's own first heading, copied. Thirty-five hand-written summaries would be invented prose inside a fixture whose point is byte identity, so the fixture takes the weaker summary and states the trade here.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. Assemble a scratch root out of three committed things and run the engine over it:

    ROOT=$(mktemp -d)
    cp -R docs/taxonomies/diataxis-site/fixtures/n8n/corpus/.agents "$ROOT/.agents"
    cp -R docs/taxonomies/diataxis-site/fixtures/n8n/.headwater "$ROOT/.headwater"
    cp -R .headwater/packages "$ROOT/.headwater/packages"
    headwater taxonomy resolve --root "$ROOT"
    headwater check --root "$ROOT" --no-cache --now 2026-09-01

The entry sits outside the corpus root on purpose, the same reason the other fixtures of this entry give. `sh tools/taxonomy/n8n-fixtures.sh` runs this corpus in CI, as a blocking step, and holds every figure of *What a run reports* against the run. [The design-spec README](../../../design-spec/fixtures/n8n/README.md#what-that-job-holds-and-what-it-does-not) says which figures that job holds and which it does not.

## What the taxonomy needed before it could read one file

**This entry, exactly as it ships, types none of these documents.** All four of its shelves are under `docs/`, and n8n keeps these files at `.agents/skills/`. The overlay adds one shelf, `.agents/skills/**`, homogeneous, carrying `how_to`.

**The whole tree is one shelf, and the corpus is what decides that.** `AGENTS.md` states that `.agents/skills/<name>/SKILL.md` is the shared form and that everything else under a skill directory belongs to that skill. Typing the twelve reference files as `kinds.reference` would split one skill across two shelves and two identifier schemes, and the corpus declares no discriminator that could carry the split. So the shelf is homogeneous and every file is a `how_to`, including the twelve reference files and `AGENTS.md` itself.

**Five namespaces are declared and one is used.** `taxonomy validate` refuses a resolved scheme that carries no namespace, and it refuses every scheme of the closure rather than the schemes the corpus mints under. So a corpus of how-to guides alone declares a namespace for the three Diátaxis kinds it holds no document of and for the base's `decision_id`. Only `how_to_id` is live, and all 35 documents carry an `N8N-HOW-{slug}` identifier.

**The identifier slug is the directory name and not the file stem.** Twenty-two of the 35 files are named `SKILL.md`, so a slug taken from the file stem would be the same value 22 times. The slug of a `SKILL.md` is its directory, the slug of a reference file is its directory and its stem joined, and the slug of `AGENTS.md` is `skills-agents`. `identifier.pattern.not_met` instantiates 35 times and reports nothing, and `identifier.claimed_twice` instantiates once and reports nothing, because no two of the 35 slugs are equal. The file-stem choice would not have passed quietly: forcing two documents onto one identifier makes `identifier.claimed_twice` report the pair by name, and this corpus was run that way to check. What no rule reads is the slug against the path that carries it, so a slug that matches no part of its own path is never reported at all.

## What a run reports

`headwater taxonomy resolve` then `headwater check --no-cache --now 2026-09-01` over the assembled root, with the committed `.headwater/`: **35 files under the corpus root, 35 typed, 0 excluded, 35 checked, 426 check instances, 136 findings, 118 of them errors and 18 warnings.** The warning count was 21 until [#783](https://github.com/headwater-ai/headwater/issues/783) put an inline quotation outside every prose rule. Three of these warnings read quoted words as n8n's own prose, and the check instances did not move. The census reads 35 `how_to`. The graph reads 35 nodes, 0 declared edge halves, and 14 prose links that did not resolve. `headwater check --strict` exits 1. Nothing under the corpus root is untyped, which is the case this fixture was vendored to record.

| Rule | Count | Severity |
|---|---|---|
| `section.required.missing` | 104 | error |
| `link.path.unresolved` | 14 | error |
| `voice.forbidden_construction` | 18 | warn |

**104 of a possible 105.** `kinds.how_to` requires `Goal`, `Steps` and `Verification`. Thirty-five documents times three headings is 105. The run reports `Goal` 35 times, `Steps` 34 times and `Verification` 35 times. One file writes a `Steps` heading by name, `.agents/skills/create-pr/SKILL.md`, and that is the only heading of the three this corpus supplies anywhere. The [review-rules fixture](../../../standards-spec/fixtures/n8n/README.md) recorded the same shape from a different kind and a different heading set.

**Thirteen of the 14 link findings are a property of the corpus root.** `corpus.root` is `.agents`, and 13 targets are relative paths that climb above it, such as `../../../packages/@n8n/instance-ai/evaluations/README.md` and `../../../AGENTS.md`. Each of the 13 was checked against the pin through the contents API and every one exists upstream. A root that reached the whole repository would resolve all 13.

**The fourteenth is a real dangling link.** `.agents/skills/design-system/rules/web-animation-guidelines.md:84` links `PRACTICAL-TIPS.md`, and no such path is under `.agents/skills/design-system/rules/` at the pin.

**`headwater check --fix` writes nothing here.** The `--fix` arm over the assembled root printed `no finding of this run carries a patch` and changed no file. That is 0 of 136. A missing heading, an unresolved link and a forbidden voice construction all need a rewrite.

**All three n8n fixtures take one version now, and a job holds it.** `.headwater/taxonomy.yml` here takes `headwater/standard` at `4.3.0`, which is the version `packages/` carries. This fixture took `4.2.0` and the two earlier ones took `4.0.0`, and `headwater taxonomy resolve` refuses that mismatch and names both versions, so for a time every one of the three pages printed a command that did not run. `docs/taxonomies/**` is outside this repository's corpus, so no check read any of the three scalars. `sh tools/taxonomy/n8n-fixtures.sh` now does: it runs the block above verbatim in CI, it fails on that refusal, and it holds every figure of this section against the run.

## What this corpus cannot see

`.claude/plugins/n8n/skills/` holds 24 entries at the pin. Twenty-two are symlinks at mode `120000`, one per shared skill directory, and the set of 22 link names equals the set of 22 shared directory names with no difference in either direction. The other two entries are a real directory, `setup-mcps/`, and the `SKILL.md` inside it. A corpus rooted at `.agents` walks none of that tree, so the run types 35 documents where a run over both trees would type 36. The one document it cannot reach is the only skill that no shared directory carries.

## What ages, and what does not

**The pin does not move and the upstream does.** Every count above is of the 35 blobs at `b0550cb`. Re-run the assembly at a later commit of `master` and every number is a different measurement.

**One finding is about a declaration rather than about a document.** The 104 `section.required.missing` errors stop the day this entry stops requiring headings that an imperative paragraph already satisfies, and that is a change to the entry rather than to n8n. [The evaluation](../../../../evaluations/n8n-worked-example.md) records it as a finding against the library.
