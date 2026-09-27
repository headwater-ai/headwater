#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Draw the adjudication sample of #1013 from a spike TSV.

    python3 draw.py <root> <authored.tsv> > results/drawn.tsv

Every finding of a rule with at most ten findings, and ten findings drawn
with a fixed seed from each larger rule. Each drawn row gets a context column,
forty bytes each side of the finding in the file, and an empty verdict column
that a person fills with TP or FP.
"""
import csv
import random
import sys
from collections import defaultdict

csv.field_size_limit(10**9)
SEED = 1013
CAP = 10

root, tsv = sys.argv[1], sys.argv[2]
rows = list(csv.DictReader(open(tsv), delimiter="\t", quoting=csv.QUOTE_NONE))
by_rule = defaultdict(list)
for row in rows:
    by_rule[row["rule"]].append(row)

out = csv.writer(sys.stdout, delimiter="\t", quoting=csv.QUOTE_NONE, escapechar="\\", lineterminator="\n")
out.writerow(["rule", "n_rule", "path", "line", "flagged", "suggestion", "context", "verdict"])
for rule in sorted(by_rule):
    found = by_rule[rule]
    drawn = found if len(found) <= CAP else random.Random(f"{SEED}:{rule}").sample(found, CAP)
    for row in drawn:
        source = open(f"{root}/{row['path']}", encoding="utf-8").read().encode()
        if row["start"] != "-":
            a, b = int(row["start"]), int(row["end"])
            context = (
                source[max(0, a - 40):a].decode(errors="replace")
                + "[[" + source[a:b].decode(errors="replace") + "]]"
                + source[b:b + 40].decode(errors="replace")
            )
        else:
            context = "(unmapped) " + row["flagged"]
        context = context.replace("\n", " ").replace("\t", " ")
        out.writerow([rule, len(found), row["path"], row["line"], row["flagged"], row["suggestion"], context, ""])
