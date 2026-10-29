---
id: HW-OBL-0232
status: current
status_since: 2026-10-02
summary: "Since #1871 the harness step writes the Claude Code hooks and their registration. It still writes no hook registration for Copilot or Codex."
last_verified: 2026-10-29
title: "headwater init --harness ships no hook and no hook configuration for any harness"
waiting_on: build
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

Since [#1586](https://github.com/headwater-ai/headwater/pull/1586), [`headwater init --harness`](../interfaces/headwater-init.md#the-harness-step) writes the skills half. It writes four skills at two paths each and the maintainer agent at one path. Until #1871 it wrote no hook. The repository that maintains this engine runs six hook positions: intent, read, write, review, derived and touch. It registers them in its own harness configuration, and each position is a `sh` script that puts together several verbs. [Spec 16](../spec/16-harness-support.md) names a hook configuration for each of three harnesses: Claude Code, Copilot and Codex. Until #1871 the step wrote none of the three.

Two rulings stop each obvious delivery:

- HW-DR-0077 says that the executable that a harness calls is a `headwater` verb, and that no script body ships. So the step cannot ship the scripts as they are.
- Spec 5 says that no hook introduces a verb. A `headwater hook <moment>` verb is a second entry point to `route` and to `check`. So the step cannot fold each script into one hook verb.

A third delivery is a flag on the verbs that ship, for example one that makes `route` read the payload of a harness. That is arguably the same second entry point. The choice among the three amends a ruling, so it is the owner's ([#1578](https://github.com/headwater-ai/headwater/issues/1578)).

The owner ruled on 2026-10-03, on #1578. One `headwater hook` verb delivers the hooks, and no shell script ships first. The change that builds the verb amends the spec 5 rule that no hook introduces a verb, and it makes that amendment first. The hook work does not block the current milestone, so [#1648](https://github.com/headwater-ai/headwater/issues/1648) carries it in milestone 0.16.

The owner ruled again on 2026-10-28 and 2026-10-29, in the body of epic [#1868](https://github.com/headwater-ai/headwater/issues/1868). That ruling is later than the one of 2026-10-03, and it replaces it for delivery. `headwater init --harness` ships the hooks layer as `sh` scripts, and the hook verb does not come first. HW-DR-0077 records this in its amendment of 2026-10-29. [#1871](https://github.com/headwater-ai/headwater/issues/1871) built it. The step now writes six scripts under `.claude/hooks/`: intent, read, write, review, lib and touch. It adds their entries to `.claude/settings.json` and the `headwater mcp` server to `.mcp.json`. Each script calls verbs only. `engine/crates/cli/tests/init_hooks.rs` runs them in a scratch adopter.

## Obligation

- The harness step owes a hook configuration for each harness that spec 16 names. It writes the Claude Code configuration. It owes the Copilot configuration, `.github/hooks/hooks.json`, and the Codex configuration.
- Spec 0 and spec 5 owe a sentence that matches what a release carries. Spec 0 says that no release ships the hooks, and that stays true until a release carries #1871.

## Discharge

The record discharges when `headwater init --harness` writes the hook configuration of each harness that spec 16 names. A test must then hold that the step writes each one, and that each hook calls only verbs. The Claude Code half is done: `init_hooks.rs` holds it.
