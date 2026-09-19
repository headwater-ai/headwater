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

**Every line of a command block is split into the pieces a shell would
actually run, and every piece is checked on its own.** A line is not one
command: `&&`, `||` and `;` sequence several, and `|` pipes one into
another, and a reader who pastes the line runs every piece the line names.
`cd tools && ./malicious-chain.sh` and `mkdir x; npm install` are two
commands apiece, and the second one of each is the one that matters. A
piece passes only if its leading word is `headwater`, or one of the
handful of ordinary shell verbs this repository's own commands already use
for scaffolding and installation (`git`, `mkdir`, `cd`, `printf`, `rm`).
`cargo` passes only for the two subcommands this corpus actually runs,
`install` and `build`: `cargo run --manifest-path
tools/evil/Cargo.toml` is `cargo`, but it is not either of those two, and a
subcommand allowlist is what a wholesale one misses. A line that contains
`$(`, a backtick, `<(` or `>(` fails outright and is never split further:
substitution can hide an arbitrary command inside an argument to an
otherwise-allowed one, and no piece of this corpus's real command blocks
ever needs it.

**The one declared exception is matched as the two-piece pipeline it is,
not as a substring of the line.** `tools/headwater-bootstrap.sh`'s network
fetch — legitimate for as long as `headwater taxonomy vendor` takes a path
and not a location — is a `curl`/`wget` naming that script piped into a
bare `sh`/`bash`, and it is recognized only when a line's pipe-split
resolves to exactly that shape, with neither piece carrying a substitution
of its own. Chaining a second command after it (`... | sh -s -- ... &&
rm -rf /`) puts that second command in its own piece, checked exactly like
any other: the exception excuses the fetch, and it excuses nothing chained
beside it.

**Which blocks are commands is knowledge this script borrows rather than
re-derives.** `README.md` has no output blocks at all — both of its fenced
blocks are commands, so every block there is checked. The tutorial states
its own convention in *Before you start*: "A block is a command or it is
output, and the two look the same" — nothing marks the difference except
which order the page tells it in, so `drive.py`'s `COMMAND_BLOCK_INDICES`,
which `main()` now checks against what it actually ran before it exits, is
the one place that ordering is recorded, and this script imports it rather
than guessing a second time.

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
# itself, with no subcommand restriction: `git` (init, add, commit,
# checkout, clone, and a future `config`), `mkdir`/`cd` (scaffolding a fresh
# repository), `printf` (writing the seed document), `rm` (removing it
# again in step 8). `cargo` is handled separately, by subcommand.
ALLOWED_LEADING_WORDS = frozenset({'headwater', 'git', 'mkdir', 'cd', 'printf', 'rm'})

# `cargo` passes only for the subcommands this corpus's real command blocks
# actually run. `cargo run --manifest-path <anything>` compiles and executes
# arbitrary code from wherever that manifest points, which a wholesale
# allowance for the verb `cargo` would let through.
CARGO_ALLOWED_SUBCOMMANDS = frozenset({'install', 'build'})

# A curl/wget fetch naming the bootstrap script, and a bare shell taking its
# output — the two pieces the one declared exception's pipe-split resolves
# to, checked as two anchored prefixes rather than as one line-wide pattern
# so that nothing chained beside them borrows the exception.
BOOTSTRAP_FETCH_PIECE = re.compile(r'^(curl|wget)\b.*headwater-bootstrap\.sh')
BOOTSTRAP_SHELL_PIECE = re.compile(r'^(sh|bash)\b')

# Command substitution: `$(...)`, a backtick pair, or process substitution.
# Any of these can hide an arbitrary command inside an argument to a piece
# that otherwise reads as allowed, and no real command block in either
# document ever needs one.
SUBSTITUTION = re.compile(r'\$\(|`|<\(|>\(')


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


def _split_unquoted(text, is_boundary):
    """Split `text` at every point `is_boundary(text, i)` accepts, except
    inside a single- or double-quoted span.

    `is_boundary` is called with the full string and an index, and returns
    the number of characters the boundary consumes (0 where none starts
    there). Shared by the chain split (`&&`, `||`, `;`) and the pipe split
    (`|`), so a quoted argument that happens to hold one of those
    characters — `printf`'s literal string is the one line in this corpus
    that could otherwise be cut in half — is never split.
    """
    pieces, buf, quote, i = [], [], None, 0
    while i < len(text):
        ch = text[i]
        if quote:
            buf.append(ch)
            if ch == quote:
                quote = None
            i += 1
            continue
        if ch in ('"', "'"):
            quote = ch
            buf.append(ch)
            i += 1
            continue
        consumed = is_boundary(text, i)
        if consumed:
            pieces.append(''.join(buf))
            buf = []
            i += consumed
            continue
        buf.append(ch)
        i += 1
    pieces.append(''.join(buf))
    return [p.strip() for p in pieces if p.strip()]


def split_chain(line):
    """The top-level sequencing, conditional and backgrounding operators
    split a line into the commands a shell actually runs: `&&`, `||`, `;`,
    and a bare `&`. A single `&` backgrounds the command before it and
    starts the next one immediately rather than waiting on it, which is a
    weaker guarantee than `&&` gives but still runs both — `git clone <url>
    & sh tools/evil.sh` runs the script exactly as `git clone <url> && sh
    tools/evil.sh` does, so the two-character check for `&&` above must
    come first, or a real `&&` would be split as if it named two bare `&`."""
    def boundary(text, i):
        if text[i:i + 2] in ('&&', '||'):
            return 2
        if text[i] in (';', '&'):
            return 1
        return 0
    return _split_unquoted(line, boundary)


def split_pipe(segment):
    """A single `|` splits one command's output into the next command's
    input — two pieces, each checked as its own command."""
    def boundary(text, i):
        return 1 if text[i] == '|' else 0
    return _split_unquoted(segment, boundary)


def is_declared_exception(segment):
    """Whether `segment`, as a whole, is the one declared exception: a
    fetch naming the bootstrap script piped into a bare shell, with neither
    piece carrying a substitution of its own."""
    pieces = split_pipe(segment)
    return (len(pieces) == 2
            and BOOTSTRAP_FETCH_PIECE.match(pieces[0])
            and BOOTSTRAP_SHELL_PIECE.match(pieces[1])
            and not SUBSTITUTION.search(pieces[0])
            and not SUBSTITUTION.search(pieces[1]))


def check_piece(piece):
    """Whether one already-split piece is an allowed command."""
    if SUBSTITUTION.search(piece):
        return False
    tokens = piece.split()
    if not tokens:
        return True
    if tokens[0] == 'cargo':
        return len(tokens) > 1 and tokens[1] in CARGO_ALLOWED_SUBCOMMANDS
    return tokens[0] in ALLOWED_LEADING_WORDS


def undeclared_pieces(block):
    """Every piece of every line of a command block that is not allowed."""
    failures = []
    for line in block.split('\n'):
        stripped = line.strip()
        if not stripped or stripped.startswith('#'):
            continue
        for segment in split_chain(stripped):
            if is_declared_exception(segment):
                continue
            for piece in split_pipe(segment):
                if not check_piece(piece):
                    failures.append(piece)
    return failures


# Regression cases: every attack shape a verifier has raised against this
# check, held here so a future edit that reopens one of them fails a run of
# `fixtures.sh` on its own, rather than waiting for another round of attack.
# Each entry is (label, block text, the exact undeclared pieces expected).
REGRESSION_CASES = [
    ('a bare undeclared command',
     'npm install',
     ['npm install']),
    ('an undeclared script beside an incidental mention of the exception',
     'tools/malicious-script.sh --do-it\n'
     '# see tools/headwater-bootstrap.sh for comparison',
     ['tools/malicious-script.sh --do-it']),
    ('an undeclared script chained with &&',
     'cd tools && ./malicious-chain.sh --do-it',
     ['./malicious-chain.sh --do-it']),
    ('an undeclared command chained with ;',
     'mkdir x; npm install',
     ['npm install']),
    ('an undeclared script chained with && after git',
     'git clone https://example.com/repo.git && sh tools/y.sh',
     ['sh tools/y.sh']),
    ('cargo allowed only for install/build, not by verb alone',
     'cargo run --manifest-path tools/evil/Cargo.toml',
     ['cargo run --manifest-path tools/evil/Cargo.toml']),
    ('a command chained onto the declared exception with &&',
     'curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/'
     'main/tools/headwater-bootstrap.sh | sh -s -- --tag x --expect y '
     '&& npm install',
     ['npm install']),
    ('an undeclared script backgrounded with a bare &',
     'git clone https://example.com/repo.git & sh tools/evil.sh',
     ['sh tools/evil.sh']),
    ('the real declared exception, alone, is not undeclared',
     'curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/'
     'main/tools/headwater-bootstrap.sh | sh -s -- --tag '
     'taxonomy/headwater-standard/v4.2.0 --expect sha256:961ecf2ae2c3c74',
     []),
    ('cargo install, the README route, is not undeclared',
     'cargo install headwater-cli',
     []),
    ('the README source-build block is not undeclared',
     'git clone https://github.com/headwater-ai/headwater.git\n'
     'cd headwater\n'
     'git checkout v0.1.2\n'
     'cargo build --release -p headwater-cli --manifest-path '
     'engine/Cargo.toml --locked',
     []),
    ('step 1 scaffolding is not undeclared',
     'mkdir -p ~/headwater-tutorial/docs/decisions\n'
     'cd ~/headwater-tutorial\n'
     'git init\n'
     "printf '# Store attempts in Postgres' > docs/decisions/postgres-note.md",
     []),
]


def run_regression_cases():
    """Every case above, checked against the live functions rather than
    against a snapshot of their output. Returns the labels that failed."""
    failed = []
    for label, block, expected in REGRESSION_CASES:
        got = undeclared_pieces(block)
        ok = got == expected
        print(('ok   ' if ok else 'FAIL ') + 'regression: ' + label)
        if not ok:
            failed.append(label)
            print(f'    expected {expected!r}, got {got!r}')
    return failed


def main():
    root = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.getcwd()

    regression_failures = run_regression_cases()

    readme_blocks = fenced_blocks(os.path.join(root, README))
    tutorial_blocks = fenced_blocks(os.path.join(root, TUTORIAL))

    checked = 0
    failures = []
    for block in readme_blocks:
        checked += 1
        for piece in undeclared_pieces(block):
            failures.append(f'{README}: undeclared command: {piece!r}')
    for index in drive.COMMAND_BLOCK_INDICES:
        checked += 1
        for piece in undeclared_pieces(tutorial_blocks[index]):
            failures.append(f'{TUTORIAL}: undeclared command: {piece!r}')

    print(f'{len(REGRESSION_CASES)} regression case(s), {len(regression_failures)} failed; '
          f'{checked} command block(s) checked across 2 documents, '
          f'{len(failures)} undeclared command(s)')
    for failure in failures:
        print(' - ' + failure)
    return 1 if (failures or regression_failures) else 0


if __name__ == '__main__':
    sys.exit(main())
