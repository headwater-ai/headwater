# Fixtures for the Diátaxis taxonomy

A miniature documentation set for Beacon, under `corpus/`. Beacon is invented, and it is the same invented system that the [design-spec fixtures](../../design-spec/fixtures/README.md) and the [decision-record fixtures](../../decision-record/fixtures/README.md) use, so a reader can hold three entries over one project.

Everything here is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`.

## Every mode page here is a `specification`, and that is a finding rather than an accident

This entry declares no kind, so its worked corpus has to borrow one. The base package declares two concrete kinds, `decision` and `specification`, and the corpus below takes both. So the tutorial page, the how-to page, the reference page and the explanation page are all of one kind, and every one of them carries the `Scope` and `Behavior` headings that the base's section contract requires.

A tutorial does not naturally take those two headings. That is [finding 3](../doctrine.md#findings) of this entry, and it arrives here rather than in the doctrine because a corpus is what shows it. It is also the clearest argument for the kind-set form of the same tradition: a corpus whose pages *are* the four modes wants kinds that carry a tutorial's shape, and no facet supplies a section contract.

## How to run this corpus

`headwater check` reads one corpus root, and this tree sits inside a directory that this repository excludes from its own corpus. Assemble a scratch root that holds `packages/headwater-standard/`, this entry's `bundle.yml` at `bundles/diataxis/`, and the tree under `corpus/docs/` at `docs/`. Point the scratch copy of `package.yml` at that directory by rewriting `contents.bundles` to `../../bundles`, which is the one scalar `headwater taxonomy publish` rewrites anyway. Bind it with a `.headwater/taxonomy.yml` that names the package, selects `bundles: [diataxis]` and sets `corpus.root` to `docs`, and add a one-line overlay declaring `identifier_schemes.decision_id.namespace`, which the base leaves to a consumer. Then run `headwater taxonomy resolve` and `headwater check`.

The entry sits outside the corpus root on purpose. A run that walked it would count this file and the doctrine in its census, and the recorded numbers below would then move every time the entry gained a page.

**That a fixture corpus needs assembling is a finding rather than an inconvenience**, and the [decision-record fixtures](../../decision-record/fixtures/README.md#a-runner-reads-these-files-now-and-that-changes-what-this-file-is) already record it. Nothing in CI runs this corpus. Nothing declares where a reference corpus sits or how a run reaches one.

## What each document exercises

| Document | Kind | `reader_mode` | What it is here for |
|---|---|---|---|
| `docs/specifications/getting-started-with-beacon.md` | `specification` | `tutorial` | The lesson: one worked exercise with a guaranteed outcome |
| `docs/specifications/rotate-a-signing-key.md` | `specification` | `how-to` | The recipe: a reader who already has the goal |
| `docs/specifications/the-delivery-api.md` | `specification` | `reference` | The description: fields, types and admitted values, and no reasoning |
| `docs/specifications/why-delivery-is-at-least-once.md` | `specification` | `explanation` | The argument: no task and no field list |
| `docs/specifications/beacon-overview.md` | `specification` | unset | The page that mixes explanation and reference, which nobody has split |
| `docs/decisions/0001-sign-every-payload.md` | `decision` | unset | The kind that no mode fits, which the theoretical-foundations evaluation names |
| `docs/specifications/publish-a-release.md` | `specification` | `cookbook` | The planted defect: a fifth name for a mode the set already has |

## What a run reports

`headwater check --no-cache --now 2026-08-24` over the assembled root: **7 files under the corpus root, 7 typed, 7 checked, 80 check instances, 7 findings.** One of the seven is planted.

**The planted one.** `docs/specifications/publish-a-release.md` reports `facet.value.not_permitted`, an error, under `OB-FACET-2`:

    `reader_mode` admits tutorial, how-to, reference, explanation, and this document declares `cookbook`

That is the whole decisive claim of this entry: a value inside the four passes, a value outside it is refused, and the refusal names the file and the value.

**The other six are one defect repeated.** Every `specification` in the corpus reports `identifier.unusable`, because the base's `specification` kind mints under no identifier scheme and no edge can name a document of it. That is [HW-OBL-0107](../../../obligations/0107-the-base-package-ships-a-kind-that-the-scaffolder-refuses-to-write.md), already recorded against the base package, and this corpus reproduces it on a first run rather than discovering it.

## The run against the same corpus with the bundle deselected

The comparison that shows the declaration is what refuses the value. Assemble the same root with `bundles: []` and change nothing else. The run reports **7 files, 7 typed, 7 checked, 80 check instances, 6 findings**, and `cookbook` passes in silence: no rule reads a facet that no taxonomy declares.

Two numbers deserve their reasons. **The finding count moves by exactly one**, which is the planted document and nothing else, so the declaration refuses one value and reclassifies nothing. **The instance count does not move at all**, and that is not an oversight. `facet.value.not_permitted` already generated one instance for each of the two kinds, because the base's `status` facet is enumerated too, so a second enumerated facet joins an instance that already existed.

**The exit status is the same in both arms and it proves nothing here.** `headwater check --strict` exits 1 either way, because the six `identifier.unusable` findings are errors before this entry exists. The discriminator is the seventh finding and its message, never the exit code.

## The two controls

Two runs measured the two halves of what the `kinds.governed_document.facets.optional` line does. Both were made against this corpus with the bundle edited, and neither edit is committed.

**Control A: delete the line.** `headwater taxonomy validate` exits 1:

    the resolved taxonomy `facets.reader_mode`: facet canons: relevance: nothing reads it. No kind requires it, no shelf discriminates on it, no expectation reads it, no projection filters on it, and it carries no engine role

So the line is what makes the entry resolve at all. This is also the measurement behind [finding 2](../doctrine.md#findings): the refusal names `require` and `optional` as two ways to satisfy the same canon, and this entry ships on the second.

**Control B: attach the facet to `kinds.decision` instead of to `kinds.governed_document`.** The taxonomy validates and resolves, and the run still reports `facet.value.not_permitted` against `publish-a-release.md`, which is a `specification`. So the attachment satisfies the relevance canon and does not scope the value check. `facet.value.not_permitted` generates an instance for every kind that does not `forbid` an enumerated facet, and neither kind here forbids one.

## The composition run: this facet over another entry's corpus

Every run above is over this entry's own Beacon corpus. This one is not. It selects the `design-spec` entry beside this one and reads the facet over that entry's eleven fixture documents, which is the demonstration the doctrine said this entry had not made. [`tools/repo/diataxis-facet-fixtures.sh`](../../../../tools/repo/diataxis-facet-fixtures.sh) performs it, CI runs that program as a blocking step, and every number below is printed by it rather than typed here. That matters for this file in particular: `docs/taxonomies/**` sits outside this repository's corpus root, so a table here reaches no gate, and the `diataxis-site` entry measured the cost of that when a planted defect lived in its worked corpus for two days with every gate green.

**The root.** A scratch tree under `mktemp -d` holds `packages/headwater-standard/`, the sibling entry's `fixtures/corpus/docs/` at `docs/`, and a `.headwater/taxonomy.yml` that selects **`design-spec`, `diataxis`, `evidence-and-obligation`**. The suite derives that selection rather than listing it: the sibling entry, the entry under demonstration, and the transitive closure of every `requires:` below them. The overlay is derived too. A first `taxonomy validate` over `add: {}` names each identifier scheme that carries no namespace, and the suite declares one namespace for each name it reported. So neither the selection nor the overlay goes stale when an entry gains a dependency or a scheme.

**The two arms.** Both arms hold the same eleven documents. The labeled arm writes `reader_mode: explanation` into `docs/evaluations/naming-survey.md` and `reader_mode: cookbook` into `docs/evaluations/queue-durability.md`, which are the first two documents of that corpus in sorted path order. Any two would carry the claim, because the facet is attached to the base's abstract kind and reaches every kind under it. **Both lines land in the scratch copy and in neither corpus in the tree**, and a case asserts that by reading the sibling corpus for a `reader_mode` line of its own and expecting none. The sibling entry's own fixtures README is not touched by any of this.

**What the run reports**, at engine 0.1.2 with the clock injected as the day of the run, 2026-09-11:

| | unlabeled | labeled |
|---|---|---|
| `headwater taxonomy validate` | exit 0, last line `headwater/standard is valid` | — |
| `headwater taxonomy resolve` | exit 0 | — |
| census | 11 files under the corpus root, 11 typed | 11 files, 11 typed |
| check instances | 115 | 115 |
| findings | **26**, of which 24 error and 2 warn | **27**, of which 25 error and 2 warn |
| `headwater check` | exit 0 | exit 0 |
| `headwater check --strict` | exit 1 | exit 1 |

**The two finding counts belong to the sibling corpus, and the suite prints them without asserting them.** What it asserts is the difference between the arms. So this table ages without reddening a gate, and a reader who reads a different absolute out of the suite's own output should trust that output over this row.

**The whole difference is one finding.** `docs/evaluations/queue-durability.md` reports `facet.value.not_permitted`, an error, under `OB-FACET-2`:

    `reader_mode` admits tutorial, how-to, reference, explanation, and this document declares `cookbook`
    fix: change `reader_mode` in docs/evaluations/queue-durability.md to one of: tutorial, how-to, reference, explanation

**The correctly labeled page reports nothing under `reader_mode`.** It is not a clean page and this file does not claim that it is: `docs/evaluations/naming-survey.md` carries four findings in both arms, for a missing `title`, an unusable identifier, a restated `doc_type` and a `doc_type` value its shelf refuses. All four predate this entry and none of them names the facet. The suite prints that count beside the case, because a page that reported nothing at all would read the same way as a page that reported and stayed silent about the facet.

**Two numbers deserve their reasons, and one of them is the same reason as the Beacon run above.** The instance count does not move, because `facet.value.not_permitted` already had one instance per kind from the base's own enumerated facets, and a second enumerated facet joins an instance that already existed. The finding count moves by exactly one, so the declaration refuses one value across another entry's corpus and reclassifies nothing else in it.

**No exit status separates the arms, and the suite measures that rather than assuming it.** The plain run exits 0 in both arms and the strict run exits 1 in both, because 24 of the 26 findings in the unlabeled arm are errors, and every one of them is reported in both arms and raised by a rule this entry declares nothing about. The discriminator is the twenty-seventh finding and its message. A reader who takes an exit status as the result of this demonstration reads a number that was never about it.

## What the composition did not need, and what it could not do

The run above needed no operation that [HW-OBL-0040](../../../obligations/0040-composition-between-two-library-entries-has-no-add-only-form.md) holds. It selects two entries beside each other, and the facet reads over the other entry's kinds because one `optional` line attaches it to the base's abstract kind, which every concrete kind of every entry inherits from. Nothing here reaches an address that the `design-spec` entry writes, and nothing here is an `override` or a `remove`. **What it could not do is make a mode mandatory over that corpus.** A demonstration in which every page of another entry's corpus must carry a value wants `kinds.<k>.facets.require` on a kind that entry declared, and an `add` cannot reach into a `facets` block another bundle already wrote. That is the operation with no add-only form, and it is the one HW-OBL-0040 still waits on a ruling for. So this file records the optional half as measured and the required half as unavailable, and it is the report from the demonstration side that the ruling has so far lacked. Nothing above settles the obligation or picks a value inside it.

## What the audit says, and why one line of it is an artifact

`headwater taxonomy audit` over the same root reports `reader_mode` as carrying no role, four values declared, five of the seven documents carrying it, and five distinct values in use. The fifth value is `cookbook`, which is the planted defect counted as data.

It also reports that **`reader_mode` fixes `status` over the five documents that carry both**. That is the orthogonality canon, and here it is an artifact of a seven-document corpus in which every document is `current`. Any facet in this corpus fixes `status`. The audit states that nothing declares where the line is and that no share it prints carries a verdict, and this is a corpus that shows why: the canon is corpus-measured, and a corpus this small measures its own size.

## What a run does not report, and cannot

**`beacon-overview.md` mixes two modes and the run says nothing about the mixing.** The page argues for the model in prose and then lists the fields of the model in a table. It is the failure the method exists to name, and it is invisible to every rule in this engine. The run does report `identifier.unusable` against the file, along with every other `specification` here, and that finding is about the base package's identifier scheme rather than about this page's modes. `OB-DX-1` is declared with an `unverifiable` disposition for exactly this, and the [doctrine](../doctrine.md#a-document-that-mixes-modes) states the reasoning.

**`0001-sign-every-payload.md` carries no `reader_mode` and nothing asks it to.** The facet is optional, so an absent value is not a finding. That is the intended outcome for a kind that no mode fits, and it means the corpus cannot tell a considered omission from an author who never read the doctrine. No declaration in this language can, because "required, except where the author judged it inapplicable" is not a thing a facet contract can say.

## What ages, and what does not

Nothing here is windowed. No expectation reads `reader_mode`, no relation is declared in the corpus, and `last_verified` is inside the base's 180-day staleness policy as of the recorded run. A rerun after 2027-02-16 adds staleness findings against every document and changes none of the seven above.
