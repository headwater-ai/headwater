---
doc_type: guide
id: GD-DIF-no-summary
status: current
status_since: 2026-03-06
---

# A guide with no summary, on a discriminated shelf

The kind here comes from `doc_type` rather than from the directory, so the schema reaches this document only through the branch that its discriminator selects. A branch that selected the wrong kind still reports a missing facet, so the differential compares the facet each side names and not only the count.
