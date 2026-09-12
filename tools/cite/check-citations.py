#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the citation comments in a code tree against the corpus that licensed them.

Spec 5 asks generated code to carry a comment naming the document that licensed
the line above it:

    # per HW-DR-0049 (docs/decisions/0049-a-corpus-wide-fold-is-derived-and-never-stored.md)

and it states the purpose that depends on the comment being true: "when
generated output is wrong, the citation says whether the corpus misled the agent
or the agent ignored the corpus". Nothing checked that. A citation that was
true when it was written and quietly went stale reads exactly like one that is
still true, and so does a plain typo in the identifier.

This is the checker. It drives the shipped binary rather than reading the corpus
itself, so it has no second opinion about what a document is: `headwater explain
<id> --json` answers whether an identifier resolves and what warrant stands
behind it, and the `governing_docs_for_path` tool of `headwater mcp` answers
which documents govern the file the comment sits in.

It lives here rather than as a `headwater` subcommand because it checks code and
every verb of that binary checks documents, and because it is meant to be lifted
into a repository of its own, where an adopter runs it over their own tree
against their own corpus. Nothing here reads anything of this repository except
through the binary and the paths on the command line.

# FOUR FINDINGS, AND ONLY ONE OF THEM LETS A BUILD THROUGH

`citation.identifier.unresolved` — the identifier names no document. A typo and
an invention both land here, and the finding says which of the two the corpus
can tell apart: an identifier that nothing carries at all, or one that a
document carries while the census leaves that document untyped.

`citation.path.mismatch` — the identifier resolves, and the path in the
parentheses is not where it lives. A document that is renamed or moved keeps
its identifier and loses its path, which is the commonest way a citation rots,
and the identifier resolving is exactly what hides it.

`citation.governance.stale` — the identifier resolves, and the file the comment
sits in is not in that document's governing set. This is the class the issue was
filed about, and it is the only one that catches a citation that used to be
true.

`citation.warrant.asserted` — the cited document's warrant is `asserted` rather
than `accepted`, so nobody has accepted what the code says it followed. It is
advisory and it never moves the exit status, for the reason spec 5 gives for
impact detection being advisory: a checker whose advisory class fails a build is
a checker an adopter turns off.

# THE LIMIT OF THE STALE CLASS, WHICH THIS TOOL PRINTS RATHER THAN HIDES

`governing_docs_for_path` matches the path against what a `governs` edge
reached, by equality. A document that governs a directory therefore answers for
no file inside it, which is HW-OBL-0104, open and waiting on a ruling. So in a
corpus whose author declared one edge onto a directory rather than one edge per
file, every citation in a file under that directory is a stale finding and every
one of them is wrong. The text report says so under the findings, and the SARIF
rule carries the same sentence, because a user who meets a wall of stale
findings needs to know which of the two things is broken.

# WHAT IT NEEDS

Python 3.8 or later and the `headwater` binary. No third-party module, no
network. It writes no file and it starts one `headwater mcp` server for the
whole run.

    python3 tools/cite/check-citations.py --root <corpus> [--format text|sarif] <path>...

Every path is a file or a directory under the corpus root. The report is keyed
by the path each file has relative to that root, because that is the path a
`governs` edge names.
"""

import argparse
import json
import os
import re
import subprocess
import sys

SARIF_SCHEMA = (
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/"
    "schemas/sarif-schema-2.1.0.json"
)

# The shape spec 5 shows, wherever it sits inside a line comment. The
# identifier carries at least one hyphen, because a single bare word after
# `per` is prose and not an identifier.
CITATION = re.compile(
    r"\bper\s+([A-Za-z][A-Za-z0-9]*(?:-[A-Za-z0-9]+)+)\s+\(([^()\s]+)\)"
)

# One marker opens a line comment in all of the languages this repository and
# its adopters write. A block comment is out of scope and the module docstring
# says so.
MARKERS = ("#", "//", "--")

# A quote that outlives its own line. Nothing else here carries state from one
# line to the next, and this is the reason it has to.
TRIPLES = ('"""', "'''")

# The one sentence a user needs beside a stale finding, because the finding is
# wrong in a corpus that governs by directory. HW-OBL-0104 is the record.
OBL_0104 = (
    "A governing set is matched by path equality, so a document that governs a "
    "directory answers for no file inside it (HW-OBL-0104, open). In a corpus "
    "that declares one edge onto a directory rather than one edge per file, "
    "every stale finding under that directory is a false positive."
)

RULES = {
    "citation.identifier.unresolved": {
        "level": "error",
        "blocking": True,
        "short": "the identifier in a citation comment names no document",
        "help": (
            "Either the identifier is wrong, or the document that carried it "
            "has gone. `headwater explain <id>` is the same read this check "
            "made."
        ),
    },
    "citation.path.mismatch": {
        "level": "error",
        "blocking": True,
        "short": "the path in the citation is not where the identifier lives",
        "help": (
            "A document that was renamed or moved keeps its identifier and "
            "loses its path, so the identifier still resolves and the "
            "parentheses name a file that is gone. `headwater explain <id> "
            "--json` prints the path this check compared against."
        ),
    },
    "citation.governance.stale": {
        "level": "error",
        "blocking": True,
        "short": "the cited document does not govern the file that cites it",
        "help": OBL_0104,
    },
    "citation.warrant.asserted": {
        "level": "note",
        "blocking": False,
        "short": "the cited document is asserted, and nobody accepted it",
        "help": (
            "Advisory, and it never moves the exit status: a checker whose "
            "advisory class fails a build is a checker an adopter turns off. "
            "A routing pointer states an asserted warrant beside its summary "
            "for the same reason, and leaves the reader to decide."
        ),
    },
}


class Finding:
    def __init__(self, rule, path, line, column, text, message, remedy):
        self.rule = rule
        self.path = path
        self.line = line
        self.column = column
        self.text = text
        self.message = message
        self.remedy = remedy

    @property
    def blocking(self):
        return RULES[self.rule]["blocking"]

    def key(self):
        return (self.path, self.line, self.column, self.rule)


# ---------------------------------------------------------------------------
# Finding the citations
# ---------------------------------------------------------------------------


def comments_in(text):
    """Every line comment in one file, as (line number, body start, line).

    A citation inside a string literal is not a citation, so the marker has to
    stand outside every quote. Two kinds of quote matter and they are not the
    same problem.

    A single-line quote is closed by the end of its line. A triple quote is
    not, and that is the one that drew blood: the docstring of this module
    writes an example of the shape, and a scanner that resets its quote state
    at every newline read that example as a live citation and reported an
    identifier it had invented for a comment about itself. The first version of
    this file did exactly that. So the triple-quote state is carried across
    lines and the single-quote state is not, which is how each one behaves.

    This is still not a parser for any one language. A raw string, a heredoc
    and a block comment all defeat it in one direction or the other. What it
    holds is the cases a fixture provokes, and the module docstring states the
    limit.
    """
    triple = None
    for number, line in enumerate(text.splitlines(), start=1):
        quote = None
        i = 0
        start = None
        while i < len(line):
            if triple:
                if line.startswith(triple, i):
                    i += 3
                    triple = None
                else:
                    i += 1
                continue
            ch = line[i]
            if quote:
                if ch == "\\":
                    i += 2
                elif ch == quote:
                    quote = None
                    i += 1
                else:
                    i += 1
                continue
            opened = next((t for t in TRIPLES if line.startswith(t, i)), None)
            if opened:
                closes = line.find(opened, i + 3)
                if closes == -1:
                    triple = opened
                    break
                i = closes + 3
                continue
            if ch in "'\"":
                quote = ch
                i += 1
                continue
            marker = next((m for m in MARKERS if line.startswith(m, i)), None)
            if marker:
                start = i + len(marker)
                break
            i += 1
        if start is not None:
            yield number, start, line


def citations_in(text, path):
    """Every citation in one file, as (line, column, identifier, cited path)."""
    found = []
    for number, start, line in comments_in(text):
        for match in CITATION.finditer(line, start):
            found.append(
                {
                    "path": path,
                    "line": number,
                    "column": match.start() + 1,
                    "id": match.group(1),
                    "cited": match.group(2),
                    "text": line.strip(),
                }
            )
    return found


SKIP_DIRS = {".git", "target", "node_modules", ".headwater"}


def files_under(paths, root):
    """Every file under the paths given, as corpus-relative paths.

    # EVERY REFUSAL HERE EXISTS BECAUSE ITS ABSENCE LOOKED LIKE A PASS

    A scanner reports what it found, so a scanner that found nothing reports a
    clean tree. That makes every way of quietly scanning nothing a way of
    quietly passing, and this function had two of them.

    `os.walk` on a path that does not exist yields no entry and raises nothing.
    So a mistyped argument — a renamed directory, a file moved in the same
    commit that a CI line still names — printed "every citation resolves, and
    every cited document governs the file that cites it" and exited 0. The
    checker refuses a missing engine with exit 2 and refused a missing scan
    target with a clean bill of health, which is the wrong way round: the
    engine is a tool and the target is the evidence.

    `os.walk` also swallows a directory it cannot read. Its default `onerror`
    is `None`, which discards the error and prunes that subtree, so one
    unreadable directory silently removes every file under it from the
    population and the report still says clean.

    Both now raise, and `main` turns either into exit 2 with a sentence. The
    fixture suite provokes the first and, on a process that is not root, the
    second.
    """
    out = []
    for given in paths:
        if not os.path.exists(given):
            raise OSError(f"no such path to scan: {given}")
        if os.path.isfile(given):
            out.append(given)
            continue

        def refuse(error):
            raise error

        for here, dirs, names in os.walk(given, onerror=refuse):
            dirs[:] = sorted(d for d in dirs if d not in SKIP_DIRS)
            for name in sorted(names):
                out.append(os.path.join(here, name))
    relative = []
    for path in out:
        rel = os.path.relpath(os.path.abspath(path), os.path.abspath(root))
        relative.append((path, rel.replace(os.sep, "/")))
    return relative


def read(path):
    """The text of one file, or None where it holds no text at all.

    A binary file carries no citation comment, and declining to decode one is
    not a finding about it. A file this process may not open is a different
    thing: skipping it is a silent hole in the population, so the error travels
    and `main` turns it into exit 2. A symbolic link with nothing on the other
    end reaches here the same way, because `os.walk` lists it and opening it
    fails, and the same refusal is the right answer — the tool was told to read
    something and could not.
    """
    try:
        with open(path, "r", encoding="utf-8", errors="strict") as handle:
            return handle.read()
    except UnicodeDecodeError:
        return None
    except OSError as error:
        raise OSError(f"cannot read {path}: {error.strerror or error}") from None


# ---------------------------------------------------------------------------
# The two reads of the corpus, both through the shipped binary
# ---------------------------------------------------------------------------


def engine_path(given):
    """The binary to drive: the flag, the environment, then the newer build.

    The gate takes the newer of `release` and `dev-release` rather than
    `release` by name, so this takes it the same way; a session that built the
    cheap profile last is checking against the engine it just built.
    """
    if given:
        return given
    if os.environ.get("HEADWATER"):
        return os.environ["HEADWATER"]
    here = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    best = None
    for profile in ("release", "dev-release"):
        candidate = os.path.join(here, "engine", "target", profile, "headwater")
        if os.access(candidate, os.X_OK):
            if best is None or os.path.getmtime(candidate) > os.path.getmtime(best):
                best = candidate
    return best or "headwater"


class Mcp:
    """One `headwater mcp` session for the whole run.

    The server walks the corpus once and answers from those values, so a
    citation checker that started one process per file would pay that walk once
    per file. It is also the only surface that answers
    `governing_docs_for_path` at all: there is no CLI verb for it, which is a
    gap in `docs/interfaces/headwater-mcp.md` and not in this checker.
    """

    def __init__(self, engine, root):
        self.process = subprocess.Popen(
            [engine, "mcp", "--root", root],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        self.next_id = 0
        self.send(
            {
                "jsonrpc": "2.0",
                "id": self.take_id(),
                "method": "initialize",
                "params": {"protocolVersion": "2024-11-05"},
            }
        )
        self.receive()
        self.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    def take_id(self):
        self.next_id += 1
        return self.next_id

    def send(self, request):
        self.process.stdin.write(json.dumps(request) + "\n")
        self.process.stdin.flush()

    def receive(self):
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError("the mcp server closed standard output")
        return json.loads(line)

    def call(self, tool, arguments):
        self.send(
            {
                "jsonrpc": "2.0",
                "id": self.take_id(),
                "method": "tools/call",
                "params": {"name": tool, "arguments": arguments},
            }
        )
        response = self.receive()
        if "error" in response:
            raise RuntimeError(f"{tool}: {response['error']}")
        blocks = response["result"]["content"]
        return "".join(block.get("text", "") for block in blocks)

    def close(self):
        try:
            self.process.stdin.close()
            self.process.wait(timeout=30)
        except Exception:
            self.process.kill()


# `<path> (<name>) — <summary> [asserted: …]`, of which this reads the path
# alone. The renderer is `Pointer::render` in `engine/crates/query/src/lib.rs`
# and every optional part of it opens with a space.
POINTER = re.compile(r"^(.+?)(?: \(| — | \[asserted:|$)")


def governing(mcp, path):
    """The documents that govern one path, as corpus-relative paths."""
    text = mcp.call("governing_docs_for_path", {"path": path})
    if text.startswith("no document governs "):
        return []
    out = []
    for line in text.splitlines():
        if not line.strip():
            continue
        match = POINTER.match(line)
        if match:
            out.append(match.group(1))
    return out


def explain(engine, root, identifier):
    """What the corpus says about one identifier: the document, or the refusal."""
    result = subprocess.run(
        [engine, "explain", identifier, "--json", "--root", root, "--no-color"],
        capture_output=True,
        text=True,
    )
    if result.returncode == 0:
        return {"resolved": True, "document": json.loads(result.stdout)}
    return {"resolved": False, "refusal": result.stderr.strip()}


def carried(mcp, identifier):
    """Whether a document this corpus leaves untyped carries the identifier.

    `headwater explain` refuses a missing identifier through the path matcher,
    so it answers "is outside every corpus root this repository declares" for
    an invented identifier and for a typo alike — the four states its contract
    names classify a path. `resolve_identifier` is the surface that separates
    an identifier nothing carries from one a document carries while the census
    leaves that document untyped, which is the only near miss this engine
    computes.

    # THE TRAP, WHICH THIS FUNCTION FELL INTO ONCE

    The two refusals share a prefix. `Resolved::Nothing` renders "no document
    carries X" and `Resolved::NearMiss` renders "no document carries X, and one
    carries Y". A test of `startswith("no document carries ")` therefore reads
    every near miss as nothing, and the branch below becomes unreachable while
    every case still passes — an absence read as satisfaction, which is the
    shape that keeps finding this repository. The discriminator is the second
    clause and nothing else.

    The engine's sentence is not quoted into the finding, because `near_miss`
    matches an identifier exactly and then reports it as the near one, so Y is
    always X and the sentence names the same identifier twice. That is filed as
    a defect of the engine rather than worked around here.
    """
    text = mcp.call("resolve_identifier", {"id": identifier})
    return ", and one carries " in text


# ---------------------------------------------------------------------------
# The check
# ---------------------------------------------------------------------------


def check(paths, root, engine):
    files = files_under(paths, root)
    citations = []
    for disk, rel in files:
        text = read(disk)
        if text is None:
            continue
        citations.extend(citations_in(text, rel))

    findings = []
    if not citations:
        return citations, findings, files

    mcp = Mcp(engine, root)
    try:
        known = {}
        governs = {}
        for citation in citations:
            identifier = citation["id"]
            if identifier not in known:
                known[identifier] = explain(engine, root, identifier)
            answer = known[identifier]

            if not answer["resolved"]:
                if carried(mcp, identifier):
                    message = (
                        f"`{identifier}` is carried by a document the census "
                        f"leaves untyped, so the graph cannot serve it"
                    )
                    remedy = (
                        "type the document that carries it, or cite one this "
                        "corpus already serves"
                    )
                else:
                    message = f"no document of this corpus carries `{identifier}`"
                    remedy = "correct the identifier, or cite a document that exists"
                findings.append(
                    Finding(
                        "citation.identifier.unresolved",
                        citation["path"],
                        citation["line"],
                        citation["column"],
                        citation["text"],
                        message,
                        remedy,
                    )
                )
                continue

            document = answer["document"]
            cited_path = document.get("path")

            named = citation["cited"]
            if named.startswith("./"):
                named = named[2:]
            if named != cited_path:
                findings.append(
                    Finding(
                        "citation.path.mismatch",
                        citation["path"],
                        citation["line"],
                        citation["column"],
                        citation["text"],
                        f"`{identifier}` lives at `{cited_path}`, and this "
                        f"citation names `{named}`",
                        "write the path the identifier resolves to, because "
                        "the document moved or the path was never right",
                    )
                )

            if citation["path"] not in governs:
                governs[citation["path"]] = governing(mcp, citation["path"])
            set_of = governs[citation["path"]]
            if cited_path not in set_of:
                if set_of:
                    instead = ", ".join(f"`{one}`" for one in set_of)
                    message = (
                        f"`{identifier}` resolves to `{cited_path}`, which does "
                        f"not govern this file. {instead} does."
                    )
                else:
                    message = (
                        f"`{identifier}` resolves to `{cited_path}`, and no "
                        f"document governs this file at all."
                    )
                findings.append(
                    Finding(
                        "citation.governance.stale",
                        citation["path"],
                        citation["line"],
                        citation["column"],
                        citation["text"],
                        message,
                        "cite the document that governs this file, or declare "
                        "the governance edge the citation assumes",
                    )
                )

            if document.get("warrant") == "asserted":
                findings.append(
                    Finding(
                        "citation.warrant.asserted",
                        citation["path"],
                        citation["line"],
                        citation["column"],
                        citation["text"],
                        f"`{cited_path}` is asserted, so nobody has accepted "
                        f"what this code says it followed",
                        "accept the cited document, or read the citation as a "
                        "draft the way a routing pointer asks you to",
                    )
                )
    finally:
        mcp.close()

    findings.sort(key=Finding.key)
    return citations, findings, files


# ---------------------------------------------------------------------------
# The two reports
# ---------------------------------------------------------------------------


def text_report(citations, findings, files, root, out):
    print(
        f"check-citations: {len(citations)} citation"
        f"{'' if len(citations) == 1 else 's'} in {len(files)} file"
        f"{'' if len(files) == 1 else 's'}, corpus {root}",
        file=out,
    )
    if not findings:
        print("", file=out)
        print("  every citation resolves, and every cited document governs "
              "the file that cites it.", file=out)
    for finding in findings:
        print("", file=out)
        print(
            f"  {finding.path}:{finding.line}:{finding.column}  "
            f"{RULES[finding.rule]['level']}  {finding.rule}",
            file=out,
        )
        print(f"      {finding.text}", file=out)
        print(f"      {finding.message}", file=out)
        print(f"      fix: {finding.remedy}", file=out)

    counted = {rule: 0 for rule in RULES}
    for finding in findings:
        counted[finding.rule] += 1
    print("", file=out)
    print(
        f"  {len(citations)} citations, "
        f"{counted['citation.identifier.unresolved']} unresolved, "
        f"{counted['citation.path.mismatch']} misplaced, "
        f"{counted['citation.governance.stale']} stale, "
        f"{counted['citation.warrant.asserted']} asserted",
        file=out,
    )
    if counted["citation.governance.stale"]:
        print("", file=out)
        print(f"  {OBL_0104}", file=out)
    if counted["citation.warrant.asserted"]:
        print(
            "  An asserted citation is advisory and did not move the exit "
            "status.",
            file=out,
        )


def sarif_report(citations, findings, root, out):
    ids = sorted(RULES)
    rules = [
        {
            "id": rule,
            "shortDescription": {"text": RULES[rule]["short"]},
            "help": {"text": RULES[rule]["help"]},
            "properties": {
                "headwater": {
                    "scope": "code",
                    "blocking": RULES[rule]["blocking"],
                }
            },
        }
        for rule in ids
    ]
    results = []
    for finding in findings:
        results.append(
            {
                "ruleId": finding.rule,
                "ruleIndex": ids.index(finding.rule),
                "level": RULES[finding.rule]["level"],
                "kind": "fail",
                "message": {"text": finding.message},
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {"uri": finding.path},
                            "region": {
                                "startLine": finding.line,
                                "startColumn": finding.column,
                            },
                        }
                    }
                ],
                "properties": {
                    "headwater": {
                        "blocking": finding.blocking,
                        "remediation": finding.remedy,
                        "citation": finding.text,
                    }
                },
            }
        )
    document = {
        "$schema": SARIF_SCHEMA,
        "version": "2.1.0",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "headwater-check-citations",
                        "informationUri": "https://github.com/headwater-ai/headwater",
                        "rules": rules,
                    }
                },
                "columnKind": "utf16CodeUnits",
                "results": results,
                "properties": {
                    "headwater": {
                        "root": root,
                        "citations": len(citations),
                    }
                },
            }
        ],
    }
    json.dump(document, out, indent=1, sort_keys=False)
    out.write("\n")


def main(argv=None):
    parser = argparse.ArgumentParser(
        prog="check-citations",
        description="check the citation comments in a code tree against the "
        "corpus that licensed them",
    )
    parser.add_argument("paths", nargs="+", help="files or directories to scan")
    parser.add_argument("--root", default=".", help="the corpus to resolve against")
    parser.add_argument("--engine", default=None, help="the headwater binary to drive")
    parser.add_argument("--format", default="text", choices=("text", "sarif"))
    args = parser.parse_args(argv)

    engine = engine_path(args.engine)
    try:
        citations, findings, files = check(args.paths, args.root, engine)
    except (RuntimeError, OSError) as error:
        print(f"check-citations: {error}", file=sys.stderr)
        return 2

    if args.format == "sarif":
        sarif_report(citations, findings, args.root, sys.stdout)
    else:
        text_report(citations, findings, files, args.root, sys.stdout)

    return 1 if any(finding.blocking for finding in findings) else 0


if __name__ == "__main__":
    sys.exit(main())
