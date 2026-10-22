#!/usr/bin/env python3
# The keys of `.headwater/probe.yml` that the dry run of a campaign reads with
# a YAML parser: `power:` and `leaks_kept:` (#1472), which the engine does not
# read, and `rounds:` and `scenarios:` (#1774).
#
#     python3 tools/probe/declared.py <declaration> power
#     python3 tools/probe/declared.py <declaration> leaks_kept
#     python3 tools/probe/declared.py <declaration> rounds
#     python3 tools/probe/declared.py <declaration> scenarios
#     python3 tools/probe/declared.py <declaration> approved
#     python3 tools/probe/declared.py <declaration> open
#     python3 tools/probe/declared.py <declaration> base_invalidators
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
# `rounds` prints one line per round of the additive campaign, in declared
# order: `<round> <set> <rate> <rate> <tries> <probe>...`, the probes being
# the members of the set under `sets:`. Each round names a set that `sets:`
# declares, two rates that are YAML numbers strictly between 0 and 1 and
# differ, and `tries`, a whole number above zero. The engine reads `sets:`
# too, for `headwater probe plan --set`, and the dry run checks that the plan
# of each line counts the probes this prints, so the two readers cannot
# disagree unseen. A declaration with no `rounds:` prints nothing.
#
# A round that measures rates rather than a difference, such as the pilot of
# the code set (#1835), declares `repetitions`, a whole number above zero, in
# place of `rates`. Its line is `<round> <set> repetitions <n> <tries>
# <probe>...`, so the probes stay in the same fields. A round declares
# exactly one of `rates` and `repetitions`, and the reader refuses both or
# neither.
#
# A round may also declare `approved_cents`, a whole number of cents at or
# above zero: the spend a person approved for every attempt of the round
# (#1775). `approved` prints one line per round, `<round> <cents>`, and 0 for
# a round that declares none, so a round nobody approved reads as refused.
# `tools/probe/campaign.sh` refuses a batch line of a round whose approval is
# 0, and the dry run prints each round's approval.
#
# The tuning round, and no other round, may also declare `open` (#1819): the
# one layer whose plan the owner accepted, or `none`. `open` prints `none`
# for `none` or for a tuning round that declares no `open`, and otherwise one
# line, `<layer> <scenario> <try> <cap_cents> <plan> <change>`. It refuses a
# layer that `layers:` does not declare, a scenario that `scenarios:` does
# not declare, a scenario that does not add exactly the named layer over its
# base (so scenario 0, which has no base), a try outside 1 to the round's
# `tries`, a cap that is not above zero or is above the round's
# `approved_cents`, a plan that is not a URL, a change that is neither a
# commit nor `none`, and an `open` that is neither `none` nor a mapping of
# exactly those six fields. `tools/probe/campaign.sh` refuses a tuning line
# when no layer is open, and a tuning line of any scenario but the open one.
#
# `base_invalidators` prints each path pattern of the top-level list of that
# name, one per line: the paths whose change can make a measured base stale,
# as the owner ruled them on 2026-10-20 (HW-DR-0111).
#
# `scenarios` prints one line per scenario under `scenarios:`: `<n> <base>
# <layer>...`, with `-` for the base of a scenario that names none. The engine
# refuses a malformed block when it reads the declaration, so this reads the
# block for the dry run's per-scenario trees and nothing more.
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


def rate(round_name, value):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        refuse(f"a rate of the `{round_name}` round in .headwater/probe.yml is {value!r}, and it must be a number, not a string or a word")
    if not 0 < value < 1:
        refuse(f"a rate of the `{round_name}` round in .headwater/probe.yml is {value}, and it must lie strictly between 0 and 1")
    return value


def rounds(declared, approvals=False, quiet=False):
    block = declared.get("rounds")
    if block is None:
        return 0
    if not isinstance(block, dict):
        refuse("`rounds:` in .headwater/probe.yml is not a mapping of round name to its set, rates and tries")
    sets = declared.get("sets") or {}
    if not isinstance(sets, dict):
        refuse("`sets:` in .headwater/probe.yml is not a mapping of set name to probe identifiers")
    for name, fields in block.items():
        name = str(name)
        if not isinstance(fields, dict):
            refuse(f"the `{name}` round in .headwater/probe.yml is not a mapping of `set`, `rates` and `tries`")
        unknown = sorted(str(key) for key in fields if key not in ("set", "rates", "repetitions", "tries", "approved_cents", "open"))
        if unknown:
            refuse(f"the `{name}` round in .headwater/probe.yml declares `{unknown[0]}`. A round declares `set`, `rates` or `repetitions`, `tries`, `approved_cents` and, on the tuning round, `open`, and nothing else")
        if "open" in fields and name != "tuning":
            refuse(f"the `{name}` round in .headwater/probe.yml declares `open`, and only the tuning round declares `open`: the one layer whose plan the owner accepted")
        approved = fields.get("approved_cents", 0)
        if isinstance(approved, bool) or not isinstance(approved, int) or approved < 0:
            refuse(f"`approved_cents` of the `{name}` round in .headwater/probe.yml is {approved!r}, and it must be a whole number of cents at or above zero")
        chosen = fields.get("set")
        members = sets.get(chosen) if isinstance(chosen, str) else None
        if not isinstance(members, list) or not members or not all(isinstance(entry, str) for entry in members):
            refuse(f"the `{name}` round in .headwater/probe.yml runs the set {chosen!r}, and `sets:` declares no such set of probe identifiers")
        if ("rates" in fields) == ("repetitions" in fields):
            refuse(f"the `{name}` round in .headwater/probe.yml declares {'both `rates` and `repetitions`' if 'rates' in fields else 'neither `rates` nor `repetitions`'}. A round declares exactly one: two rates its count is computed from, or a count of its own")
        if "repetitions" in fields:
            count = fields["repetitions"]
            if not whole(count) or count < 1:
                refuse(f"`repetitions` of the `{name}` round in .headwater/probe.yml is {count!r}, and it must be a whole number above zero")
            high, low = "repetitions", str(count)
        else:
            rates = fields.get("rates")
            if not isinstance(rates, list) or len(rates) != 2:
                refuse(f"`rates` of the `{name}` round in .headwater/probe.yml is not a sequence of two rates")
            high, low = (rate(name, value) for value in rates)
            if high == low:
                refuse(f"the two rates of the `{name}` round in .headwater/probe.yml are equal, so no difference is powered")
        tries = fields.get("tries")
        if isinstance(tries, bool) or not isinstance(tries, int) or tries < 1:
            refuse(f"`tries` of the `{name}` round in .headwater/probe.yml is {tries!r}, and it must be a whole number above zero")
        if quiet:
            continue
        if approvals:
            print(f"{name} {approved}")
        else:
            print(" ".join([name, chosen, str(high), str(low), str(tries)] + members))
    return 0


OPEN_FIELDS = ("layer", "scenario", "try", "cap_cents", "plan", "change")


def whole(value):
    return isinstance(value, int) and not isinstance(value, bool)


def open_layer(declared):
    # Every round reads first, so an `open` never stands on a round block the
    # round gate would refuse.
    rounds(declared, quiet=True)
    tuning = (declared.get("rounds") or {}).get("tuning")
    block = tuning.get("open", "none") if isinstance(tuning, dict) else "none"
    if block == "none":
        print("none")
        return 0
    where = "`rounds.tuning.open` in .headwater/probe.yml"
    if not isinstance(block, dict):
        refuse(f"{where} is {block!r}, and it is neither `none` nor a mapping of {', '.join(OPEN_FIELDS)}")
    unknown = sorted(str(key) for key in block if key not in OPEN_FIELDS)
    if unknown:
        refuse(f"{where} declares `{unknown[0]}`. An open layer declares {', '.join(OPEN_FIELDS)}, and nothing else")
    for field in OPEN_FIELDS:
        if field not in block:
            refuse(f"{where} does not declare `{field}`. An open layer declares {', '.join(OPEN_FIELDS)}")
    layer, scenario = block["layer"], block["scenario"]
    layers = declared.get("layers") or {}
    if not isinstance(layer, str) or not isinstance(layers, dict) or layer not in layers:
        refuse(f"`layer` of {where} is {layer!r}, and `layers:` declares no layer {layer}")
    scenarios = declared.get("scenarios") or {}
    if not whole(scenario) or not isinstance(scenarios, dict) or scenario not in scenarios:
        refuse(f"`scenario` of {where} is {scenario!r}, and `scenarios:` declares no scenario {scenario}")
    fields = scenarios[scenario] if isinstance(scenarios[scenario], dict) else {}
    base = fields.get("base")
    if base is None or base not in scenarios:
        refuse(f"`scenario` of {where} is {scenario}, and scenario {scenario} names no base, so no one layer is tuned over it")
    held = fields.get("layers") or []
    base_fields = scenarios[base] if isinstance(scenarios[base], dict) else {}
    base_held = base_fields.get("layers") or []
    added = [entry for entry in held if entry not in base_held]
    if added != [layer] or any(entry not in held for entry in base_held):
        refuse(f"scenario {scenario} adds {added} over its base, scenario {base}, so it does not add exactly the {layer} layer over its base, as {where} names it")
    tuning_tries = tuning.get("tries")
    attempt = block["try"]
    if not whole(attempt) or not 1 <= attempt <= tuning_tries:
        refuse(f"`try` of {where} is {attempt!r}, and it must be a whole number from 1 to the round's `tries`, {tuning_tries}")
    approved = tuning.get("approved_cents", 0)
    cap = block["cap_cents"]
    if not whole(cap) or cap < 1:
        refuse(f"`cap_cents` of {where} is {cap!r}, and it must be a whole number of cents above zero")
    if cap > approved:
        refuse(f"`cap_cents` of {where} is {cap}, above the tuning round's `approved_cents` of {approved}")
    plan = block["plan"]
    if not isinstance(plan, str) or not plan.startswith("https://") or any(char.isspace() for char in plan):
        refuse(f"`plan` of {where} is {plan!r}, and it must be the URL of the comment in which the owner accepted the layer's plan")
    change = str(block["change"]) if isinstance(block["change"], str) else None
    if change != "none" and (change is None or not 7 <= len(change) <= 40 or any(char not in "0123456789abcdef" for char in change)):
        refuse(f"`change` of {where} is {block['change']!r}, and it must be the commit of the try's change, in hexadecimal and quoted where it holds digits alone, or `none`")
    print(" ".join(str(block[field]) for field in OPEN_FIELDS))
    return 0


def base_invalidators(declared):
    entries = declared.get("base_invalidators")
    if not isinstance(entries, list) or not entries or not all(isinstance(entry, str) and entry for entry in entries):
        refuse("`base_invalidators` in .headwater/probe.yml is not a sequence of path patterns")
    for entry in entries:
        print(entry)
    return 0


def main(argv):
    modes = ("power", "leaks_kept", "rounds", "scenarios", "approved", "open", "base_invalidators")
    if len(argv) != 2 or argv[1] not in modes:
        refuse(f"usage: python3 tools/probe/declared.py <declaration> {'|'.join(modes)}")
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

    if key == "rounds":
        return rounds(declared)

    if key == "approved":
        return rounds(declared, approvals=True)

    if key == "open":
        return open_layer(declared)

    if key == "base_invalidators":
        return base_invalidators(declared)

    if key == "scenarios":
        block = declared.get("scenarios") or {}
        if not isinstance(block, dict):
            refuse("`scenarios:` in .headwater/probe.yml is not a mapping of scenario number to its base and layers")
        for number_, fields in block.items():
            fields = fields if isinstance(fields, dict) else {}
            base = fields.get("base")
            layers = fields.get("layers") or []
            if not isinstance(layers, list):
                refuse(f"`layers` of scenario {number_} in .headwater/probe.yml is not a sequence of layer names")
            print(" ".join([str(number_), "-" if base is None else str(base)] + [str(layer) for layer in layers]))
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
