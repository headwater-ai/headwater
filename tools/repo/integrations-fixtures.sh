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
# It also holds `assert-no-inline-expression-in-run.py` against the real
# `action.yml` and against a reintroduced copy of the exact injection a
# second-opinion security review found on #1034: `action.yml`'s own `run:`
# steps route every value that traces to `inputs.*` or to another step's
# `steps.*.outputs.*` through `env:`, never splicing a `${{ }}` expression
# into the script text directly, because GitHub substitutes that expression
# into the script BEFORE the shell parses it — a value spliced there becomes
# shell syntax rather than shell data. Nothing before that review parsed or
# executed this file's script bodies at all, so a human reading the diff by
# eye was the only thing standing between a later edit and the same defect.
# This suite is what stands there now.
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
# It needs `python3`, `PyYAML` (`python3 -c 'import yaml'`, already installed
# in the CI job this runs in, by the `mkdocs` step ahead of it — see that
# step's own comment for why the two pins move together) and nothing else. It
# writes nothing under this checkout.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
action="$root/integrations/headwater-check/action.yml"
doc="$root/docs/how-to/wire-headwater-check-into-your-own-workflow.md"
resolver="$root/integrations/headwater-check/resolve-latest.py"
guard="$root/integrations/headwater-check/fixtures/assert-no-inline-expression-in-run.py"

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
[ -f "$guard" ] || {
    printf 'no assert-no-inline-expression-in-run.py at %s\n' "$guard" >&2
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

printf '\n# assert-no-inline-expression-in-run.py, against the real file and a reintroduced injection\n'

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

python3 "$guard" "$action" >"$scratch/guard-real.out" 2>&1
same 'the real, fixed action.yml has no forbidden inline expression' 0 "$?"

# The exact injection the review found and #1034 fixed: splice
# `${{ steps.merge.outputs.sarif-path }}` straight into the run: text of
# "Fail the job on a strict verdict of fail" instead of reading it back as
# `$SARIF_PATH` from that step's own `env:` mapping. Reintroduced by string
# substitution on a scratch copy rather than committed anywhere, so this case
# proves the guard catches the defect without the defect ever landing on the
# real file.
python3 - "$action" "$scratch/vulnerable.yml" <<'PY'
import sys
src, dst = sys.argv[1], sys.argv[2]
text = open(src, encoding="utf-8").read()
old = 'Read $SARIF_PATH and this job'
new = 'Read ${{ steps.merge.outputs.sarif-path }} and this job'
if old not in text:
    print(f"the line to corrupt ({old!r}) was not found verbatim in {src}", file=sys.stderr)
    sys.exit(2)
open(dst, "w", encoding="utf-8").write(text.replace(old, new))
PY
if [ $? -ne 0 ]; then
    fail 'the exact fixed line still reads $SARIF_PATH verbatim, so the injection can be reintroduced for this case' \
        'action.yml no longer matches the string this case corrupts — update both together'
else
    python3 "$guard" "$scratch/vulnerable.yml" >"$scratch/guard-vuln.out" 2>&1
    same 'reintroducing the exact injection by hand is caught' 1 "$?"
    same '  and the report names the step and the forbidden expression' 1 \
        "$(grep -c "steps.merge.outputs.sarif-path" "$scratch/guard-vuln.out")"
fi

# A second shape of the same class: `inputs.*` spliced into a run: script
# directly, rather than through `steps.*.outputs.*`. Built from nothing on
# the tree, because no real file here carries this one either.
cat >"$scratch/inputs-injection.yml" <<'YAML'
name: synthetic
description: a synthetic action.yml for this one case alone
inputs:
  root:
    required: false
    default: "."
runs:
  using: composite
  steps:
    - name: splice an input straight into the script
      shell: bash
      run: |
        echo "root is ${{ inputs.root }}"
YAML
python3 "$guard" "$scratch/inputs-injection.yml" >"$scratch/guard-inputs.out" 2>&1
same 'an inputs.* expression spliced into run: is caught the same way' 1 "$?"

# The negative case this guard exists not to break: the same expression,
# read back from env: as a shell variable, is the correct and common shape
# every step in the real file already uses, and it must never be flagged.
cat >"$scratch/env-only.yml" <<'YAML'
name: synthetic
description: a synthetic action.yml for this one case alone
inputs:
  root:
    required: false
    default: "."
runs:
  using: composite
  steps:
    - name: read the same value through env instead
      shell: bash
      env:
        ROOT: ${{ inputs.root }}
      run: |
        echo "root is $ROOT"
YAML
python3 "$guard" "$scratch/env-only.yml" >"$scratch/guard-env.out" 2>&1
same 'the same expression read back from env: is not flagged' 0 "$?"

printf '\n%s passed, %s failed\n' "$passed" "$failed"
[ "$failed" -eq 0 ]
