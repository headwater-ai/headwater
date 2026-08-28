# SPDX-License-Identifier: Apache-2.0
"""Verify the answered-probe export differential with Python's JSON parser."""

import json
import pathlib
import sys


ANSWER = "amber"


def load(path: str) -> dict:
    return json.loads(pathlib.Path(path).read_text(encoding="utf-8"))


control = load(sys.argv[1])
filtered = load(sys.argv[2])
control_text = json.dumps(control, sort_keys=True)
filtered_text = json.dumps(filtered, sort_keys=True)
tombstones = filtered.get("tombstones", [])

assert ANSWER in control_text
assert ANSWER not in filtered_text
assert filtered["profile"]["filtered"] is True
assert tombstones == [{"documents": 1, "rule": "filtered.exclude.status"}]
assert filtered["census"]["nodes"]["accounted for"] == 1
assert filtered["census"]["nodes"]["unaccounted for"] == 0

print("control answer: present")
print("filtered answer: withheld")
print("filtered tombstone: filtered.exclude.status = 1")
