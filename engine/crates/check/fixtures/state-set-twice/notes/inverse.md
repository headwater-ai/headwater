---
id: NOTE-FIX-inverse
status: superseded
relations:
  retired_by:
    - NOTE-FIX-retired-by-inverse
summary: the target of one setter, and the source of another it wrote from the far end
---

# inverse

`inverse-superseder.md` declares `supersedes` here. This file writes
`retired_by`, the inverse of `retires`, so the relation runs from this document
to `NOTE-FIX-retired-by-inverse`. This document sets a state there and is not
told one. A rule that read the writing file as the target reports a clash here.
