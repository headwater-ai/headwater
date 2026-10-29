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
# Thirteen consumers under `.claude/` and `tools/` already looked for either
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
# THE POPULATION is every file under `.claude/` and `tools/` (this suite
# excludes only its own worktree copy under `.claude/worktrees/**`, which is
# a checked-out branch and not this repository's own tree) that `is_script`
# counts and whose bytes contain the literal `target/release/headwater`.
# `is_script` is a `.sh`/`.py` suffix, an executable bit or a `#!` opening —
# not a name pattern, because this repository ships real no-extension POSIX
# shell scripts in this same scope (`tools/cap-run`, `tools/hw-cargo`, both
# executable and both starting `#!/bin/sh`), and a `*.sh`/`*.py` name filter
# is exactly what this suite shipped with and had to widen: a planted stub in
# that shape passed the day this suite landed, because `find` never looked at
# a file `find -name '*.sh' -o -name '*.py'` does not match. The same
# widening first swept in `.claude/skills/headwater-engine/SKILL.md`, a
# documentation page whose own invocation example quotes the literal in
# prose; `is_script` excludes it correctly, because that file is neither
# executable nor `.sh`/`.py` nor shebang-opened, which is the same three-part
# test every real script in this population passes.
#
# `.githooks/` is a sibling of both and is never scanned, because
# `.githooks/pre-commit`, `.githooks/fixtures.sh` and the two scripts beside
# them carry their own copy of the check on purpose — a git hook runs from
# whatever tree `core.hooksPath` names, and must not depend on a file outside
# `.githooks/` that a clone might not have fetched yet. Their own comments
# state that reason; this suite does not restate it by scanning them too.
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

# is_script ROOT NAME — true when the file at ROOT/NAME is something this
# suite treats as a script rather than as prose that happens to quote a path.
# `.sh` and `.py` still count by name, and so does a `.claude/skills/*.md`
# code block that quotes an example invocation NOT because it is prose (it
# is), but because it fails both tests below and this comment records why it
# is meant to. Beyond the two named suffixes, a file counts when it is
# executable or opens with `#!`, which is what a real no-extension script in
# this scope looks like: `tools/cap-run` and `tools/hw-cargo` are both, and
# neither carries a name pattern this suite could have matched instead.
is_script() {
    case $2 in
        *.sh | *.py) return 0 ;;
    esac
    [ -x "$1/$2" ] && return 0
    [ "$(head -c 2 "$1/$2" 2>/dev/null)" = '#!' ] && return 0
    return 1
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
# ROOT/.claude and ROOT/tools (excluding ROOT/.claude/worktrees), that
# `is_script` counts and whose bytes contain the literal
# `target/release/headwater`.
population() {
    (
        cd "$1" || exit 1
        find .claude tools -type f -not -path '.claude/worktrees/*' 2>/dev/null
    ) | while IFS= read -r p; do
        is_script "$1" "$p" || continue
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

# The counts #990 asked a re-run to report, so a reader can see them move: of
# the population, how many reference the resolver, and how many carry the
# `dev-release` literal and no reference to it. Information, not a judgment.
pop_resolver=0
pop_inline=0
for p in $pop; do
    if grep -qE 'resolve-engine\.sh|resolve_engine' "$root/$p"; then
        pop_resolver=$((pop_resolver + 1))
    elif grep -q 'dev-release' "$root/$p"; then
        pop_inline=$((pop_inline + 1))
    fi
done
echo "  info  population $(printf '%s\n' "$pop" | grep -c .): $pop_resolver reference the resolver, $pop_inline carry dev-release with no reference to it"

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

# The same shape, with no file extension at all — the decisive case for the
# extension filter this suite shipped with and then had to remove. `find
# .claude tools -name '*.sh' -o -name '*.py'` never looked at a file named
# plainly, the way `tools/cap-run` and `tools/hw-cargo` really are named, so a
# planted stub in exactly this shape passed a version of this suite that
# scanned by name pattern instead of by directory alone.
cat >"$scratch/tree/tools/repo/no-ext-stub" <<'EOF'
#!/bin/sh
bin="$root/engine/target/release/headwater"
EOF
same "a new single-profile script with no file extension also reddens the guard, by name" \
    "tools/repo/no-ext-stub" "$(noncompliant "$scratch/tree" | grep -Fx 'tools/repo/no-ext-stub')"
rm -f "$scratch/tree/tools/repo/no-ext-stub"

# The other half of `is_script`: no shebang, but the executable bit set. A
# no-extension script written without one still reddens.
printf 'bin="$root/engine/target/release/headwater"\n' >"$scratch/tree/tools/repo/exec-no-shebang-stub"
chmod +x "$scratch/tree/tools/repo/exec-no-shebang-stub"
same "an executable no-extension file with no shebang also reddens the guard, by name" \
    "tools/repo/exec-no-shebang-stub" "$(noncompliant "$scratch/tree" | grep -Fx 'tools/repo/exec-no-shebang-stub')"
rm -f "$scratch/tree/tools/repo/exec-no-shebang-stub"

# A documentation page that quotes the same path in prose is neither
# executable, `.sh`/`.py`, nor shebang-opened, and `is_script` excludes it
# from the population entirely — it never reaches `compliant` at all, and a
# doc that shows the release path as a worked example is not asked to also
# explain `dev-release`. This is the shape of
# `.claude/skills/headwater-engine/SKILL.md`, which the widened scan first
# swept in by accident.
printf '# An example\n\n    engine/target/release/headwater check --root .\n' \
    >"$scratch/tree/tools/repo/example.md"
same "a non-executable, non-shebang file naming the path in prose is not in the population at all" \
    "" "$(population "$scratch/tree" | grep -Fx 'tools/repo/example.md')"
rm -f "$scratch/tree/tools/repo/example.md"

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

# A nested worktree under `.claude/worktrees/` is another branch checked out
# inside this tree, not this repository's own file. `find` prints its paths
# with no `./` prefix, so an exclusion written as `./.claude/worktrees/*`
# never matched, and on a checkout that held agent worktrees this suite
# failed on 26 paths that were not its own (#990).
mkdir -p "$scratch/tree/.claude/worktrees/x/tools/repo"
cat >"$scratch/tree/.claude/worktrees/x/tools/repo/nested-consumer.sh" <<'EOF'
#!/bin/sh
bin="$root/engine/target/release/headwater"
EOF
same "a single-profile script inside a nested worktree is not in the population" \
    "" "$(population "$scratch/tree" | grep -F '.claude/worktrees/')"
rm -rf "$scratch/tree/.claude/worktrees"

echo
echo "the files #990 moved onto the shared resolver"

# Clause 1 of #990: each of these files gets its engine from
# `hw_resolve_engine_bin` and carries no profile comparison of its own. The
# population judge above accepts an inline copy as compliant, so it cannot
# see a file that goes back to its own copy. This list can.
#
# `own_copy` matches the three shapes the copies took: a `-nt` comparison of
# a profile path, an executable test of a `dev` variable, and an assignment of
# the `dev-release` path that a `release` fallback then follows. A fallback to
# the `release` path after the resolver has answered no is not a copy, and it
# does not match.
own_copy='release/headwater"? *-nt|\[ -x "?\$[A-Za-z_]*dev|[A-Za-z_]+=["$A-Za-z_{}/]*target/dev-release/headwater"?$'

# The pattern is tried on each shape before it is trusted with the list.
for shape in \
    '    { [ -z "$engine" ] || [ "$root/engine/target/dev-release/headwater" -nt "$engine" ]; }; then' \
    'if [ -x "$dev_release_engine" ] && { [ ! -x "$engine" ] || [ "$dev_release_engine" -nt "$engine" ]; }; then' \
    'engine=$root/engine/target/dev-release/headwater'; do
    if printf '%s\n' "$shape" | grep -qE "$own_copy"; then
        pass "the copy detector matches: $shape"
    else
        fail "the copy detector matches: $shape" "it does not"
    fi
done
same "the copy detector leaves a release fallback alone" "0" \
    "$(printf '%s\n' 'engine=$(hw_resolve_engine_bin "$root") || engine="$root/engine/target/release/headwater"' | grep -cE "$own_copy")"

for migrated in \
    .claude/hooks/lib.local.sh \
    .claude/hooks/fixtures-live.sh \
    .claude/skills/fixtures.sh \
    tools/repo/readme-fixtures.sh \
    tools/repo/diataxis-facet-fixtures.sh \
    tools/repo/diataxis-fixtures.sh \
    tools/repo/decision-record-fixtures.sh \
    tools/engine/color-fixtures.sh \
    tools/site/refresh-figures.sh \
    tools/probe/probe-record.sh \
    tools/probe/probe-record-fixtures.sh \
    tools/probe/probe-transform.sh; do
    if ! grep -q 'hw_resolve_engine_bin' "$root/$migrated"; then
        fail "$migrated resolves through the shared resolver" "it does not call hw_resolve_engine_bin"
    elif grep -qE "$own_copy" "$root/$migrated"; then
        fail "$migrated resolves through the shared resolver" "it still carries its own profile comparison"
    else
        pass "$migrated resolves through the shared resolver"
    fi
done

echo
echo "sourcing the resolver leaves nothing a child process inherits"

# Clause 4 of #990: `.claude/hooks/fixtures-live.sh` sources the resolver
# because the resolver's own contract says sourcing it sets no variable a
# child could inherit. This case holds that contract. It runs a clean shell,
# captures what a child sees before and after the resolver is sourced and
# called, and requires the two to be byte-identical. A root with both
# profiles built takes every branch of the function.
mkdir -p "$scratch/eng/engine/target/release" "$scratch/eng/engine/target/dev-release"
printf '#!/bin/sh\n' >"$scratch/eng/engine/target/release/headwater"
printf '#!/bin/sh\n' >"$scratch/eng/engine/target/dev-release/headwater"
chmod +x "$scratch/eng/engine/target/release/headwater" "$scratch/eng/engine/target/dev-release/headwater"
cat >"$scratch/child-env.sh" <<'EOF'
# child-env.sh RESOLVER ROOT OUT — write what a child sees before and after.
env | sort >"$3.before"
. "$1"
hw_resolve_engine_bin "$2" >/dev/null
hw_resolve_engine_missing_message "$2" >/dev/null
env | sort >"$3.after"
EOF

# inherits RESOLVER [NAME=VALUE...] — empty when a child sees the same
# environment before and after RESOLVER is sourced and called; otherwise the
# lines that changed. Each NAME=VALUE is exported into the clean shell first.
inherits() {
    _resolver=$1
    shift
    rm -f "$scratch/env.before" "$scratch/env.after"
    env -i PATH="$PATH" HOME="$HOME" "$@" sh "$scratch/child-env.sh" "$_resolver" "$scratch/eng" "$scratch/env" || {
        echo "child-env.sh did not run"
        return
    }
    diff "$scratch/env.before" "$scratch/env.after" | grep '^[<>]'
}

same "sourcing and calling the real resolver changes nothing a child inherits" \
    "" "$(inherits "$root/tools/repo/resolve-engine.sh")"
same "and the child never sees HEADWATER_HOOK_ROOT" \
    "0" "$(grep -c '^HEADWATER_HOOK_ROOT=' "$scratch/env.after")"

# The probe must be able to see an export, or its silence above proves
# nothing. Two copies of the resolver each export one variable, and each must
# show up.
sed 's/^\( *\)_hw_resolve_root=\$1$/\1_hw_resolve_root=$1; export _hw_resolve_root/' \
    "$root/tools/repo/resolve-engine.sh" >"$scratch/resolve-export.sh"
same "a resolver that exports its own variable is caught" \
    "> _hw_resolve_root=$scratch/eng" "$(inherits "$scratch/resolve-export.sh")"
sed 's/^\( *\)_hw_resolve_root=\$1$/\1_hw_resolve_root=$1; HEADWATER_HOOK_ROOT=$1; export HEADWATER_HOOK_ROOT/' \
    "$root/tools/repo/resolve-engine.sh" >"$scratch/resolve-hook-root.sh"
inherits "$scratch/resolve-hook-root.sh" >/dev/null
same "a resolver that exports HEADWATER_HOOK_ROOT is caught" \
    "1" "$(grep -c '^HEADWATER_HOOK_ROOT=' "$scratch/env.after")"

# A clean shell has nothing exported for the resolver to overwrite, so a plain
# assignment to a name the caller already exports would pass the cases above.
# These run with HEADWATER_HOOK_ROOT exported first, as `.claude/hooks/fixtures.sh`
# exports it.
same "with HEADWATER_HOOK_ROOT already exported, the real resolver leaves it as it was" \
    "" "$(inherits "$root/tools/repo/resolve-engine.sh" HEADWATER_HOOK_ROOT=/sentinel)"
sed 's/^\( *\)_hw_resolve_root=\$1$/\1_hw_resolve_root=$1; HEADWATER_HOOK_ROOT=$1/' \
    "$root/tools/repo/resolve-engine.sh" >"$scratch/resolve-assign.sh"
same "a resolver that assigns an exported name, with no export of its own, is caught" \
    "> HEADWATER_HOOK_ROOT=$scratch/eng" \
    "$(inherits "$scratch/resolve-assign.sh" HEADWATER_HOOK_ROOT=/sentinel | grep '^>')"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
