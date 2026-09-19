#!/usr/bin/env python3
"""Flag adopter-facing prose that instructs a command that is not `headwater`.

[HW-DR-0072](../../docs/decisions/0072-the-binary-is-the-only-interface-an-adopter-must-run-and-every-integration-point-outside-it-is-declared.md)
rules that an adopter reaches the whole governed loop through the binary,
that git plumbing is the one integration point the necessity test admits,
and that the list grows by a decision record and never by a script. Nothing
checked that boundary before #933: both real violations on record (#929,
the change manifest only `.githooks/change-manifest` could write; #930, the
bootstrap script presented as the only route) were found and fixed by hand,
and both sat in exactly the two documents a newcomer actually reads. This
script re-reads those two documents for the same shape, on the terms #933's
own Done-when states: it fails on a tree where adopter-facing prose
instructs a command that is not `headwater`.

**A command block's every line is checked, and a line passes only if its
leading word is `headwater`, one of the handful of ordinary shell verbs
this repository's own commands already use for scaffolding and
installation (`git`, `cargo`, `mkdir`, `cd`, `printf`, `rm`), or the one
declared exception** — `tools/headwater-bootstrap.sh`'s network fetch,
which HW-DR-0072 names as legitimate for as long as `headwater taxonomy
vendor` takes a path and not a location. Anything else — `npm install`, a
direct call to a script this repository ships (`tools/*.sh`,
`.githooks/*`), a `curl`/`wget` pipe into a shell that is not that one
exception — fails. A future git-plumbing convenience (#892) needs no
further exception: a bare `git config` line already passes, because `git`
is already an allowed verb.

**Which blocks are commands is knowledge this script borrows rather than
re-derives.** `README.md` has no output blocks at all — both of its fenced
blocks are commands, so every block there is checked. The tutorial states
its own convention in *Before you start*: "A block is a command or it is
output, and the two look the same" — nothing marks the difference except
which order the page tells it in, so `drive.py`'s `COMMAND_BLOCK_INDICES`,
built from the same run it drives, is the one place that ordering is
already recorded, and this script imports it rather than guessing.

Run it through `.claude/tutorial/fixtures.sh`, or directly:
`python3 .claude/tutorial/adopter_interface.py <repo-root>`. Unlike
`drive.py`, it needs no built engine — it only reads two documents.
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import drive  # noqa: E402  (COMMAND_BLOCK_INDICES; see the module docstring above)

README = 'README.md'
TUTORIAL = drive.DOC

# Verbs this repository's own command blocks already use besides `headwater`
# itself: `git` (init, add, commit, checkout, clone, and a future `config`),
# `cargo` (install, build), `mkdir`/`cd` (scaffolding a fresh repository),
# `printf` (writing the seed document), `rm` (removing it again in step 8).
ALLOWED_LEADING_WORDS = frozenset({'headwater', 'git', 'cargo', 'mkdir', 'cd', 'printf', 'rm'})

# The one declared exception (HW-DR-0072, "A network fetch left this list"):
# a `curl`/`wget` pipeline into a shell that fetches
# `tools/headwater-bootstrap.sh` by name, on this line and no other.
BOOTSTRAP_EXCEPTION_LINE = re.compile(
    r'^(curl|wget)\b[^\n]*headwater-bootstrap\.sh[^\n]*\|\s*(sh|bash)\b')


def fenced_blocks(path):
    """Every fenced code block's content, in document order.

    Mirrors `drive.py`'s `read_blocks`: front matter is stripped first so
    that a `---` a document merely quotes (front matter shown inside a
    fence) never reads as a second document boundary.
    """
    if not os.path.isfile(path):
        raise SystemExit(f'adopter interface: no such document: {path}')
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


def undeclared_lines(block):
    """Every line of a command block whose leading word is not allowed.

    Checked one line at a time, and the declared exception is matched
    against the line it would excuse rather than against the block as a
    whole — an undeclared script call does not become declared because
    some other, unrelated line of the same block happens to mention the
    exception's name.
    """
    failures = []
    for line in block.split('\n'):
        stripped = line.strip()
        if not stripped or stripped.startswith('#'):
            continue
        if BOOTSTRAP_EXCEPTION_LINE.match(stripped):
            continue
        first = stripped.split()[0]
        if first in ALLOWED_LEADING_WORDS:
            continue
        failures.append(stripped)
    return failures


def main():
    root = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.getcwd()

    readme_blocks = fenced_blocks(os.path.join(root, README))
    tutorial_blocks = fenced_blocks(os.path.join(root, TUTORIAL))

    checked = 0
    failures = []
    for index, block in enumerate(readme_blocks):
        checked += 1
        for line in undeclared_lines(block):
            failures.append(f'{README}: undeclared command: {line!r}')
    for index in drive.COMMAND_BLOCK_INDICES:
        checked += 1
        for line in undeclared_lines(tutorial_blocks[index]):
            failures.append(f'{TUTORIAL}: undeclared command: {line!r}')

    print(f'{checked} command block(s) checked across 2 documents, '
          f'{len(failures)} undeclared command(s)')
    for failure in failures:
        print(' - ' + failure)
    return 1 if failures else 0


if __name__ == '__main__':
    sys.exit(main())
