---
id: AC-PROBE-the-session-reads-the-other-document
status: current
summary: The second well-formed probe, so that the selection has more than one probe and a part of it is not the whole.
probe_category: discovery
expectation: opened
action: ask_or_locate
oracle: "none"
relations:
  examines:
    - AC-PROBE-the-session-reads-the-document
---

# The session reads the other document

## Task

Do the thing the document below governs.

## Target

The document that the key reads.

## Key

`opened` over the document this probe names.
