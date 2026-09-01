---
# Added by Headwater. Everything after the closing `---` and the blank line
# below it is the pinned upstream file, byte for byte.
# Upstream: headwater-ai/n8n, master, b0550cb3cb4d1752546a69056c55eccfb9111a12,
# .agents/review-rules/security/credentials-and-secrets.md
id: N8N-STD-credentials-and-secrets
status: current
status_since: 2026-09-01
last_verified: 2026-09-01
summary: The ways credential material reaches a log, an error message or a reader without the scope to see it, and what a reviewer flags.
provenance:
  warrant: asserted
  agency: mixed
  evidence_basis: evidenced
---

# Credentials and secrets

Applies to: all backend packages.

Flag:

- Credentials logged or exposed in error messages
- Hardcoded secrets, API keys, or tokens
- Changes to credential encryption/decryption that weaken security
- OAuth state/CSRF token handling that bypasses validation
- Webhook requests that don't sanitize auth cookies
- External secrets provider integrations with insecure configurations
- Credential access not respecting scope boundaries (instance/project/user)
- Data fetched via credentials being exposed to unauthorized user groups

An error message that interpolates a caught error from an integration is a
common way credential material reaches a persisted status field or an API
response. Sanitize before recording.
