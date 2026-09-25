---
id: DR-FIX-0003
title: Later decision
status: current
status_since: 2026-03-01
last_verified: 2026-03-15
summary: the second document on the `decisions` shelf, whose two dates are both later than the first document's, so a fold over the read set answers differently as a minimum and as a maximum.
---

# Later decision

The second document on the `decisions` shelf of this fixture tree. It declares no relation, so it sets no state on any target. Its dates are what hold the folds apart: the date a state was entered, with no edge setting the state, is the newest over the read set (`2026-03-01`), and freshness is the stalest (`2026-02-01`, from the first document).
