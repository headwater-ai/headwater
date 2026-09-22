#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# `sys.argv[1]` is a SARIF file `merge-generate-check.py` wrote. `sys.argv[2]`
# is the path a stale projection is expected to name. Exit 0 and print the
# matching result when one names it; exit 1 and print every result's ruleId
# and message when none does, so a CI failure here shows what the SARIF held
# instead of asking a reader to download the artifact to find out.
import json
import sys

if len(sys.argv) != 3:
    print("usage: assert-sarif-names-staleness.py <sarif-file> <expected-path>", file=sys.stderr)
    sys.exit(2)

sarif_path, expected_path = sys.argv[1], sys.argv[2]

with open(sarif_path, encoding="utf-8") as f:
    sarif = json.load(f)

results = sarif["runs"][0].get("results", [])
matches = [
    r
    for r in results
    if r.get("ruleId") == "headwater/projection.stale"
    and any(
        loc.get("physicalLocation", {}).get("artifactLocation", {}).get("uri") == expected_path
        for loc in r.get("locations", [])
    )
]

if not matches:
    print(f"no SARIF result named {expected_path!r} under headwater/projection.stale", file=sys.stderr)
    print(f"this run held {len(results)} result(s):", file=sys.stderr)
    for r in results:
        print(f"  {r.get('ruleId')}: {r.get('message', {}).get('text')}", file=sys.stderr)
    sys.exit(1)

for m in matches:
    print(m["message"]["text"])
