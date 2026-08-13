#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Run a stock JSON Schema validator over a corpus and print its finding set.

This script is the independent half of the `exportable_as` differential. It
shares no code with the engine: it reads the emitted schema as a file, reads the
corpus as files, parses front matter with PyYAML, and validates with the
`jsonschema` package. Nothing here imports anything Headwater wrote, so a bug in
the emitter cannot hide behind a matching bug in the checker.

Usage:

    differential_oracle.py <schema.json> <fixtures-dir>

It prints one tab-separated record per finding, sorted:

    <path>\t<rule>\t<facet>

`<rule>` is `facet.required.missing`, `facet.value.not_permitted`, or
`unclassified:<keyword>` for a validator error that neither claimed family
explains. That last form is the point of the exercise. A schema construct that
rejects a document for a reason no check carries shows up here as a finding the
engine cannot match, and the differential fails on it.

# The one dialect adjustment, and why it is not a thaw

PyYAML implements YAML 1.1, which resolves an unquoted `2026-03-01` to a date.
Headwater reads YAML 1.2 core, which has no timestamp type and leaves it a
string. The loader below drops the timestamp resolver so that both sides read
the same document. It is the dialect the engine documents rather than a
concession made to get a green run.
"""

import fnmatch
import json
import pathlib
import re
import sys

import jsonschema
import yaml


class CoreLoader(yaml.SafeLoader):
    """YAML 1.2 core, as far as the scalars in this corpus reach."""


CoreLoader.yaml_implicit_resolvers = {
    key: [(tag, regexp) for tag, regexp in resolvers if tag != "tag:yaml.org,2002:timestamp"]
    for key, resolvers in yaml.SafeLoader.yaml_implicit_resolvers.items()
}

FRONT_MATTER = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)


def front_matter(text):
    match = FRONT_MATTER.match(text)
    if match is None:
        return None
    loaded = yaml.load(match.group(1), Loader=CoreLoader)
    return loaded if isinstance(loaded, dict) else None


def matches(glob, path):
    """Whether a shelf pattern selects a path.

    `fnmatch` translates `*` to a pattern that crosses a separator, which is
    what a shelf's `**` means, and the shelves in the differential taxonomy use
    nothing else.
    """
    return fnmatch.fnmatchcase(path, glob)


def schema_for(root, pointer):
    if pointer == "#":
        return root
    return {"$defs": root.get("$defs", {}), "$ref": pointer}


def classify(error):
    """Name the check family a validator error belongs to.

    This is the one judgment the oracle makes, and it is deliberately narrow.
    Anything that is not the emitted form of one of the two claimed families is
    reported unclassified rather than quietly dropped, because a construct that
    fires for an unnamed reason is exactly what the differential is looking for.
    """
    keyword = error.validator
    if keyword == "required":
        match = re.match(r"'(.+?)' is a required property", error.message)
        return "facet.required.missing", match.group(1) if match else "?"
    if keyword == "anyOf" and error.path:
        # The emitted form of a declared value set: a composite is admitted and
        # a scalar is enumerated. See `constraint` in export.rs.
        return "facet.value.not_permitted", str(error.path[-1])
    return f"unclassified:{keyword}", "/".join(str(part) for part in error.path)


def main(argv):
    schema_path, fixtures = pathlib.Path(argv[1]), pathlib.Path(argv[2])
    root = json.loads(schema_path.read_text())
    bindings = root.get("headwater:bindings", [])

    findings = []
    for source in sorted(fixtures.rglob("*.md")):
        path = source.relative_to(fixtures).as_posix()
        instance = front_matter(source.read_text())
        if instance is None:
            continue
        pointer = next((b["schema"] for b in bindings if matches(b["path"], path)), None)
        if pointer is None:
            continue
        validator = jsonschema.Draft202012Validator(schema_for(root, pointer))
        for error in validator.iter_errors(instance):
            rule, facet = classify(error)
            findings.append((path, rule, facet))

    for record in sorted(set(findings)):
        print("\t".join(record))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
