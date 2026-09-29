---
id: NOTE-FIX-k
relations:
  governs:
    - ./self-target/notes/../notes/k.md
summary: an explanation that governs its own file through a path it did not write in canonical form
---

# an explanation that governs its own file through a path it did not write in canonical form

The pattern spells this file with a leading `./` and a `..` segment. The resolver normalizes it to this file's own path, so the entry is the case of `notes/g.md` and is reported. A rule that compared the spelling the author wrote would miss it.
