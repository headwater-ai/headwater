---
id: GS-FIX-hooks
status: current
status_since: 2026-01-05
last_verified: 2026-09-24
summary: governs the hooks a commit runs
relations:
  governs:
    - to: hooks/pre-commit
      verified_revision: "sha256:35aa3219a1de823ddb43637cebb8bb1b28f256956aaff107934f9a2d3e21bfd1"
    - to: hooks/merge-regenerate
      verified_revision: "sha256:0"
    - hooks/write.sh
    - to: hooks
      verified_revision: "sha256:0"
    - [lib, hooks/extra.sh]
    - to: hooks/*.sh
      verified_revision: "sha256:0"
    - to: [lib/lib.sh, lib/other.sh]
      verified_revision: "sha256:1f7f927bf50422b96d641c7ad8948b5587b230c02d67492822d28b7de8d4a075"
---

# Governs the hooks

The recorded corpus of `tests/editions.rs`. Each `governs` entry reaches one arm of `relation.target.suspect`: a recorded digest that matches, a moved digest, an unrecorded file, a directory literal, a list with a directory member, a moved wildcard, and a list whose digest matches. Do not edit a byte here without re-recording `editions.ledger`.
