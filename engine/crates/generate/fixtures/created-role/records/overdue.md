---
id: CR-FIX-overdue
status: current
entered: 2026-08-01
filed: 2026-01-05
summary: filed long ago, entered its state recently, and nothing has reviewed it
---

# Overdue by the role the expectation names

`filed` is 219 days before the pinned clock and `entered` is 11 days before it. The expectation declares `since: created`, so a window measured from the role it names reports this document. A window measured from a literal `state_entered` finds 11 days inside a 30-day window and reports nothing at all.
