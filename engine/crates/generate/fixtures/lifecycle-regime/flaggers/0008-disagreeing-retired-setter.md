---
id: DR-FIX-0008
title: Disagreeing retired setter
status: current
status_since: 2026-03-20
summary: the document that sets the clash notice to retired, a state the notice does not stand at.
relations:
  flags_clash_retired:
    - CL-FIX-clash
---

# Disagreeing retired setter

Its state loses to `current` in sorted order, so its date is not the date the clash notice entered its state. It is the oldest date of the three setters on purpose.
