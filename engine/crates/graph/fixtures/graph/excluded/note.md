---
id: SPEC-FIX-excluded
doc_type: design_spec
---

# A file the corpus declares is not corpus content

The census reports a row for it and reads nothing. A `governs` anchor onto it still resolves, because the file is there and write-time impact detection fires on the file. What the anchor carries beside the hit is the rule that excluded it.
