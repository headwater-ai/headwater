---
id: HW-OBL-0186
status: discharged
status_since: 2026-09-21
summary: "HW-DR-0079 rules the templates half, and criterion 4 now says a facet-only entry's corpus borrows a kind and demonstrates the facet, not the kind."
last_verified: 2026-09-21
title: "A facet-only entry has no kind, and the entry anatomy asks for two things that presume one"
waiting_on: ruling
relations:
  traces_to:
    - HW-DR-0079
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
---

# A facet-only entry has no kind, and the entry anatomy asks for two things that presume one

## Context

The [`diataxis` entry](../taxonomies/diataxis/doctrine.md#findings) is the first facet-only entry of the canonical library. It declares facets and a shelf discriminator, and it declares no concrete kind. [#2](https://github.com/headwater-ai/headwater/issues/2) settles that such an entry is admissible. Criterion 5 names it as "a facet-only overlay like Diátaxis that composes onto a base rather than standing alone".

The [entry anatomy](../taxonomies/README.md#what-an-entry-ships) asks every entry for two things that presume a concrete kind. No criterion says what a facet-only entry puts in either one. This record carries the third and the fourth finding of that entry, at [line 134](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/diataxis/doctrine.md#L134) and [line 136](https://github.com/headwater-ai/headwater/blob/5fb9518/docs/taxonomies/diataxis/doctrine.md#L136). The argued form stays in the doctrine, and this record carries the pointer.

**The worked corpus borrows its kinds.** Criterion 4 asks every entry for a worked instance corpus. The `diataxis` fixture corpus has no kind of its own to type a document as. It takes `decision` and `specification` from the base instead. Every mode page in it is therefore a `specification`, and it carries the `Scope` and `Behavior` headings of the base. A tutorial page does not take those two headings naturally. So the corpus demonstrates the base's kinds with this entry's facets on them, and no criterion reads that case.

**The `templates/` slot has no stated content.** The anatomy asks for one template per concrete kind. An entry with no concrete kind has no kind for a template to shape. `diataxis` leaves the directory absent, which is a reading of the anatomy rather than a rule of it. Criterion 2 asks for the whole anatomy, so the reading and the criterion disagree. The entry merged on the reading.

## Obligation

**The criterion-2 half is answered.** [HW-DR-0079](../decisions/0079-criterion-5-admits-naming-or-extending-a-core-serving-kind-and-a-facet-only-entry-ships-no-templates-directory-under-criterion-2.md) rules that a facet-only entry that adds no concrete kind ships `templates/` absent entirely. It also rules that an entry that extends a base kind it did not add may ship a template fragment for that kind. The [library index](../taxonomies/README.md#what-an-entry-ships) carries both sentences now.

**The criterion-4 half stays open.** The owner still owes a sentence in criterion 4 of the library index. Criterion 4 asks every entry for a worked instance corpus. The `diataxis` fixture corpus has no kind of its own to type a document as, and it takes `decision` and `specification` from the base instead. Every mode page in it is therefore a `specification`. It carries the `Scope` and `Behavior` headings of the base, which a tutorial page does not take naturally. Whoever rules on criterion 4 states whether that borrowing counts as a demonstration of the base's kinds or as a borrowing the anatomy must name.

The same hole in criterion 4 is the half of [#520](https://github.com/headwater-ai/headwater/issues/520) that decision [HW-DR-0079](../decisions/0079-criterion-5-admits-naming-or-extending-a-core-serving-kind-and-a-facet-only-entry-ships-no-templates-directory-under-criterion-2.md) does not reach. That decision rules criteria 2 and 5. It does not rule criterion 4. This record states the gap as an obligation of the corpus rather than as an issue number. A reader who follows a citation to an open issue arrives at the question rather than at the answer.

**Nothing measures criterion 4's slot.** Criteria 3, 6 and 7 became mechanical when the resolver landed. Criterion 4 reads a directory listing, and no verb reads one. A reviewer reads the anatomy by hand against the entry in front of them. So a second facet-only entry takes the same reading again, with no record that the first one took it.

## Discharge

**The criterion-2 half discharged on 2026-09-21.** [HW-DR-0079](../decisions/0079-criterion-5-admits-naming-or-extending-a-core-serving-kind-and-a-facet-only-entry-ships-no-templates-directory-under-criterion-2.md) states what a facet-only entry ships under criterion 2, with the reason, and the library index carries it.

**The criterion-4 half stays open.** This record discharges the rest of it when the library index states what a facet-only entry's worked corpus counts as under criterion 4. The worked corpus of a facet-only entry types every page under a kind that another entry owns. The section contract of that kind then reaches the page. Whoever rules on criterion 4 states whether that is a demonstration or a borrowing. A ruling that calls it a borrowing needs a form for a corpus that demonstrates facets alone.

## Discharge, 2026-09-21

**The criterion-4 half is ruled, and this record closes.** It is a borrowing, not a demonstration of the borrowed kind. The [library index](../taxonomies/README.md#admission-criteria) now states criterion 4's own reading for a facet-only entry's worked corpus. The corpus types pages under a kind it borrows from another entry, and it names that kind. It demonstrates the entry's own facet, not the borrowed kind's purpose.

The `diataxis` entry is the case this record measured. Its fixture corpus types every mode page as `specification`, and it carries the base kind's `Scope` and `Behavior` headings. It demonstrates the `diataxis` facets those pages carry, not the `specification` kind's own purpose. A reviewer could already read this in the entry's own doctrine. The library index now states it as a reading of criterion 4 itself. So a second facet-only entry meets a stated form, not an unread gap.
