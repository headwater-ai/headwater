#!/usr/bin/env python3
"""Run every n8n fixture corpus by the commands its own README prints, and diff the result against the figures that README states.

Three READMEs under `docs/taxonomies/*/fixtures/n8n/` each carry a *How to run this corpus* block and a *What a run reports* paragraph. The block is the recipe a reader types; the paragraph is a claim about this engine. Neither re-derives, and both went stale unseen: all three recipes refused at `headwater taxonomy resolve` for four minor versions of the vendored package before anything ran them.

Nothing here carries a second copy of a figure. Every expected value is parsed out of the README, so a number this file holds is a number a reader reads.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

SHIM = """#!/bin/sh
n=$(ls "$HW_LOG" | grep -c '\\.argv$' || true)
i=$(printf '%03d' "$n")
printf '%s\\n' "$@" > "$HW_LOG/$i.argv"
"$HW_BIN" "$@" > "$HW_LOG/$i.out" 2> "$HW_LOG/$i.err"
s=$?
printf '%s\\n' "$s" > "$HW_LOG/$i.status"
cat "$HW_LOG/$i.out"
cat "$HW_LOG/$i.err" >&2
exit $s
"""

# Each entry is (label, regex over the README's *What a run reports* section).
# A label absent from the section is expected to be zero, which is how a corpus
# with nothing excluded states it.
FIGURES = [
    ("files under the corpus root", r"\b(\d+) files under the corpus root\b"),
    ("typed", r"\b(\d+) typed\b"),
    ("excluded", r"\b(\d+) excluded\b"),
    ("checked", r"\b(\d+) checked\b"),
    ("check instances", r"\b(\d+) check instances\b"),
    ("findings", r"\b(\d+) findings\b"),
    ("errors", r"\ball (\d+) of them errors\b|\b(\d+) of them errors\b"),
    ("warnings", r"\b(\d+) warnings\b"),
    ("census kind count", r"The census reads \*{0,2}(\d+)\*{0,2} `\w+`"),
    ("census kind", r"The census reads \*{0,2}\d+\*{0,2} `(\w+)`"),
    ("graph nodes", r"The graph reads \*{0,2}(\d+)\*{0,2} nodes"),
    ("declared edge halves", r"\b(\d+)\*{0,2} declared edge halves\b"),
    ("unresolved prose links", r"\b(\d+)\*{0,2} prose links? that did not resolve\b"),
    ("strict exit status", r"`headwater check --strict` exits \*{0,2}(\d+)"),
]

ZERO_BY_DEFAULT = {"excluded", "warnings"}


class Mismatch(Exception):
    pass


def section(text, heading):
    """The body of one `##` section of a Markdown document, heading excluded."""
    out, taking = [], False
    for line in text.splitlines():
        if line.startswith("## "):
            if taking:
                break
            taking = line[3:].strip() == heading
            continue
        if taking:
            out.append(line)
    if not taking and not out:
        raise Mismatch("no `## %s` section" % heading)
    return "\n".join(out)


def indented_block(body):
    """The first four-space-indented code block of a section, dedented."""
    lines, out, taking = body.splitlines(), [], False
    for line in lines:
        if line.startswith("    "):
            taking = True
            out.append(line[4:])
        elif taking and not line.strip():
            out.append("")
        elif taking:
            break
    while out and not out[-1].strip():
        out.pop()
    if not out:
        raise Mismatch("no indented command block")
    return "\n".join(out)


def stated_figures(readme_text):
    body = section(readme_text, "What a run reports")
    stated = {}
    for label, pattern in FIGURES:
        m = re.search(pattern, body)
        if m is None:
            if label in ZERO_BY_DEFAULT:
                stated[label] = "0"
            continue
        stated[label] = next(g for g in m.groups() if g is not None)
    missing = [l for l, _ in FIGURES if l not in stated]
    if missing:
        raise Mismatch("the section states no " + ", no ".join(missing))
    return stated


def measured_figures(report, strict_status):
    """Read the same figures out of what `headwater check` printed.

    **A pattern that misses is an error and never a zero.** Two lines of this
    report are genuinely absent when their value is zero: the census omits
    `excluded` and the findings block omits a severity nobody hit. Every other
    line is printed whatever the count, `  0 findings` included, which
    `engine/crates/cli/tests/interface_contract.rs` holds. So `absent` is a
    reading only where the engine really omits, and each of the two is then
    held by arithmetic below rather than trusted. A report whose layout moved
    under this parser fails the job; it does not read as a corpus with nothing
    in it, which would be a suite reporting green on an input it never found.
    """

    def one(pattern, absent=None):
        m = re.search(pattern, report, re.M)
        if m is None:
            if absent is None:
                raise Mismatch("the run printed no line matching `%s`, so this suite cannot read that figure. "
                               "The report layout moved, or the run is not the run this parser was written for." % pattern)
            return absent
        return m.group(1)

    def total(pattern):
        return sum(int(m) for m in re.findall(pattern, report, re.M))

    kind = re.search(r"^\s+\d+ typed (\w+)$", report, re.M)
    if kind is None:
        raise Mismatch("the census named no typed kind")
    measured = {
        "files under the corpus root": one(r"^\s+(\d+) files under the corpus root$"),
        "typed": one(r"^\s+(\d+) typed$"),
        # Omitted when zero, and held by the census arithmetic below.
        "excluded": one(r"^\s+(\d+) excluded$", "0"),
        "checked": one(r"^\s+\d+ seen, \d+ classified, (\d+) checked, \d+ check instances$"),
        "check instances": one(r"^\s+\d+ seen, \d+ classified, \d+ checked, (\d+) check instances$"),
        "findings": one(r"^\s+(\d+) findings$"),
        # Omitted when zero, and held by the severity arithmetic below.
        "errors": one(r"^\s+(\d+) . error$", "0"),
        "warnings": one(r"^\s+(\d+) . warn$", "0"),
        "census kind count": one(r"^\s+(\d+) typed \w+$"),
        "census kind": kind.group(1),
        "graph nodes": one(r"^\s+(\d+) nodes, \d+ declared edge halves$"),
        "declared edge halves": one(r"^\s+\d+ nodes, (\d+) declared edge halves$"),
        "unresolved prose links": one(r"^\s+(\d+) prose links? that did not resolve$"),
        "strict exit status": str(strict_status),
    }

    # The two omitted-when-zero readings, checked against a line that is always
    # printed. A layout change that silences `excluded` or a severity does not
    # reach a README comparison as a zero; it fails here naming both sides.
    untyped = total(r"^\s+(\d+) untyped$")
    parts = int(measured["typed"]) + int(measured["excluded"]) + untyped
    if parts != int(measured["files under the corpus root"]):
        raise Mismatch("the census reports %s files under the corpus root and %d typed, excluded and untyped between them, "
                       "so this suite did not read every census line"
                       % (measured["files under the corpus root"], parts))
    by_severity = total(r"^\s+(\d+) . (?:error|warn|advice)$")
    if by_severity != int(measured["findings"]):
        raise Mismatch("the run reports %s findings and %d of them by severity, "
                       "so this suite did not read every severity line"
                       % (measured["findings"], by_severity))
    return measured


def run_recipe(root, readme_path, binary, scratch):
    """Run the README's own command block, and return its `check` report plus the strict exit status."""
    body = section(open(readme_path, encoding="utf-8").read(), "How to run this corpus")
    block = indented_block(body)

    log = os.path.join(scratch, "log")
    shim = os.path.join(scratch, "shim")
    tmp = os.path.join(scratch, "tmp")
    for d in (log, shim, tmp):
        os.makedirs(d)
    path = os.path.join(shim, "headwater")
    with open(path, "w", encoding="utf-8") as f:
        f.write(SHIM)
    os.chmod(path, 0o755)

    env = dict(os.environ, HW_LOG=log, HW_BIN=binary, TMPDIR=tmp,
               PATH=shim + os.pathsep + os.environ["PATH"])
    done = subprocess.run(["sh", "-e"], input=block, cwd=root, env=env,
                          text=True, capture_output=True)

    invocations = []
    for name in sorted(os.listdir(log)):
        if not name.endswith(".argv"):
            continue
        i = name[:-5]
        argv = open(os.path.join(log, name), encoding="utf-8").read().split("\n")[:-1]
        status = int(open(os.path.join(log, i + ".status"), encoding="utf-8").read())
        out = open(os.path.join(log, i + ".out"), encoding="utf-8").read()
        err = open(os.path.join(log, i + ".err"), encoding="utf-8").read()
        invocations.append((argv, status, out, err))

    for argv, status, _, err in invocations:
        if status != 0:
            raise Mismatch("the README's own command `headwater %s` exits %d, so the recipe this page prints does not run:\n%s"
                           % (" ".join(argv), status, err.strip()))
    if done.returncode != 0:
        raise Mismatch("the README's command block exits %d:\n%s" % (done.returncode, done.stderr.strip()))

    check = [(a, o) for a, _, o, _ in invocations if a[:1] == ["check"]]
    if len(check) != 1:
        raise Mismatch("the block runs `headwater check` %d times, and this suite reads one" % len(check))
    argv, report = check[0]
    strict = subprocess.run([binary] + argv + ["--strict"], capture_output=True, text=True, env=env)
    return report, strict.returncode


def compare(name, readme_path, stated, measured):
    """Every difference, each naming the file, the figure and both values."""
    return ["%s: the README states %s %s, and the run reports %s"
            % (readme_path, stated[label], label, measured[label])
            for label, _ in FIGURES if stated[label] != measured[label]]


def provoke_pin(root, readme_path, binary, scratch):
    """Arm 7. A fixture `taxonomy.yml` naming a version the vendored package no longer carries must fail this job rather than a reader."""
    fixture = os.path.dirname(readme_path)
    tree = os.path.join(scratch, "pin")
    shutil.copytree(root, tree, symlinks=True,
                    ignore=shutil.ignore_patterns(".git", "target", "node_modules"))
    pin = os.path.join(tree, os.path.relpath(fixture, root), ".headwater", "taxonomy.yml")
    text = open(pin, encoding="utf-8").read()
    bumped = re.sub(r"(?m)^(\s+version: )\d+\.\d+\.\d+$", r"\g<1>0.0.1", text, count=1)
    if bumped == text:
        raise Mismatch("%s declares no package version to provoke" % pin)
    open(pin, "w", encoding="utf-8").write(bumped)
    try:
        run_recipe(tree, os.path.join(tree, os.path.relpath(readme_path, root)),
                   binary, os.path.join(scratch, "pin-run"))
    except Mismatch as e:
        if "taxonomy resolve" in str(e) and "0.0.1" in str(e):
            return
        raise Mismatch("a wrong version pin failed for the wrong reason: %s" % e)
    raise Mismatch("a fixture pinned to a version the package does not carry ran green, so the job does not hold the pin")


def provoke_figure(name, readme_path, stated, measured):
    """Arm 5. An edited figure must fail this job naming the file and both values."""
    label = "excluded" if name == "design-spec" else "findings"
    wrong = dict(stated, **{label: str(int(stated[label]) + 7)})
    problems = compare(name, readme_path, wrong, measured)
    named = [p for p in problems if label in p and wrong[label] in p and measured[label] in p]
    if not named:
        raise Mismatch("an edited `%s` figure in %s did not fail this suite naming the file and both values; got %r"
                       % (label, readme_path, problems))


def main(root, binary):
    corpora = sorted(
        (os.path.basename(os.path.dirname(os.path.dirname(os.path.dirname(p)))), p)
        for p in __import__("glob").glob(os.path.join(root, "docs/taxonomies/*/fixtures/n8n/README.md")))

    # The denominator. A suite that reports green over an input set it never
    # found is the defect HW-OBL-0147 records against the skills suite, and it
    # is the one failure mode a passing run cannot show you.
    if len(corpora) != 3:
        print("n8n fixtures: found %d corpora under docs/taxonomies/*/fixtures/n8n/, and this suite holds 3" % len(corpora), file=sys.stderr)
        for name, path in corpora:
            print("  %s" % path, file=sys.stderr)
        return 1

    problems, held = [], 0
    scratch = tempfile.mkdtemp(prefix="hw-n8n-")
    try:
        for name, readme_path in corpora:
            rel = os.path.relpath(readme_path, root)
            try:
                stated = stated_figures(open(readme_path, encoding="utf-8").read())
                report, strict = run_recipe(root, readme_path, binary, os.path.join(scratch, name))
                measured = measured_figures(report, strict)
                problems += compare(name, rel, stated, measured)
                provoke_figure(name, rel, stated, measured)
                held += len(FIGURES)
            except Mismatch as e:
                problems.append("%s: %s" % (rel, e))
                continue
            print("n8n fixtures: %s, %d figures held" % (rel, len(FIGURES)))
        try:
            provoke_pin(root, corpora[0][1], binary, os.path.join(scratch, "provoke"))
            print("n8n fixtures: a wrong version pin fails this job")
        except Mismatch as e:
            problems.append("the version-pin arm: %s" % e)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)

    if problems:
        print("n8n fixtures: %d claims of %d corpora do not hold" % (len(problems), len(corpora)), file=sys.stderr)
        for p in problems:
            print("  " + p, file=sys.stderr)
        return 1
    print("n8n fixtures: %d figures of %d corpora hold, and each was read out of the page that states it" % (held, len(corpora)))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], os.environ["HEADWATER_BIN"]))
