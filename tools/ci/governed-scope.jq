# The governed-scope table of the CI job summary, from `taxonomy audit --json`.
#
#     jq -r -f tools/ci/governed-scope.jq governed-scope.json
#
# `pct` prints a share with one decimal place, as the text report does, on any
# jq version. It never interpolates the number the document holds: jq 1.6
# prints the literal `100.0` as `100`, and jq 1.7 and later print it as written
# (#1649). It computes the tenths as an integer and writes the decimal point
# itself. `tools/ci/governed-scope-fixtures.sh` holds it.
def pct:
  if . == null then "no entry"
  else ((. * 10 | round) as $t | "\($t / 10 | floor).\($t % 10)%")
  end;

"## The governed scope, and what reaches it",
"",
"The share of the entries each pattern admits that a `governance` edge reaches. It is computed on every run and committed nowhere, and the `governed-scope` artifact of this run holds it as JSON.",
"",
"| pattern | anchor kind | governed | in scope | share |",
"|---|---|---:|---:|---:|",
(.scope[] | "| `\(.pattern)` | \(.anchor_kind) | \(.governed) | \(.in_scope) | \(.share | pct) |"),
"| **total, over the union** | | \(.scope_total.governed) | \(.scope_total.in_scope) | \(.scope_total.share | pct) |",
"",
"<details><summary>The in-scope entries no edge reaches</summary>",
"",
(.scope[] | select(.ungoverned | length > 0) | "**`\(.pattern)`**", "", (.ungoverned[] | "- `\(.)`"), ""),
"</details>"
