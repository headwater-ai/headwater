---
id: HW-DR-0116
status: current
status_since: 2026-11-27
summary: "One rule, facet.value.not_of_type, reads a document facet declared date, integer, boolean or duration against its type. It uses the meta-schema's own predicate, which now reads a date against the calendar, so 2026-02-30 fails both gates"
last_verified: 2026-11-27
title: "116 — A document facet value is read against its declared type by one rule and one predicate"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5.5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0058
    - HW-OBL-0171
  governs:
    - engine/crates/check/src/facet_type.rs
---

# 116 — A document facet value is read against its declared type by one rule and one predicate

## Context

A facet declaration can state a scalar `type`: `string`, `integer`, `boolean`, `date` or `duration`. Before this record, no rule read a document's value against that type ([HW-OBL-0171](../obligations/0171-no-rule-reads-a-document-facet-value-against-the-type-its-declaration-states.md)). The meta-schema reads a taxonomy and never a document. `facet.value.not_permitted` declines a value that is not a scalar, because "the meta-schema owns shape". `facet.value.blank` ([HW-DR-0058](0058-a-blank-facet-value-is-a-rule-of-its-own-and-it-reads-every-string-facet.md)) reads `type: string` only. So `status_since: {from: x}` and `last_verified: 2026-02-30` passed `check --strict` at exit 0, and each rule that computes with a date then read a date that nobody wrote.

The meta-schema's own type test had a second defect. Its date arm read digit positions only, so it admitted `2026-02-30` and `0000-00-00`. The check layer's date reader, `Date::parse`, refused both. One declaration had two parsers, which is the defect that `headwater_yaml::core_schema` records for the boolean reading.

On 2026-11-27 the owner answered the recommended form on #1492 with "yeah why wouldn't we?". That form was a Shape-origin rule per declared type, with the date case of the hostile review. The owner then gave the reading of "a rule per declared type" to the parent. The parent ruled for one rule identifier with a case per type.

## Decision

**One rule, `facet.value.not_of_type`, reads every document facet that declares `date`, `integer`, `boolean` or `duration`.** It is generated from the facet declarations, as `facet.value.blank` is. A kind that forbids a facet does not carry it in the population. The rule has one identifier and not one for each type name. Four identifiers need four controls, four rows in each recorded fixture and four ledger entries, all for one table of predicates. `type: string` stays with `facet.value.blank`, so the two populations do not overlap and one defect gets one finding.

**The predicate is `ScalarType::admits` in the meta crate, and the meta-schema and the rule both call it.** A date is a real date of the proleptic Gregorian calendar in the form `YYYY-MM-DD`. An integer and a boolean are the readings of the YAML core schema. A duration is a count of digits and one unit from `d`, `w`, `m` and `y`. The date arm now reads the calendar, and `Date::parse` in the check layer calls the same function, `calendar_date`. So a date that `taxonomy validate` refuses in a taxonomy is a date that this rule refuses in a document. The check layer also computes with each date that the rule admits, and with no other. A unit test in the check crate holds the two readers to one answer on eight values.

**A null, a sequence or a mapping on a typed facet is this rule's finding.** That is a break from the posture that the meta-schema owns shape. The posture holds for a taxonomy, which the meta-schema validates. No schema validates a document value, so for a document the posture left the shape to nobody.

**A type name that the engine does not read decides nothing.** The meta-schema refuses any `type` outside the five names before it reaches a lock, so no unknown name reaches the rule. The rule maps the name through `ScalarType::parse`, and a name that gives no type gives no instance.

**A date after the run is not this rule.** `last_verified: 2999-01-01` is of type `date`. Whether it defeats `stale_after_days` is a question about time. The probe oracle `future-date` in `.headwater/probe.yml` holds that question, as the hidden task of probe #1835.

The finding is an error at the value, and it carries no patch, because the value is a fact that the author knows. `headwater/standard` 4.31.0 binds the rule to `OB-FACET-4` through `CT-FACET-4`, at `posture: advisory` with promotion criteria.

## Consequences

A document whose date facet holds an impossible date, a word, a null, a list or a mapping fails a strict run. On this repository on 2026-11-27, the rule had 739 instances, one for each of the 739 typed documents, and it reported 0 findings.

A taxonomy that writes an impossible date in a date member now fails `taxonomy validate`, where before it passed. That is correct, and it is the same answer that the check layer gave for the same text.

`engine/crates/cli/tests/facet_type.rs` runs the date cases of the rule through the binary. The unit tests in `engine/crates/check/src/facet_type.rs` hold the integer, boolean and duration cases. This repository declares no document facet of those three types that the next major version of its package keeps.

This question opens again when the meta-schema admits a sixth type name. A document value of that type then needs a predicate, and this record does not say what it is.
