---
id: SPEC-FIX-first
doc_type: design_spec
relations:
  cites_evidence:
    - EVAL-FIX-alpha
    - to: EVAL-FIX-beta
      cue: the retry budget, not the error taxonomy
  governs:
    - graph/spec/01-second.md
    - ./graph/spec/./01-second.md
    - graph/excluded/note.md
    - graph/no-such-file.rs
---

# The well-formed document

Both authored forms of an entry are here. `EVAL-FIX-alpha` is the scalar sugar, and `EVAL-FIX-beta` is the mapping with an instance attribute. Q4 says the two produce the same edge, so the only difference between them below is the attribute.

The two `governs` targets that start with `graph/spec` are one file spelled two ways. A resolver that made two nodes of them would not fail: it would report two edges, and every check over the second would pass on a node nobody governs.

A [prose link to the second document](01-second.md#a-heading) resolves to a corpus file. A [link into the excluded directory](../excluded/note.md) resolves to a file the corpus declared is not corpus content, which is not the same thing as a broken link. A [link to a file that is not there](../no-such-file.md) is a defect. A [link out of the repository](https://w3id.org/headwater/) is never followed, and a [link to this document](#the-well-formed-document) points at itself.

> Another author wrote this sentence, and the [link inside it](../no-such-file.md) is their reference rather than this document's. Binding it would attribute a reference to the wrong document, so it is counted and skipped.
