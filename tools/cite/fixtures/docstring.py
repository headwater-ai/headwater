"""What a scanner sees when a string outlives its own line.

This docstring writes the citation shape as an example, the way the checker's
own module docstring does:

    # per HW-DR-0049 (docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)

and writes it again inline, as `x = 1  # per HW-DR-0049 (docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)`.

Neither is a citation. Both are prose about the shape. A scanner that resets
its quote state at every newline reports both, and the first version of this
checker did exactly that against its own source.
"""

# This one is outside the docstring and is the only citation in the file.
# per HW-DR-0049 (docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)
value = 1
