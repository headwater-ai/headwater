#!/bin/sh
# What holds `integrations/headwater-check`, the composite action #833
# packages, and the adopter-facing doc beside it.
#
# There is no engine contract for "a published GitHub Action" to extend —
# #833's own adjudication says so. The nearest precedent is
# `tools/repo/readme-fixtures.sh` group 7: a fixture that reads a claim in
# one file against a claim in another, and fails the build if they disagree.
# This suite gives `docs/how-to/wire-headwater-check-into-your-own-workflow.md`
# and `integrations/headwater-check/action.yml` that same cross-file check,
# so the doc's usage example cannot drift from the action's actual inputs
# unwatched, the way the decisive-fixture workflow's own header found the
# taxonomy-lock skew unwatched the first time it was written.
#
# It also holds `resolve-latest.py` against the three release shapes that
# matter, with no network reachable and no corpus built: an engine release
# with the asset this action needs, a `taxonomy/…` release (a distinct
# stream, and never a match), and an engine release with no release object
# at all — `v0.1.1`, a real collision this repository's own history carries,
# not a hypothetical one.
#
# What this suite does NOT hold: that the action actually passes on a clean
# corpus and fails on a staled one, end to end, against a real download.
# `.github/workflows/integrations-headwater-check.yml` is that suite, and it
# needs a real network and a built corpus, which is why it is a workflow of
# its own and not a case here.
#
# Run it from anywhere:
#     sh tools/repo/integrations-fixtures.sh
#
# It needs `python3` and nothing else. It writes nothing under this checkout.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
action="$root/integrations/headwater-check/action.yml"
doc="$root/docs/how-to/wire-headwater-check-into-your-own-workflow.md"
resolver="$root/integrations/headwater-check/resolve-latest.py"

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    printf '  ok   %s\n' "$1"
}

fail() {
    failed=$((failed + 1))
    printf '  FAIL %s\n       %s\n' "$1" "$2"
}

same() {
    name=$1 want=$2 got=$3
    if [ "$want" = "$got" ]; then
        pass "$name"
    else
        fail "$name" "expected \`$want\`, got \`$got\`"
    fi
}

[ -f "$action" ] || {
    printf 'no action.yml at %s\n' "$action" >&2
    exit 1
}
[ -f "$doc" ] || {
    printf 'no doc at %s\n' "$doc" >&2
    exit 1
}
[ -x "$resolver" ] || [ -f "$resolver" ] || {
    printf 'no resolve-latest.py at %s\n' "$resolver" >&2
    exit 1
}

printf '# the doc'"'"'s usage example names inputs action.yml actually declares\n'

# Every declared input, one per line, by name alone.
declared_inputs=$(awk '
    /^inputs:/ { in_inputs = 1; next }
    /^[a-z]/ { in_inputs = 0 }
    in_inputs && /^  [a-z][a-z-]*:$/ { gsub(/[: ]/, ""); print }
' "$action")

# Every `with:` key inside the first fenced code block of the doc that also
# contains `uses:.*headwater-check`, which is the usage example rather than
# any other fence on the page (a page may show more than one block).
used_keys=$(awk '
    /^```/ { fence = 1 - fence; if (fence) { buf = ""; saw_uses = 0 } else { if (saw_uses) print buf; }; next }
    fence { buf = buf $0 "\n"; if ($0 ~ /uses:.*headwater-check/) saw_uses = 1 }
' "$doc" | awk '
    # Only the keys directly under the block'"'"'s own `with:` mapping, one
    # step indented, so `permissions:`, `jobs:` and the workflow'"'"'s other
    # keys at every indent are never mistaken for an action input. Written
    # with the two-argument `match()` (RSTART/RLENGTH), which every awk this
    # repository already runs (see `tools/repo/readme-fixtures.sh`) supports,
    # rather than the array form that only `gawk` carries.
    {
        n = match($0, /[^ ]/)
        indent = (n > 0) ? n - 1 : -1
    }
    indent >= 0 && substr($0, n) ~ /^with:[ \t]*$/ { depth = indent; in_with = 1; next }
    in_with {
        if (indent < 0) next
        if (indent <= depth) { in_with = 0; next }
        if (indent == depth + 2 && substr($0, n) ~ /^[A-Za-z0-9_-]+:/) {
            key = substr($0, n)
            sub(/:.*/, "", key)
            print key
        }
    }
')

if [ -z "$used_keys" ]; then
    fail 'the doc shows a usage example that sets at least one input' \
        'no fenced block in the doc both `uses:` this action and sets a `with:` key'
else
    unknown=""
    for key in $used_keys; do
        if ! printf '%s\n' "$declared_inputs" | grep -qx "$key"; then
            unknown="$unknown $key"
        fi
    done
    if [ -z "$unknown" ]; then
        pass 'every input the doc sets is declared in action.yml'
    else
        fail 'every input the doc sets is declared in action.yml' \
            "the doc sets$unknown, and action.yml declares: $(printf '%s' "$declared_inputs" | tr '\n' ' ')"
    fi
fi

printf '\n# resolve-latest.py, against the three release shapes that matter\n'

# Shape 1: the ordinary case. Newest first, an engine release with both files.
json1='[
  {"tag_name": "v0.1.2", "assets": [
    {"name": "headwater-v0.1.2-x86_64-unknown-linux-gnu.tar.gz"},
    {"name": "headwater-v0.1.2-x86_64-unknown-linux-gnu.tar.gz.sha256"}
  ]},
  {"tag_name": "v0.1.0", "assets": [
    {"name": "headwater-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"},
    {"name": "headwater-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"}
  ]}
]'
same 'the newest engine release with both files wins' 'v0.1.2' \
    "$(printf '%s' "$json1" | python3 "$resolver")"

# Shape 2: a taxonomy release, newer by publish order, must not win over the
# engine release beneath it. This is the exact skew this repository's own
# release history carries: `taxonomy/headwater-standard/v4.2.0` published two
# hours before `v0.1.2`.
json2='[
  {"tag_name": "taxonomy/headwater-standard/v4.2.0", "assets": [
    {"name": "headwater-standard-4.2.0.zip"}
  ]},
  {"tag_name": "v0.1.2", "assets": [
    {"name": "headwater-v0.1.2-x86_64-unknown-linux-gnu.tar.gz"},
    {"name": "headwater-v0.1.2-x86_64-unknown-linux-gnu.tar.gz.sha256"}
  ]}
]'
same 'a taxonomy release ahead of it in the list is skipped, never picked' 'v0.1.2' \
    "$(printf '%s' "$json2" | python3 "$resolver")"

# Shape 3: a tag with no release object at all, so it carries no assets —
# `v0.1.1`'s own real shape. Newest by list order, and it must be skipped for
# the newest one that actually carries the archive.
json3='[
  {"tag_name": "v0.1.1", "assets": []},
  {"tag_name": "v0.1.0", "assets": [
    {"name": "headwater-v0.1.0-x86_64-unknown-linux-gnu.tar.gz"},
    {"name": "headwater-v0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"}
  ]}
]'
same 'a tag with no asset at all is skipped, never picked' 'v0.1.0' \
    "$(printf '%s' "$json3" | python3 "$resolver")"

# Shape 4: nothing in the v* stream carries the asset. The resolver prints
# nothing and exits 0; the refusal is `resolve-version.sh`'s to write, not
# this script's, and this case holds that the resolver does not invent one.
json4='[
  {"tag_name": "taxonomy/headwater-standard/v4.2.0", "assets": [
    {"name": "headwater-standard-4.2.0.zip"}
  ]}
]'
got=$(printf '%s' "$json4" | python3 "$resolver")
same 'no matching release prints nothing rather than a wrong tag' '' "$got"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
