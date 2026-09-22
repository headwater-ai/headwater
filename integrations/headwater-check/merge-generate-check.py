#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# `headwater check --format sarif` writes one SARIF finding per document-level
# rule instance, and `headwater generate --check` writes a plain-text report
# with no SARIF shape of its own, because it judges a whole projection against
# its source rather than one document against a rule. This script reads both
# and writes one SARIF file that carries what a reader of code scanning needs
# from each: read `generate --check`'s report for the lines this engine
# writes for a projection that disagrees with its source, under the header
# "projections, held to regeneration", and add one `error`-level SARIF result
# per stale projection, under a rule this action declares (`headwater/
# projection.stale`) because the engine states no SARIF shape for a verb that
# never learned one.
#
# THE TWO-LINE SHAPE THIS PARSES, from `headwater generate --check`'s own
# report (see `engine/crates/generate/src/lib.rs`):
#
#     projections, held to regeneration
#       shelf_index docs/how-to/README.md
#         committed, and it is not what this corpus and this lock produce
#       shelf_index docs/decisions/README.md
#         unchanged
#
# Only the two-space-indented "<kind> <path>" line immediately followed by a
# four-space-indented "committed, and it is not..." line is a finding; every
# other pair (including "unchanged", and every line under the "refused
# transcripts" and "what this verb does not write" headers that follow) is
# left alone.
import json
import sys

STALE_MARKER = "committed, and it is not what this corpus and this lock produce"
RULE_ID = "headwater/projection.stale"


def find_stale_projections(report_text):
    lines = report_text.splitlines()
    found = []
    for i, line in enumerate(lines):
        if not line.startswith("  ") or line.startswith("   "):
            continue
        header = line.strip()
        parts = header.split(None, 1)
        if len(parts) != 2:
            continue
        kind, path = parts
        if i + 1 >= len(lines):
            continue
        verdict = lines[i + 1].strip()
        if verdict == STALE_MARKER:
            found.append((kind, path))
    return found


def main():
    if len(sys.argv) != 4:
        print(
            "usage: merge-generate-check.py <check.sarif> <generate-check.txt> <out.sarif>",
            file=sys.stderr,
        )
        return 2
    sarif_path, report_path, out_path = sys.argv[1:4]

    with open(sarif_path, encoding="utf-8") as f:
        sarif = json.load(f)

    with open(report_path, encoding="utf-8") as f:
        report_text = f.read()

    stale = find_stale_projections(report_text)

    run = sarif["runs"][0]
    driver = run["tool"]["driver"]
    rules = driver.setdefault("rules", [])
    if not any(rule.get("id") == RULE_ID for rule in rules):
        rules.append(
            {
                "id": RULE_ID,
                "shortDescription": {
                    "text": "a committed projection is not what this corpus and this lock produce"
                },
                "help": {
                    "text": "Run `headwater generate` and commit the result. `headwater generate --check` holds this in continuous integration."
                },
            }
        )

    results = run.setdefault("results", [])
    for kind, path in stale:
        results.append(
            {
                "ruleId": RULE_ID,
                "level": "error",
                "kind": "fail",
                "message": {
                    "text": (
                        f"the committed {kind} at `{path}` is not what this corpus and this "
                        "lock produce. Run `headwater generate` and commit the result."
                    )
                },
                "locations": [
                    {
                        "physicalLocation": {
                            "artifactLocation": {"uri": path},
                        }
                    }
                ],
                "properties": {"headwater": {"source": "generate --check"}},
            }
        )

    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(sarif, f, indent=2)
        f.write("\n")

    print(
        f"{len(stale)} stale projection(s) of {len(results)} total SARIF result(s) written to {out_path}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
