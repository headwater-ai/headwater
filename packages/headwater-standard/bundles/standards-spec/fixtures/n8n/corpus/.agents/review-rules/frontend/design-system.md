---
# Added by Headwater. Everything after the closing `---` and the blank line
# below it is the pinned upstream file, byte for byte.
# Upstream: headwater-ai/n8n, master, b0550cb3cb4d1752546a69056c55eccfb9111a12,
# .agents/review-rules/frontend/design-system.md
id: N8N-STD-design-system
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: The enforcement level a reviewer applies to n8n's frontend design tokens, which the linter validates by name and never by value.
provenance:
  warrant: asserted
  agency: mixed
  evidence_basis: evidenced
---

# Design system enforcement

Applies to: `packages/frontend`.

The design system skill linked alongside this file is the source of truth for
which token to reach for. This file sets the enforcement level.

Stylelint validates CSS custom-property *names* but never their values, so
nothing catches a hard-coded one. Never comment on the naming itself — that
half already fails the build.

- Strong warning: hard-coded visual values (px, rem, hex colours, durations)
  where a token exists; legacy token usage; deprecated style or component
  surfaces.
- Soft warning: token-to-token substitutions. Ask for intent rather than
  asserting a regression — a deliberate change looks identical to a mistake.
