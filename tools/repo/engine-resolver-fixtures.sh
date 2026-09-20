#!/bin/sh
# What holds clause 2 of #647: the resolution that finds either build profile
# lives in one place, `tools/repo/resolve-engine.sh` and
# `tools/repo/resolve_engine.py`, and this suite reddens the day a forty-third
# script under `.claude/` or `tools/` names only `target/release/headwater`
# and skips a `dev-release` worktree the way three consumers did before this
# issue.
#
# Run it from anywhere:
#     sh tools/repo/engine-resolver-fixtures.sh
#
# # THE DEFECT THIS SUITE EXISTS FOR
#
# `engine/.cargo/config.toml` ships `dev-release-cli` as the fast build the
# build order recommends, and it writes `engine/target/dev-release/headwater`.
# Nine consumers under `.claude/` and `tools/` already looked for either
# profile, each as its own copy of the same four lines, and three did not look
# for `dev-release` at all: `.claude/tutorial/fixtures.sh`,
# `.claude/tutorial/drive.py` and `tools/taxonomy/n8n-fixtures.sh` reported "no
# engine" against a worktree that had one, built exactly the way this
# repository's own documentation tells a session to build it. Fixing those
# three closes today's population; nothing before this suite stopped a
# forty-third script from reintroducing the same hole tomorrow.
#
# # WHAT EACH JUDGE READS
#
# THE POPULATION is every `*.sh` and `*.py` file under `.claude/` and `tools/`
# (this suite excludes only its own worktree copy under
# `.claude/worktrees/**`, which is a checked-out branch and not this
# repository's own tree) whose bytes contain the literal
# `target/release/headwater`. `.githooks/` is a sibling of both and is never
# scanned, because `.githooks/pre-commit`, `.githooks/fixtures.sh` and the two
# scripts beside them carry their own copy of the check on purpose — a
# git hook runs from whatever tree `core.hooksPath` names, and must not depend
# on a file outside `.githooks/` that a clone might not have fetched yet. Their
# own comments state that reason; this suite does not restate it by scanning
# them too.
#
# A member of the population is COMPLIANT when any of the following holds:
#
#   - it is named on the ALLOWLIST below, each with the reason it is not
#     really a consumer of the CLI binary's resolved path;
#   - its bytes also contain the literal `dev-release`, which is what every
#     already-correct duplicate carries inline; or
#   - its bytes reference `resolve-engine.sh` or `resolve_engine`, which is
#     how a consumer calls the shared implementation instead of writing an
#     eleventh copy.
#
# A population member that is none of those reddens this suite, by name.
#
# THE ALLOWLIST is two files, and adding a third needs a reason beside it, not
# just a line:
#
#   - `tools/engine/language-spike/build.sh` builds and copies
#     `target/release/libheadwater_node.so` and
#     `target/release/headwater.node`, a Node addon from an unrelated crate.
#     `headwater.node` is not the `headwater` CLI binary this suite protects.
#   - `tools/repo/developing-fixtures.sh` builds synthetic CI-workflow YAML
#     text as an `echo` fixture for its own `judge()` function, mirroring
#     `.github/workflows/*.yml`'s own steps. `CLAUDE.md` states that those
#     workflows "name `release` on purpose, because they build the shipped
#     artifact", and this fixture's text keeps matching that rather than
#     gaining a `dev-release` arm it would then have to keep true of a
#     workflow this repository chose not to change.
#
# # WHAT IT NEEDS, AND WHAT IT WRITES
#
# `grep` and `find`. It builds no engine and runs none. Every scratch tree is
# made under `mktemp -d`, the directory goes on an interrupt, and nothing
# inside this checkout is written.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)

scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

pass() {
    passed=$((passed + 1))
    echo "  ok    $1"
}

fail() {
    failed=$((failed + 1))
    echo "  FAIL  $1"
    echo "          $2"
}

same() {
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected \`$2\`, got \`$3\`"
    fi
}

more_than() {
    if [ "$3" -gt "$2" ] 2>/dev/null; then
        pass "$1 ($3)"
    else
        fail "$1" "expected more than $2, got \`$3\`"
    fi
}

# allowed NAME — true when NAME is one of the two stated exceptions.
allowed() {
    case $1 in
        tools/engine/language-spike/build.sh) return 0 ;;
        tools/repo/developing-fixtures.sh) return 0 ;;
        *) return 1 ;;
    esac
}

# compliant ROOT NAME — true when the file at ROOT/NAME is a compliant member
# of the population: allowlisted, carrying the `dev-release` literal, or
# referencing the shared resolver.
compliant() {
    if allowed "$2"; then
        return 0
    fi
    grep -q 'dev-release' "$1/$2" 2>/dev/null && return 0
    grep -qE 'resolve-engine\.sh|resolve_engine' "$1/$2" 2>/dev/null && return 0
    return 1
}

# population ROOT — one path per line, relative to ROOT, under
# ROOT/.claude and ROOT/tools (excluding ROOT/.claude/worktrees) whose bytes
# contain the literal `target/release/headwater`.
population() {
    (
        cd "$1" || exit 1
        find .claude tools \( -name '*.sh' -o -name '*.py' \) -type f \
            -not -path './.claude/worktrees/*' 2>/dev/null
    ) | while IFS= read -r p; do
        grep -qF 'target/release/headwater' "$1/$p" 2>/dev/null && printf '%s\n' "$p"
    done | sort
}

# noncompliant ROOT — one path per line, the members of `population ROOT` that
# `compliant` refuses.
noncompliant() {
    population "$1" | while IFS= read -r p; do
        compliant "$1" "$p" || printf '%s\n' "$p"
    done
}

# ---------------------------------------------------------------------------

echo "this repository's own tree"

pop=$(population "$root")
more_than "the population is non-empty" 0 "$(printf '%s\n' "$pop" | grep -c .)"

bad=$(noncompliant "$root")
same "every real consumer today carries the resolver or the dual-profile check" \
    "" "$(printf '%s\n' "$bad" | tr '\n' '|' | sed 's/^|$//')"

# The two allowlisted files really are in the population (naming a file that
# does not exist would allowlist nothing) and really are judged compliant only
# because of the allowlist, not by accident of also carrying `dev-release` or
# a reference to the resolver — a fixture case, not a comment, holds the
# reason each is on the list.
for allowed_file in tools/engine/language-spike/build.sh tools/repo/developing-fixtures.sh; do
    case $(printf '%s\n' "$pop" | grep -Fx "$allowed_file") in
        "$allowed_file") pass "the allowlist names a real population member: $allowed_file" ;;
        *) fail "the allowlist names a real population member: $allowed_file" \
                "it is not in today's population; delete the allowlist entry or find out why it vanished" ;;
    esac
done

echo
echo "the judge, provoked over a scratch tree"

mkdir -p "$scratch/tree/.claude/hooks" "$scratch/tree/tools/repo"

# The decisive case: a new script naming only the release path, with no
# dev-release fallback and no reference to the resolver. This is the shape of
# all three consumers this issue fixed, before the fix.
cat >"$scratch/tree/.claude/hooks/new-consumer.sh" <<'EOF'
#!/bin/sh
bin="$root/engine/target/release/headwater"
[ -x "$bin" ] || exit 1
EOF

bad_scratch=$(noncompliant "$scratch/tree")
same "a new single-profile script reddens the guard, by name" \
    ".claude/hooks/new-consumer.sh" "$bad_scratch"

# Repair it the first sanctioned way: carry the dev-release literal inline,
# the way the nine pre-existing correct consumers do.
cat >"$scratch/tree/.claude/hooks/new-consumer.sh" <<'EOF'
#!/bin/sh
release="$root/engine/target/release/headwater"
dev_release="$root/engine/target/dev-release/headwater"
bin=$release
[ -x "$dev_release" ] && bin=$dev_release
EOF
same "carrying the dev-release literal inline clears the guard" \
    "" "$(noncompliant "$scratch/tree")"

# Repair it the second sanctioned way: reference the shared resolver instead.
cat >"$scratch/tree/.claude/hooks/new-consumer.sh" <<'EOF'
#!/bin/sh
# not sourced for real in this fixture, only referenced by name
. "$root/tools/repo/resolve-engine.sh"
bin=$(hw_resolve_engine_bin "$root")
EOF
same "referencing the shared resolver clears the guard, with no dev-release literal in the file" \
    "" "$(noncompliant "$scratch/tree")"
same "the repaired file still names the release path, and still carries no bare dev-release literal" \
    "0" "$(grep -c 'dev-release' "$scratch/tree/.claude/hooks/new-consumer.sh")"

# A file under `tools/repo/` gets the same judgment: the population is not
# scoped to `.claude/hooks/` alone. `new-consumer.sh` was already repaired
# above, so this checks it starts clean and only the new file reddens.
cat >"$scratch/tree/tools/repo/another-new-consumer.sh" <<'EOF'
#!/bin/sh
bin="$root/engine/target/release/headwater"
EOF
same "the population reaches tools/ as well as .claude/" \
    "tools/repo/another-new-consumer.sh" "$(noncompliant "$scratch/tree" | sort | tr '\n' '|' | sed 's/|$//')"

# An allowlisted name is never flagged, even with no dev-release literal and
# no resolver reference, because the allowlist is a name-and-reason exemption
# and not a content test.
mkdir -p "$scratch/tree/tools/repo" "$scratch/tree/tools/engine/language-spike"
cat >"$scratch/tree/tools/repo/developing-fixtures.sh" <<'EOF'
#!/bin/sh
echo '  ./engine/target/release/headwater check --no-cache'
EOF
cat >"$scratch/tree/tools/engine/language-spike/build.sh" <<'EOF'
#!/bin/sh
cp target/release/libheadwater_node.so target/release/headwater.node
EOF
noflag=$(noncompliant "$scratch/tree" | grep -E 'tools/repo/developing-fixtures\.sh|tools/engine/language-spike/build\.sh')
same "the two allowlisted files are never flagged" "" "$noflag"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
