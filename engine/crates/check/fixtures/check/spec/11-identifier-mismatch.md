---
id: SPEC-XX-mismatch
doc_type: design_spec
status: current
status_since: 2026-03-01
summary: The failing fixture for the identifier pattern, with a namespace that its scheme does not declare.
---

# Identifier mismatch

Its kind mints under `spec_id`, and that scheme declares the pattern `SPEC-{namespace}-{slug}` with the namespace `FIX`. This document writes `XX` instead, so the identifier instance fails and the finding names the segment that stopped the match.

The namespace is the part of a pattern that an adopter overlay may not change. An identifier travels into a commit message, a ticket and an agent prompt. A corpus can rewrite its own documents and it can never rewrite somebody else's ticket. So a wrong namespace is the one lexical error in an identifier that nothing later repairs.

The finding reports at the `id` key, which is the line an author edits. It offers no fix: the pattern states the shape of a replacement and never its content.
