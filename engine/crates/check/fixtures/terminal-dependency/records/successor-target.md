---
id: NOTE-FIX-half-written
status: deprecated
status_since: 2026-08-01
relations:
  superseded_by:
    - NOTE-FIX-successor
summary: the only half of its pair, written from the target end
---

# The document that wrote its own half

The relation runs from `NOTE-FIX-successor` to this document, and this file is
the only one that wrote anything. A rule that read the declaring document as the
source reports the wrong pair, or reports nothing at all.
