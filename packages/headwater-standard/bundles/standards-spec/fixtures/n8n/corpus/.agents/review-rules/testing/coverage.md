---
# Added by Headwater. Everything after the closing `---` and the blank line
# below it is the pinned upstream file, byte for byte.
# Upstream: headwater-ai/n8n, master, b0550cb3cb4d1752546a69056c55eccfb9111a12,
# .agents/review-rules/testing/coverage.md
id: N8N-STD-coverage
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: The bar n8n's reviewer applies to new behavior that ships with no test, and the four things it does not flag.
provenance:
  warrant: asserted
  agency: mixed
  evidence_basis: evidenced
---

# Test coverage

Applies to: any package with a test suite.

Flag new behaviour that ships with no test — a service method, controller
route, node operation, store action, or composable. Coverage does not need to
be complete; core functionality and the critical path do.

Do NOT flag:

- Exports, types, configuration objects, metadata, version files
- A percentage — the number is not the bar, the critical path is
- Edge cases a human reviewer is better placed to ask for
- A skipped test — the `no-skipped-tests` ESLint rule already fails the build
