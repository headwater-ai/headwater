---
id: HW-OBL-0232
status: current
status_since: 2026-10-02
summary: "The harness step writes no hook and no hook registration, because no ruling chooses a script body, a hook verb or a flag."
last_verified: 2026-10-02
title: "headwater init --harness ships no hook and no hook configuration for any harness"
waiting_on: ruling
provenance:
  warrant: asserted
relations:
  traces_to:
    - HW-DR-0077
    - HW-IFACE-headwater-init
---

# headwater init --harness ships no hook and no hook configuration for any harness

## Context

[Spec 0](../spec/00-vision-and-scope.md) says that the harness hooks ship with the first release. [Spec 5](../spec/05-ai-integration.md) calls the skills and the hooks the adoption model. [HW-DR-0077](../decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md) found that no release carried either, and it says that an obligation record holds that gap.

Since [#1586](https://github.com/headwater-ai/headwater/pull/1586), [`headwater init --harness`](../interfaces/headwater-init.md#the-harness-step) writes the skills half. It writes four skills at two paths each and the maintainer agent at one path. It writes no hook. The repository that maintains this engine runs six hook positions: intent, read, write, review, derived and touch. It registers them in its own harness configuration, and each position is a `sh` script that puts together several verbs. [Spec 16](../spec/16-harness-support.md) names a hook configuration for each of three harnesses: Claude Code, Copilot and Codex. The step writes none of the three.

Two rulings stop each obvious delivery:

- HW-DR-0077 says that the executable that a harness calls is a `headwater` verb, and that no script body ships. So the step cannot ship the scripts as they are.
- Spec 5 says that no hook introduces a verb. A `headwater hook <moment>` verb is a second entry point to `route` and to `check`. So the step cannot fold each script into one hook verb.

A third delivery is a flag on the verbs that ship, for example one that makes `route` read the payload of a harness. That is arguably the same second entry point. The choice among the three amends a ruling, so it is the owner's ([#1578](https://github.com/headwater-ai/headwater/issues/1578)).

## Obligation

- The corpus owes a ruling on the delivery of a hook. The three choices are script bodies with an amendment to HW-DR-0077, and one hook verb with an amendment to spec 5. The third is a payload flag on the verbs that ship, with a sentence in spec 5 that states what an entry point is.
- After that ruling, the harness step owes a hook at each position that an adopter can use. It also owes a hook configuration for each harness that spec 16 names.
- Spec 0 and spec 5 owe a sentence that matches what a release carries. Spec 0 already says that no release ships the hooks.

## Discharge

The record discharges when two things are true. An owner ruling chooses the delivery. Under that ruling, `headwater init --harness` writes the hook positions, and the hook configuration of each harness that spec 16 names. A test then holds that the step writes each configuration, and that each hook calls only what the ruling allows.
