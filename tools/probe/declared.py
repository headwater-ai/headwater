#!/usr/bin/env python3
# Two keys of `.headwater/probe.yml` that the dry run of a campaign reads and
# the engine does not: `power:` and `leaks_kept:` (#1472).
#
#     python3 tools/probe/declared.py <declaration> power
#     python3 tools/probe/declared.py <declaration> leaks_kept
#
# `power` prints `present <rate>`, `absent <rate>`, `alpha <level>`,
# `power <power>` and `pooled <category>...`, one per line. Each of the four
# numbers must be a YAML number strictly between 0 and 1, and the two rates
# must differ. A quoted number, a word, or a missing key is refused, because a
# hand-written reader read `"0.15"` as 0 and priced the plan at exit 0 (verify
# round 5). `pooled` is a sequence of category names, or absent.
#
# `leaks_kept` prints each probe the declaration keeps, one per line, in any
# YAML sequence form. The same key is read by `tools/probe/leak.py`, with the
# same parser, so the leak check and the dry run never disagree about which
# probe is kept (verify round 5: a flow sequence read as nothing).
#
# It reads with PyYAML and never with a hand-written reader, as the parent
# ruled after verify round 3. It exits 0 when it printed the key, 2 when the
# declaration or the key does not read, and 3 when python3 has no PyYAML.
import sys

try:
    import yaml
except ImportError:
    print("campaign: python3 has no PyYAML, so .headwater/probe.yml cannot be read", file=sys.stderr)
    sys.exit(3)


def refuse(message):
    print(f"campaign: {message}", file=sys.stderr)
    sys.exit(2)


def number(block, key):
    value = block.get(key)
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        refuse(f"`power.{key}` in .headwater/probe.yml is {value!r}, and it must be a number, not a string or a word")
    if not 0 < value < 1:
        refuse(f"`power.{key}` in .headwater/probe.yml is {value}, and it must lie strictly between 0 and 1")
    return value


def main(argv):
    if len(argv) != 2 or argv[1] not in ("power", "leaks_kept"):
        refuse("usage: python3 tools/probe/declared.py <declaration> power|leaks_kept")
    path, key = argv
    try:
        with open(path, encoding="utf-8-sig") as handle:
            declared = yaml.safe_load(handle) or {}
    except (OSError, yaml.YAMLError) as error:
        refuse(f"{path} did not read: {error}")
    if not isinstance(declared, dict):
        refuse(f"{path} is not a mapping")

    if key == "leaks_kept":
        kept = declared.get("leaks_kept") or []
        if not isinstance(kept, list) or not all(isinstance(entry, str) for entry in kept):
            refuse("`leaks_kept` in .headwater/probe.yml is not a sequence of probe identifiers")
        for entry in kept:
            print(entry)
        return 0

    block = declared.get("power")
    if not isinstance(block, dict):
        refuse("`power:` in .headwater/probe.yml is not a mapping of present, absent, alpha and power")
    values = {name: number(block, name) for name in ("present", "absent", "alpha", "power")}
    if values["present"] == values["absent"]:
        refuse("`power.present` and `power.absent` are equal, so no difference is powered")
    pooled = block.get("pooled") or []
    if isinstance(pooled, str):
        pooled = [pooled]
    if not isinstance(pooled, list) or not all(isinstance(entry, str) for entry in pooled):
        refuse("`power.pooled` in .headwater/probe.yml is not a sequence of category names")
    for name, value in values.items():
        print(f"{name} {value}")
    print(" ".join(["pooled"] + pooled))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
