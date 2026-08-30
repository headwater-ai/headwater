# Fixtures for the evidence-and-obligation taxonomy

This entry has no worked corpus of its own yet, and that is a stated gap rather than an oversight — see [finding 2](../doctrine.md#findings).

`evaluation`, the one kind this entry declares, is already exercised twice, in both cases by assembling this entry alongside the one that needs it:

- The [design-spec fixtures](../../design-spec/fixtures/README.md) place `docs/evaluations/queue-durability.md` and `docs/evaluations/naming-survey.md`, both cited (or deliberately not cited) through `cites_evidence` from `design-spec`'s registers.
- The [decision-record fixtures](../../decision-record/fixtures/README.md) place `docs/evaluations/retry-ceiling-measurement.md`, reached through `relations.discharges` from an `obligation_record`.

Neither corpus selects this entry alone. A fixture that did would place one or more `docs/evaluations/**` documents with nothing citing or discharging against them, and it would measure what `headwater check` reports about an `evaluation` that sits on no other entry's graph at all — which today is nothing, because no rule in this library reads `evaluation` except through a relation that a different entry declares. Writing that fixture is future work, not a blocked one: nothing in the taxonomy needs it to resolve.
