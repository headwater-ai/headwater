---
id: HW-OBL-0231
status: current
status_since: 2026-10-01
summary: "Run 20261001-1107 surfaced five findings about its CI toolchain step, the n8n fixture scripts and a wedged verifier. None has a reader outside this repository, so this record files them together."
last_verified: 2026-10-01
title: "Five run-tooling gaps from run 20261001-1107, filed together"
waiting_on: build
---

# Five run-tooling gaps from run 20261001-1107, filed together

## Context

`hw-run-policy` sends a finding about the build order, its tooling or this repository's CI to this shelf. Run `20261001-1107` wrote five such intake lines, and the product owner ruled each one RECORD. Each was checked on `d309e914`.

## Obligation

**The CI toolchain step.**

- `tools/ci/toolchain.sh` installs stable Rust with a fallback for the self-hosted runner, whose image cannot `rustup update` across overlayfs (EXDEV). The script is in governed scope, and no document governs it. The EXDEV measurement is written only in its header comment and in the commit message of #1556. A record under `docs/process/decisions/`, or a `governs` edge from HW-PD-0019, would state it where a reader of the shelf meets it.
- The header of `tools/ci/toolchain.sh` says that five workflows carry the same fallback inline: `release.yml`, `publish-crates.yml`, `release-taxonomy.yml`, `deploy-site.yml` and `preview-site.yml`. Each checks out a ref that can predate the script. No check holds the five copies against the script, so they can drift.

**The n8n fixture scripts.**

- `tools/taxonomy/drive_n8n.py` now also guards `docs/evaluations/n8n-worked-example.md` (#1452). It and `tools/taxonomy/n8n-fixtures.sh` sit in governed scope, and no document declares `governs` onto either.
- `docs/evaluations/n8n-worked-example.md` says in prose that each fixture README states its check-instance count and that the n8n fixture job compares it. It declares no `traces_to` onto `tools/taxonomy/drive_n8n.py` or onto the three fixture READMEs.

**The agents of the build order.**

- A round-1 `hw-verify` of #1366 wedged for four hours. A foreground Bash call with a timeout of 200000 ms, made at 2026-10-01T15:07Z, never returned a result, and no process survived. A message sent to the agent sat unread until the parent stopped the task and the iterate loop resumed it by id. A verifier that probes a hang should wrap every call that touches a FIFO in `timeout`. A check-in that finds an unanswered tool call as an agent's last record should stop the task and resume it.

## Discharge

Each item discharges alone, when the file it names says or does what the item asks, or when a change records why it stays. The record discharges when every item has. An item that grows a reader outside this repository leaves this record for an issue, and the record says where it went.
