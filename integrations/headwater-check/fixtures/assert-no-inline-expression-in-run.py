#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# The static guard #1034's own second-opinion security review named: this
# action's own `run:` steps route every value that traces to `inputs.*` or
# to another step's `steps.*.outputs.*` through `env:` and read it back as a
# shell variable, never splicing a `${{ }}` expression into the script text
# directly. GitHub substitutes a `${{ }}` expression into the script BEFORE
# the shell parses it, so a value spliced there becomes shell syntax rather
# than shell data — the exact shape of the injection the review found and
# #1034 fixed once already, by hand, in `env: SARIF_PATH: ...` /
# `"$SARIF_PATH"` terms. Nothing before this script re-checked that the fix
# holds after a later edit; a human reading the diff by eye was the only
# thing standing between a future step and the same defect.
#
# This reads `action.yml`, walks `runs.steps`, and for every step that
# carries a `run:` key (a shell script this engine's own YAML loader hands
# back as one string), searches that string alone — never the step's `env:`
# mapping, where the same expression is the safe, correct place for it — for
# `${{ inputs.<name>` or `${{ steps.<id>.outputs.<name>`. Either shape found
# inside the script text is a violation, independent of which input or which
# step's output it names: the fixed line traced to `inputs.sarif-path` by
# way of `steps.merge.outputs.sarif-path`, and a future step could reach the
# same defect through any other input or any other step's output just as
# easily.
#
# `runner.*`, `github.*` and comparable GitHub-controlled context values are
# not flagged. They come from the runner's own environment rather than from
# a calling workflow's `with:` block or an upstream step's own computed
# output, so they are not the class of risk this guard exists for.
import re
import sys

try:
    import yaml
except ImportError:
    print("assert-no-inline-expression-in-run.py needs PyYAML (python3 -m pip install pyyaml)", file=sys.stderr)
    sys.exit(2)

FORBIDDEN = re.compile(r"\$\{\{\s*(inputs\.[A-Za-z0-9_-]+|steps\.[A-Za-z0-9_-]+\.outputs\.[A-Za-z0-9_-]+)")


def violations_in(action_path):
    with open(action_path, encoding="utf-8") as f:
        data = yaml.safe_load(f)

    steps = (data.get("runs") or {}).get("steps") or []
    found = []
    for i, step in enumerate(steps):
        run_text = step.get("run")
        if not isinstance(run_text, str):
            continue
        name = step.get("name", f"step {i}")
        for match in FORBIDDEN.finditer(run_text):
            line_no = run_text.count("\n", 0, match.start()) + 1
            found.append((name, line_no, match.group(0)))
    return len(steps), found


def main():
    if len(sys.argv) != 2:
        print("usage: assert-no-inline-expression-in-run.py <action.yml>", file=sys.stderr)
        return 2
    action_path = sys.argv[1]
    total_steps, found = violations_in(action_path)

    if found:
        print(f"{action_path}: {len(found)} forbidden inline expression(s) in a run: script:", file=sys.stderr)
        for name, line_no, expr in found:
            print(f"  step {name!r}, run: text line {line_no}: {expr}...", file=sys.stderr)
        print(
            "route the value through this step's own env: mapping instead, and read it "
            "back as a shell variable (e.g. env: { NAME: ${{ ... }} }, then \"$NAME\" in run:)",
            file=sys.stderr,
        )
        return 1

    print(f"{action_path}: {total_steps} run: step(s) checked, 0 forbidden inline expressions")
    return 0


if __name__ == "__main__":
    sys.exit(main())
