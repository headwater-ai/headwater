#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Write the stratified sample of #1013 from a `headwater check --format json` run.

    python3 sample.py <check.json> <root> > sample.txt

For each of five shelves it takes, in path order, up to three documents that
carry a `language.controlled.not_met` finding and up to three that carry none.
On the corpus of 2026-09-27 that is 25 documents, 10 of them with a language
finding: the evaluations shelf had one flagged document and how-to had none.
The rule is fixed so that a rerun on the same corpus picks the same files.
"""
import json
import os
import sys

SHELVES = [
    "docs/spec/",
    "docs/decisions/",
    "docs/evaluations/",
    "docs/obligations/",
    "docs/how-to/",
]
PER_SIDE = 3

check = json.load(open(sys.argv[1]))
root = sys.argv[2]
flagged = {
    f["path"] for f in check["findings"] if f["rule"] == "language.controlled.not_met"
}
for shelf in SHELVES:
    names = sorted(
        shelf + name
        for name in os.listdir(os.path.join(root, shelf))
        if name.endswith(".md") and name != "README.md"
    )
    with_finding = [p for p in names if p in flagged][:PER_SIDE]
    without = [p for p in names if p not in flagged][:PER_SIDE]
    print(f"# {shelf}: {len(names)} documents, {sum(p in flagged for p in names)} with a language finding")
    for path in with_finding + without:
        print(path)
