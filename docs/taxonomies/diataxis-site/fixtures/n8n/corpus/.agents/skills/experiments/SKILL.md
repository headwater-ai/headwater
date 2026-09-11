---
# Added by Headwater. Everything after this block is the pinned upstream file,
# byte for byte.
# Upstream: n8n-io/n8n, master, b0550cb3cb4d1752546a69056c55eccfb9111a12,
# .agents/skills/experiments/SKILL.md
# n8n's own Agent Skills front matter is below, unchanged. Headwater's keys are
# merged into the same block because a second block would not be read.
name: n8n:experiments
description: >-
  Guides work on `packages/frontend/editor-ui` experiments. Use when creating,
  extending, wiring, testing, reviewing, or retiring editor-ui experiments,
  PostHog feature flags, experiment key indexes, variants, stores/composables,
  persisted experiment state, or experiment telemetry.
id: N8N-HOW-experiments
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: ">-"
provenance:
  warrant: asserted
  agency: mixed
  evidence_basis: evidenced
---

# Experiments

Use this skill for frontend experiment code lifecycle work in `packages/frontend/editor-ui`.

Start with the relevant mode in [reference.md](reference.md):

- `Create` for a new experiment folder, constant, store/composable, and tests.
- `Extend` for new variants, behavior, display logic, or telemetry on an existing experiment.
- `Wire` for host-surface integration through routes, modals, views, or components.
- `Test` for store, composable, persistence, telemetry, and UI behavior coverage.
- `Review` for auditing experiment changes.
- `Retire` for cleaning up completed or abandoned experiments.

When experiment work touches Vue components or user-facing copy, also follow `n8n:design-system` and `n8n:content-design`.
