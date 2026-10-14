#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
#
# Read the JSON body of `GET /repos/headwater-ai/headwater/releases` (newest
# first, which is the order that endpoint states) from standard input, and
# print the first tag in the engine's own release stream that carries both
# the archive and its checksum. Print nothing, and exit 0, when none does:
# the caller (`resolve-version.sh`) decides that a "latest" no release
# answers is a refusal, and this script's job is finding, not judging.
#
# With `--range RANGE`, the first such tag must also satisfy the engine range
# a corpus states in the `engine:` line of `.headwater/taxonomy.yml` (#1600),
# so a corpus that pins an older release gets that release and not the newest.
# A range this syntax cannot read exits 2 with the reason on standard error,
# before any release is looked at.
#
# Two more modes, for the caller and for the fixtures:
#
#     resolve-latest.py --engine-line FILE      print the range FILE states, or nothing
#     resolve-latest.py --satisfies RANGE VER   print yes, no or error
#
# The range syntax is the engine's own, and `release::satisfies` in
# `engine/crates/resolve/src/release.rs` is its definition. `satisfies` below
# is a second reading of it, written so that no engine has to exist before
# one is chosen. `engine/crates/resolve/tests/engine-ranges.tsv` is the case
# table both readings answer to, and `tools/repo/integrations-fixtures.sh`
# holds this one to it row by row.
#
# Pulled out of `resolve-version.sh` as its own file so that
# `tools/repo/integrations-fixtures.sh` can hold this logic against the
# shapes a real release list actually carries, with no network reachable and
# no corpus needed: an engine release with the asset, a `taxonomy/…` release
# (a different stream, never a match), and an engine release with no asset at
# all — `v0.1.1`, a real, not hypothetical, collision with an immutable
# release object that this repository's own release history carries.
import json
import re
import sys

U64_MAX = 2**64 - 1
OPERATORS = (">=", "<=", ">", "<", "=")
# The characters Rust's `split_whitespace` and `trim` read as blank: Unicode
# White_Space. Python's own `split()` and `strip()` also take U+001C to
# U+001F, which the engine does not, so neither is used here.
BLANKS = "\t\n\x0b\x0c\r \x85\xa0                　"


def terms_of(range_):
    return [term for term in re.split(f"[{BLANKS}]+", range_) if term]


def parse(version):
    """One to three dot-separated whole numbers; an absent part is zero."""
    trimmed = version.strip(BLANKS)
    if not trimmed:
        raise ValueError("a version with no digits in it")
    split = trimmed.split(".")
    if len(split) > 3:
        raise ValueError(f"`{trimmed}` has more than three parts")
    parts = [0, 0, 0]
    for index, part in enumerate(split):
        # Rust's u64 parse: an optional `+`, then ASCII digits, below 2^64.
        # Leading zeros go first, so a long run of them is no large number.
        if not re.fullmatch(r"\+?[0-9]+", part):
            raise ValueError(f"`{trimmed}` is not three numbers")
        digits = part.lstrip("+").lstrip("0") or "0"
        if len(digits) > 20 or int(digits) > U64_MAX:
            raise ValueError(f"`{trimmed}` holds a number past 2^64")
        parts[index] = int(digits)
    return tuple(parts)


def split_operator(term):
    for operator in OPERATORS:
        if term.startswith(operator):
            return operator, term[len(operator):]
    return "=", term


def satisfies(range_, version):
    """True or False, or ValueError, as `release::satisfies` answers."""
    target = parse(version)
    comparators = 0
    for term in terms_of(range_):
        comparators += 1
        operator, rest = split_operator(term)
        bound = parse(rest)
        held = {
            ">=": target >= bound,
            ">": target > bound,
            "<=": target <= bound,
            "<": target < bound,
            "=": target == bound,
        }[operator]
        if not held:
            return False
    if comparators == 0:
        raise ValueError("it states no comparison at all")
    return True


def check_range(range_):
    """Refuse a range any of whose comparators cannot be read."""
    terms = terms_of(range_)
    if not terms:
        raise ValueError("it states no comparison at all")
    for term in terms:
        parse(split_operator(term)[1])


def engine_line(path):
    """The range in the first top-level `engine:` line of a taxonomy file."""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError:
        return ""
    for line in lines:
        match = re.match(r"engine:[ \t]*(.*)$", line)
        if not match:
            continue
        value = match.group(1)
        quoted = re.match(r"\"([^\"]*)\"|'([^']*)'", value)
        if quoted:
            return (quoted.group(1) if quoted.group(1) is not None else quoted.group(2)).strip()
        return re.sub(r"[ \t]+#.*$", "", value).strip()
    return ""


def resolve_latest(releases, range_=None):
    for release in releases:
        tag = release.get("tag_name", "")
        # The engine stream alone: "v<digit>...", never "taxonomy/...".
        if "/" in tag or not tag.startswith("v"):
            continue
        if len(tag) < 2 or not tag[1].isdigit():
            continue
        if range_ is not None:
            try:
                if not satisfies(range_, tag[1:]):
                    continue
            except ValueError:
                # A tag that is no version, such as a pre-release, is no
                # answer to a range.
                continue
        names = {asset.get("name", "") for asset in release.get("assets", [])}
        want = f"headwater-{tag}-x86_64-unknown-linux-gnu.tar.gz"
        if want in names and f"{want}.sha256" in names:
            return tag
    return None


def main(argv):
    if len(argv) == 3 and argv[1] == "--engine-line":
        value = engine_line(argv[2])
        if value:
            print(value)
        return 0
    if len(argv) == 4 and argv[1] == "--satisfies":
        try:
            print("yes" if satisfies(argv[2], argv[3]) else "no")
        except ValueError:
            print("error")
        return 0
    range_ = None
    if len(argv) == 3 and argv[1] == "--range":
        range_ = argv[2]
        try:
            check_range(range_)
        except ValueError as reason:
            print(f'the engine range "{range_}" cannot be read: {reason}', file=sys.stderr)
            return 2
    elif len(argv) != 1:
        print("usage: resolve-latest.py [--range RANGE] < releases.json", file=sys.stderr)
        return 2
    tag = resolve_latest(json.load(sys.stdin), range_)
    if tag:
        print(tag)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
