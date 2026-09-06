---
id: HW-OBL-0163
status: current
status_since: 2026-09-06
summary: "Ten decision records name a Q number in their file name that docs/spec/09-decisions.md has no section for, and nothing reads either side of the pair."
last_verified: 2026-09-06
title: "Ten decision records claim a Q number the decision register does not carry"
waiting_on: ruling
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# Ten decision records claim a Q number the decision register does not carry

## Context

`docs/spec/09-decisions.md` is the decision register of this repository, and its sections are keyed on a question number. It carries 36 of them, and they run Q1 to Q32 and then Q40 to Q43.

Ten decision records name a question number in their file name that the register has no section for. Enumerated on 2026-09-06 by taking the two sets and comparing them: `Q33 Q34 Q35 Q36 Q37 Q38 Q39 Q44 Q50 Q51`. The reading is a set difference rather than a sample:

    comm -13 <(grep -oE '^## Q[0-9]+' docs/spec/09-decisions.md | sed 's/## //' | LC_ALL=C sort -u) \
             <(ls docs/decisions/ | grep -oE '^[0-9]{4}-q[0-9]+' | sed 's/^[0-9]*-q/Q/' | LC_ALL=C sort -u)

**The two sides answer to nothing.** A Q number in a file name is not a facet, so no rule reads it. A section heading of the register is prose, and `section.required.missing` reads whether a required heading is there rather than which headings a document owes. So neither half of the pair is a declaration, and a strict run has never had anything to say about the gap.

**The convention is already retired for new records, and that is the reason a ruling is owed rather than a repair.** HW-DR-0045, 0046, 0047, 0048, 0052, 0053 and 0054 carry no Q number at all. Each of those was ruled in conversation rather than filed as an open question on the board. The register gains nothing from them, because its sections are Q-keyed. The ten above are the residue of the older convention, and whether they are a gap or a fossil is the question.

## Obligation

A reader who follows a Q number out of a decision record file name reaches nothing. That is the cost, and it is small: the file name is a convenience and the identifier is what a relation names.

The larger cost is that the register asserts a shape it does not hold. It reads as the record of every question this repository has ruled, and it is the record of 36 of them.

## Discharge

A ruling on the Q key, and one of three answers discharges this.

**The register owes a section for each of the ten.** The discharge is prose, plus a rule that reads a Q number out of a file name and holds the register to it.

**The Q key is retired.** The discharge is a rename of the ten file names and of every citation of them. The register then says that its sections cover the questions filed on the board and no others.

**The gap is accepted.** The discharge is one sentence in the register stating the set it covers, so no reader infers a completeness the file does not claim.

Nothing here proposes one. The value of a decision register is a judgment about who reads it, and this record is the measurement rather than the answer.
