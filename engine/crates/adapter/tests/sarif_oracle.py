#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Read a SARIF document the way a consumer that never saw this engine reads it.

    sarif_oracle.py <schema.json> <artifact.sarif>

Two jobs, and neither one asks the emitter anything.

The first is validation. The schema is the OASIS copy beside this file, and the
evaluator is the `jsonschema` package. Every object in that schema carries
`additionalProperties: false`, so a member this project invented, or misspelled,
or nested one level wrong, is an error rather than an extension.

The second is the differential. It re-derives the finding set from the document
alone — rule, path, line, level, and what became of the finding — and prints it.
The caller holds that against the run the emitter started from. A record here
that the run does not have is an invented finding, and a finding in the run that
is missing here is a dropped one. Neither shows up in schema validation, because
both are valid SARIF.

What became of a finding is read from `result.suppressions[].kind`, which is
SARIF's own member, and never from the `properties` bag this project writes.
Reading our own property back would compare a string with itself.

Output is tab-separated, one record per line, sorted:

    schema  valid
    error   <path>: <message>
    result  <ruleId>  <uri>  <line>  <level>  <live|inSource|external>

It imports nothing this project wrote.
"""

import json
import sys

import jsonschema


def records(document):
    """Every result, in the grain both sides can state."""
    out = []
    for run in document.get("runs", []):
        for result in run.get("results", []):
            rule = result.get("ruleId", "")
            level = result.get("level", "warning")
            uri, line = "", "0"
            for location in result.get("locations", []):
                physical = location.get("physicalLocation", {})
                uri = physical.get("artifactLocation", {}).get("uri", "")
                line = str(physical.get("region", {}).get("startLine", 0))
                break
            suppressions = result.get("suppressions", [])
            state = suppressions[0].get("kind", "") if suppressions else "live"
            out.append(("result", rule, uri, line, level, state))
    return out


def main(argv):
    if len(argv) != 3:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    with open(argv[1], encoding="utf-8") as handle:
        schema = json.load(handle)
    with open(argv[2], encoding="utf-8") as handle:
        document = json.load(handle)

    # The schema declares draft-04, and the validator is chosen from that
    # declaration rather than pinned here: a schema that moves drafts should
    # move this with it.
    validator = jsonschema.validators.validator_for(schema)(schema)
    errors = sorted(validator.iter_errors(document), key=lambda e: list(e.path))

    lines = []
    lines.append("\t".join(("schema", "valid" if not errors else "invalid")))
    for error in errors:
        where = "/".join(str(step) for step in error.path)
        lines.append("\t".join(("error", f"{where}: {error.message}")))
    for record in sorted(records(document)):
        lines.append("\t".join(record))
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
