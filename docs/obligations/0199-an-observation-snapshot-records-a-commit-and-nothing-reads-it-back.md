---
id: HW-OBL-0199
status: discharged
status_since: 2026-09-26
summary: "Discharged by the owner's ruling on issue 937. An observation's commit is provenance, and the engine never reads its ancestry. The standard taxonomy states that the adopter's process keeps it honest."
last_verified: 2026-09-26
title: "An observation snapshot records a commit and nothing reads it back"
waiting_on: ruling
---

# An observation snapshot records a commit and nothing reads it back

## Context

[#934](https://github.com/headwater-ai/headwater/issues/934) gave the register an observation dimension. `crate::observation::Observations` reads `.headwater/observations.yml`. A control naming a mechanism outside the engine discharges its obligation only where an entry there names that control. Each entry also names a commit. [Spec 4](../spec/04-assurance-model.md#every-obligation-has-exactly-one-disposition) and [HW-DR-0073](../decisions/0073-a-verification-is-a-kind-and-its-identity-is-minted-rather-than-found-in-the-code-that-cites-it.md) ruling 3 both write the shape as "a control naming the commit it ran against."

`headwater check` runs no version control command. [`headwater-vcs`](../../engine/crates/vcs/src/lib.rs) states that boundary for the crate that owns it, and the check-evaluation path never links it. A comparison against this repository's own history, or against the commit that last changed a control's declaration, needs exactly that command.

## Obligation

Nothing reads a control's commit field back, and this half of the gap stands as first written. `Observations::observed` asks only whether an entry names the control, and the commit is carried and never inspected.

An entry naming a commit this repository never held discharges a control the same way an entry naming the real commit does. `CT-EXT-1: {commit: "never ran"}` and `CT-EXT-1: {commit: 8f2c1a0}` read identically to `Observations::observed`. So does `CT-EXT-1: {commit: ""}`: the field parses as an empty string, `Observations::at` accepts it as present, and discharge follows the same way. Only an entry naming no `commit` key at all is refused, as a malformed entry.

[#937](https://github.com/headwater-ai/headwater/issues/937) narrowed the other half, for a verification and not for a control. A `kind: verification` entry now carries a `criterion_digest` beside its commit. That digest is the acceptance criterion's bytes, hashed at the moment the snapshot was written. `crate::verification::Verified` compares it against the criterion's digest today, off the same census read every edge-scoped rule already trusts. The two disagree exactly when the criterion's content moved after the snapshot. HW-DR-0073 ruling 3 calls that state a verification going `suspect`. No version-control command runs to answer it. Content stands in for history, and `crate::verification`'s own module comment states why the substitution is exact and not approximate for this one question. A verification entry now also fails a syntax check on its `commit`: non-empty, all hexadecimal, four to forty characters. So the empty-string case this record names above does not repeat for a verification. Such an entry is a reported `Problem`, and not a silent `declared`.

That syntax check is not the same fact as an ancestry check, and this record still owes the difference. `plausible_commit` (`crate::observation`) refuses `""` and `"never ran"`. It accepts `deadbeef`, which reads as a real Git object id and may never have existed on this tree. Telling the two apart needs `git merge-base --is-ancestor` or its equivalent. `headwater-vcs` already knows how to produce that fact, but no manifest carries it to the check-evaluation path today, and [spec 12](../spec/12-check-layer.md#temporal-inputs-the-clock-and-the-prior-version) rules that the path opens no file a manifest did not name. Building that manifest, and wiring `headwater check` to read it, moves spec 12's stated boundary. It would widen from "a change's prior version" to "a change's prior version, and whether a named commit is an ancestor of this tree." [#937](https://github.com/headwater-ai/headwater/issues/937)'s adjudication ruled that widening out of its own scope, on exactly this reasoning, and not by deciding it inside a pull request description.

A snapshot recorded once still discharges forever, for a control and for a verification both. Suppose a control's mechanism, its taxonomy declaration, or the pipeline it names changes after the snapshot commit. Nothing re-reads the snapshot against that change. A verification's freshness check answers the equivalent question, for the one document a snapshot names as proved. It answers nothing about the mechanism or the pipeline behind that document.

## Discharge

The owner ruled on 2026-09-25, in [a comment on #937](https://github.com/headwater-ai/headwater/issues/937#issuecomment-5828949496): "No, keep boundary: state in the taxonomy that the adopter's own process keeps the commit field honest, and close box 4 on that. The content digest already catches a changed criterion, and a widening adds a new input to every check run."

The ruling says three things. The manifest boundary does not widen. The taxonomy states that the process of the adopter keeps the commit field honest. The content digest is the answer to a changed criterion. So no ancestry fact reaches the check-evaluation path, and this record's second remedy is the one taken.

This change wrote that statement. The comment above `controls` in [the standard taxonomy](../../taxonomy-source/headwater-standard/taxonomy.yml) states that the `commit` of an observation entry is provenance. It states that the engine checks the `commit` of a verification entry for form only, and never for ancestry. It states that the process of the adopter, which is the job that writes the snapshot, keeps the field honest. Version 4.6.2 of `headwater/standard` publishes that comment.

The ruling does not address the syntax check. This change adds no syntax check for the commit of a control, and the commit of a control stays stored and not compared. That is a statement of this change and not of the ruling.
