---
id: SPEC-FIX-unshelved
---

# A document that no shelf claims

It carries an identifier and the census gave it no kind, so it is not a node. It is in the index all the same, because an edge that names it is a defect in this file rather than in the edge, and telling those two apart is the whole reason the second shelf of the index exists.

One file, two spellings: [the one whose name holds a space](a%20spaced.md) is `a spaced.md` on the disk, and [the one whose name holds the escape](a%20literal.md) is a different file whose name is the three characters themselves. Both resolve, and neither reading of a destination alone would bind both.

And a third destination, [one whose escape names nothing](a%20missing.md), where no file stands under either reading. It is reported, because a decode that swallowed a real miss on its way past would be worse than the false positive it removes.

A query string is not part of a filename: [a destination that carries one](../spec/00-first.md?v=2) names the file its path names, and [a query on a path that stands nowhere](no-such-note.md?v=2) is still reported, with no query string inside the name it reports.

A destination that opens with a separator is written from the root of the corpus: [one that names a file there](/graph/spec/01-second.md) binds to that file, and [one that names nothing there](/graph/notes/no-such-root-file.md) is reported as the author wrote it, with none of this document's own directory in front of it.
