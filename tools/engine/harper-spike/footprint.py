#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Count what harper-core adds, from `cargo metadata --format-version 1` output.

    python3 footprint.py <metadata-output-file>

Reads the first line of the file that parses as JSON, so the progress lines
cargo writes before it do not matter. Prints the crate count that the normal
dependency graph of harper-core reaches on linux x86_64, the crates that
headwater-doc already reaches, and the licenses of the difference.
"""
import json
import sys
from collections import Counter

meta = None
for line in open(sys.argv[1], encoding="utf-8"):
    line = line.strip()
    if line.startswith("{"):
        meta = json.loads(line)
        break

packages = {p["id"]: p for p in meta["packages"]}
nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}


def reach(root_name):
    root = next(i for i, p in packages.items() if p["name"] == root_name)
    seen, stack = set(), [root]
    while stack:
        current = stack.pop()
        if current in seen:
            continue
        seen.add(current)
        for dep in nodes[current]["deps"]:
            kinds = [k["kind"] for k in dep["dep_kinds"]]
            if None in kinds and dep["pkg"] not in seen:
                stack.append(dep["pkg"])
    return seen


harper = reach("harper-core")
doc = reach("headwater-doc")
added = harper - doc
print(f"harper-core reaches {len(harper)} crates (itself included), all targets")
print(f"headwater-doc reaches {len(doc)} crates")
print(f"crates harper-core adds: {len(added)}")
licenses = Counter(packages[i].get("license") or "(none)" for i in added)
for license, n in licenses.most_common():
    print(f"  {n:4d}  {license}")
