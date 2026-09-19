#!/usr/bin/env python3
"""Flag a script slipping into adopter-facing prose.

[HW-DR-0072](../../docs/decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md)
rules that an adopter reaches the whole governed loop through the binary,
that git plumbing is the one integration point the necessity test admits,
and that the list grows by a decision record and never by a script. Nothing
checked that boundary before #933: both real violations on record (#929,
the change manifest only `.githooks/change-manifest` could write; #930, the
bootstrap script presented as the only route) were found and fixed by hand,
and both sat in exactly the two documents a newcomer actually reads. This
script re-reads those two documents for the same shape.

A **script invocation** here is a `curl`/`wget` pipeline into a shell, or a
direct call to a path this repository ships under `tools/` or
`.githooks/`. That is deliberately narrower than "any command that is not
`headwater`": the tutorial and the README both use `git`, `cargo`, `mkdir`,
`ls`, `cat` and the rest of a normal shell for scaffolding and installation,
and none of that is the boundary HW-DR-0072 draws. What the decision record
rules out is an adopter reaching for a *script this repository wrote* to do
what a verb already does — the exact shape both #929 and #930 took, and the
only shape either historical violation ever took.

The one declared exception is `tools/headwater-bootstrap.sh`'s network
fetch, which HW-DR-0072 names as legitimate for as long as `headwater
taxonomy vendor` takes a path and not a location. A future git-plumbing
convenience (#892) needs no exception here: a bare `git config` line invokes
no script, so the pattern below never matches one.

Run it through `.claude/tutorial/fixtures.sh`, or directly:
`python3 .claude/tutorial/adopter_interface.py <repo-root>`. Unlike
`drive.py`, it needs no built engine — it only reads two documents.
"""

import os
import re
import sys

DOCS = [
    'README.md',
    'docs/tutorials/your-first-governed-corpus.md',
]

# A curl/wget pipeline into a shell, or a direct invocation of a script this
# repository ships under `tools/` or `.githooks/`.
SCRIPT_INVOCATION = re.compile(
    r'(curl|wget)\b[^\n]*\|\s*(sh|bash)\b'
    r'|(^|[\s`])(\./|tools/|\.githooks/)\S+\.sh\b',
    re.MULTILINE,
)

# The one declared exception (HW-DR-0072, "A network fetch left this list"):
# the bootstrap script fetching the standard taxonomy package over the
# network, for a reader who has no package tree yet.
DECLARED_EXCEPTION = re.compile(r'headwater-bootstrap\.sh\b')


def fenced_blocks(path):
    """Every fenced code block's content, in document order.

    Mirrors `drive.py`'s `read_blocks`: front matter is stripped first so
    that a `---` a document merely quotes (front matter shown inside a
    fence) never reads as a second document boundary.
    """
    lines = open(path).read().split('\n')
    if lines and lines[0] == '---':
        lines = lines[lines.index('---', 1) + 1:]
    blocks, current, inside = [], [], False
    for line in lines:
        if line.strip() == '```':
            if inside:
                blocks.append('\n'.join(current))
                current = []
            inside = not inside
            continue
        if inside:
            current.append(line)
    return blocks


def main():
    root = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.getcwd()
    matches = 0
    failures = []
    for doc in DOCS:
        path = os.path.join(root, doc)
        for block in fenced_blocks(path):
            found = list(SCRIPT_INVOCATION.finditer(block))
            for match in found:
                matches += 1
                line = match.group(0).strip()
                if DECLARED_EXCEPTION.search(block):
                    continue
                failures.append(f'{doc}: undeclared script invocation: {line!r}')

    print(f'{matches} script invocation(s) found across {len(DOCS)} documents, '
          f'{len(failures)} with no declared exception')
    for failure in failures:
        print(' - ' + failure)
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
