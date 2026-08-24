# Fixtures for the design-spec taxonomy

A miniature corpus in the tradition, under `corpus/`. It exists so that the schema meets documents rather than only a reader. Beacon is invented, and the tree is the smallest one that exercises every declaration in `bundle.yml`.

Everything in the corpus is invented, including the names. `warrant: accepted` requires an `accepted_by` that names a human, so the fixtures write `fixture-acceptor`. A placeholder that reads as one keeps a fixture from implying a real person.

No runner reads these files. There is no engine, so the expected findings below are stated in prose rather than in a manifest that nothing executes. The day an engine exists, this file states what a run over `corpus/` must report. It becomes the fixture manifest at that point.

## What each document exercises

| Document | Kind | What it is here for |
|---|---|---|
| `docs/spec/00-motivation.md` | `design_spec` | The head of the series. Sequence zero, and a scope section that states a refusal |
| `docs/spec/01-model.md` | `design_spec` | The core-model stage, and the target of the review record below |
| `docs/spec/02-transport.md` | `design_spec` | The tombstone convention: a retained `superseded` document whose body is a redirect map |
| `docs/spec/04-transport.md` | `design_spec` | The successor. A gap at 3 shows that the series is ordered and not contiguous |
| `docs/spec/09-decisions.md` | `decision_register` | The `rationale` kind on the same shelf, and one `cites_evidence` edge |
| `docs/spec/13-obligations.md` | `obligation_register` | The `obligation` purpose, and a claim that names its instrument |
| `docs/evaluations/queue-durability.md` | `evaluation` | Evidence that a register cites, with both halves of the reciprocal edge present |
| `docs/evaluations/naming-survey.md` | `evaluation` | Two deliberate defects, below |
| `docs/reviews/first-pass-prompt.md` | `review_prompt` | The instrument, committed with its findings |
| `docs/reviews/first-pass-findings.md` | `review_record` | One application of that instrument, with an `assesses` edge |
| `docs/reviews/second-pass-prompt.md` | `review_prompt` | One deliberate defect, below |

## What a run must report

Three findings are planted. A run that reports fewer has a hole in it, and a run that reports more has found something this file did not intend.

**1. `naming-survey.md` carries `doc_type` on a homogeneous shelf.** The `evaluations` shelf declares one kind, so placement already states it, and `kinds.evaluation` forbids the facet. Expected as an error, from the rule that [spec 2](../../../spec/02-taxonomy-model.md#placement-is-primary-metadata-fills-the-gap) states: a second statement of one fact eventually disagrees with the first.

**2. `naming-survey.md` is cited by no register.** The `evidence-cited` expectation on `kinds.evaluation` gives thirty days from `status_since`, and the date is 2026-05-01. Expected as a warning against the evaluation, which is where the reader who can act will look.

**3. `second-pass-prompt.md` has no review record.** The `instrument-applied` expectation gives ninety days from `status_since`, and the date is 2026-01-10. Expected as a warning against the prompt.

## What a run reports that is not planted

**Every document in `docs/spec/` is unlinked to code.** No `governs` edge exists, because Beacon has no source tree in this fixture. That is the state of any design corpus written before its implementation, including this repository's own. It reproduces what [the first-run walkthrough](../../../evaluations/default-taxonomy-first-run.md#three-things-the-derivation-contradicts) found when it wrote the base out. A behavior-serving kind with no edge to code has nothing to attach to. Whether a pre-implementation corpus reports that as an orphan class or as an ordinary state is a question for the engine. The fixture is here to force it.

**Dates are fixed, and they will age.** The two windowed expectations fire because the origin dates are old. They stay old, so the fixture keeps working. A fixture whose verdict depends on the day it runs is not a fixture.
