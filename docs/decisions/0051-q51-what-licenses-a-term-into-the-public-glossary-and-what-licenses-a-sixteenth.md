---
id: HW-DR-0051
status: current
status_since: 2026-09-06
summary: "A term reaches the public glossary because it recurs in the visitor-facing pages of the site, and for no other reason. The tutorial and the glossary itself are not sources, and the specification glossary answers to a different reader."
last_verified: 2026-09-06
title: "Q51 — What licenses a term into the public glossary, and what licenses a sixteenth"
provenance:
  warrant: asserted
  agency: agent
  drafted_by: claude-opus-5
  activity: measure+draft
  evidence_basis: evidenced
relations:
  traces_to:
    - HW-DR-0037
    - notes/website-design-brief.md
---

# Q51 — What licenses a term into the public glossary, and what licenses a sixteenth

## Context

**Two glossaries exist, and they answer to different readers.** `docs/spec/glossary.md` is scoped to a person who already files documents in this corpus. Its own text says that an author meets eight terms and a taxonomy author meets roughly thirty-five, out of about 175. Most of those belong to the engine, the check layer or the publisher. The page at `/glossary/` is scoped to a visitor who has read none of the documentation and is deciding whether to spend an hour.

**The page carries fifteen terms today.** They are Census, Check, Corpus, Facet, Finding, Front matter, Governs, Graph, Kind, Obligation, Overlay, Projection, Relation, Shelf and Taxonomy. Section 9 of [the design brief](../../notes/website-design-brief.md) states the basis on which that set was drawn, and no record held it.

**A basis matters here more than the list does.** A glossary with no rule for admission grows by feeling. The reader it serves is the one who can least afford a page of terms that the site does not use.

## Decision

**A term is licensed by recurrence in the visitor-facing pages of the site, and by nothing else.** The sources are the hand-built pages that address a visitor: `site/index.html`, `site/how-it-works/index.html`, `site/compare/index.html`, `site/proof/index.html`, `site/ns/index.html` and `site/changelog/index.html`. A term that appears more than once across them is admitted. Editorial judgment about what sounds important admits nothing.

**The tutorial is not a source.** It is a walkthrough that defines its vocabulary inline, as the reader meets each term. Restating that vocabulary on the glossary page repeats the defect the removed terms page had against the Terms section of `/ns/`. That defect is a second page competing with a first one that already says the thing.

**The glossary itself is not a source.** A page cannot license its own contents, and counting its definitions as occurrences admits every term it already carries.

**A term defined in the one sentence that uses it is left out.** A definition beside a definition is not a lookup that a visitor is missing. `ABox` and `TBox` are each measured at one occurrence, both on `/how-it-works/`, and both are defined where they appear.

**A term the source pages do not use is left out, whatever the specification makes of it.** `regime` is measured at zero occurrences across the source pages. Admitting it would be a guess about what a visitor needs rather than a reading of what the site says. `warrant` is measured at zero occurrences as well.

**A sixteenth term needs the same measurement that the first fifteen needed.** Count the occurrences in the source pages above. A page that the site gains joins the sources, and a page it loses leaves them.

## Consequences

**Nothing checks this.** `.headwater/corpus.json` names `docs` as the one corpus root, so no rule of this engine reads a byte under `site/`. The page is held by [HW-DR-0037](0037-q37-which-parts-of-the-site-are-hand-built-and-which-are-a-projection-of-this-corpus.md), which governs it, and by this record, which states what belongs on it. A term added with no measurement behind it is caught by a reader or by nobody.

**The measurement moves as the site does.** `site/changelog/index.html` did not exist when the fifteen were drawn, and it is a source now. Every term it uses more than once is already on the list, so the count it changes is zero. That reading is what a seventh page has to be given as well.

**The specification glossary is unaffected.** It keeps its own scope, its own reader and its own 175 terms. Neither page is a subset of the other, and neither one is the place to look for what the other holds.
