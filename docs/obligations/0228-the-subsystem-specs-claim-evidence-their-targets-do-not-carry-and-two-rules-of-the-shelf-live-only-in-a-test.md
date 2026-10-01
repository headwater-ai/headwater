---
id: HW-OBL-0228
status: current
status_since: 2026-10-01
summary: "Every subsystem spec claims evidenced and traces to asserted documents, HW-DR-0098 does not point at the measurement spec, and the rule that a spec governs exactly its crates' src trees is stated only by subsystem_map.rs."
last_verified: 2026-10-01
title: "The subsystem specs claim evidence their targets do not carry, and two rules of the shelf live only in a test"
waiting_on: ruling
---

# The subsystem specs claim evidence their targets do not carry, and two rules of the shelf live only in a test

## Context

Run `20261001-1107` wrote the last five subsystem specs of #1288, and its builds and the maintainer wrote seven intake lines about the shelf they built. The product owner ruled each one RECORD, because none names a reader outside this repository. On `d309e914` these hold:

- Each of the ten specs under `docs/subsystems/` declares `evidence_basis: evidenced` under `provenance`. Each one declares `traces_to` onto documents whose `warrant` is `asserted`, such as HW-DR-0064, HW-SPEC-parse-and-census and HW-SPEC-graph-build. So `warrant.evidence.unsupported` reports each spec. `docs/subsystems/authoring.md` alone carries five of those warnings, and `docs/subsystems/taxonomy-distribution-and-audit.md` traces onto HW-OBL-0119, whose warrant is `proposed`, a value outside the closed set of spec 3.
- The *The unit is a subsystem, not a crate* paragraph of HW-DR-0098 names the measurement layer as a subsystem that the diagram does not draw. It gives no link to `docs/subsystems/measurement.md`.
- `a_row_that_links_a_subsystem_spec_is_governed_by_it` in `engine/crates/cli/tests/subsystem_map.rs` holds that each subsystem spec governs exactly `engine/crates/<crate>/src/**` for each crate of its row and nothing else. Only that test states the rule. Neither HW-DR-0098 nor spec 6 states it.

## Obligation

One ruling settles the first item: either the shelf claims `evidence_basis: reconstructed`, because a spec read from the code is reconstructed, or a person reads each asserted target and accepts it. Until the ruling, every new subsystem spec adds warnings of the same kind. The second item is one link in HW-DR-0098. The third item is one sentence in HW-DR-0098 or in the subsystem table of spec 6, which the test then cites.

## Discharge

The first item discharges when `headwater check` reports no `warrant.evidence.unsupported` finding on `docs/subsystems/`, or when a recorded ruling says why the findings stand. The second discharges when HW-DR-0098 links the measurement spec. The third discharges when a document states the rule that the test holds. The record discharges when all three have.
