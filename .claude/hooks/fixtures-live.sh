#!/bin/sh
# What holds the claim `.claude/hooks/fixtures.sh` cannot: that Codex and
# Copilot actually send the shapes that suite assumes, and actually honor the
# deny and the block the way spec 16 says they do. `fixtures.sh` proves this
# repository's own scripts answer a recorded payload correctly. It cannot
# prove a harness still sends that payload, because it never runs a harness.
# This does: a real `codex exec` and a real `copilot -p`, over a scratch clone
# of this repository, with the real engine.
#
# Run it from anywhere:
#     sh .claude/hooks/fixtures-live.sh
#
# What it is not: a check, a gate, or a CI job. `headwater probe` and
# `headwater sweep` are the precedent — a person runs this by hand, on their
# own cadence, because every case here spends real AI credits against a real
# account and needs a real login. Nothing here is gated on it, the same way
# nothing gates on a probe.
#
# What it skips, and why that is not a failure: a harness not installed on
# this machine, or installed but not authenticated. Either way the case is
# unproven rather than disproven, and the summary line says which.
#
# What each case reads as evidence: never the model's own prose, because a
# model can say anything and an instruction like "do not fix this file" is a
# request, not a constraint the harness enforces. Every case below reads a
# signal the harness itself emits, or the plain state of the filesystem
# afterward.
#
#   Codex     the plain text `codex exec` already prints to standard output:
#             `hook: PreToolUse Blocked`, `hook: Stop Blocked`, `hook: Stop
#             Completed`. Confirmed by reading a real run before this script
#             existed; grep patterns below, not a guess.
#   Copilot   no such line exists in anything Copilot prints, plain or
#             `--output-format json` (checked: no event type here names a
#             hook at all). What does exist: a blocked `Stop` forces a second
#             `assistant.turn_start`, because the harness has to keep the
#             conversation open to hand the model the reason. One `turn_start`
#             is a pass. More than one is a block. The write-refusal cases
#             read the filesystem instead, which needs no such inference.
#
# Neither harness's trust model lets a script answer its prompt, so this
# works around each in the way that harness allows rather than the same way
# twice:
#   Codex     `--dangerously-bypass-hook-trust`, a real flag for exactly this.
#   Copilot   the scratch clone's path is added to `trustedFolders` in
#             `~/.copilot/config.json` for the duration of this script, and
#             removed again in a trap that fires on exit, interrupt or error.
#             A run this kills between the add and the remove leaves the
#             scratch path trusted; the trap still runs on INT and TERM, so
#             only a `kill -9` reaches that state, and the fix is deleting one
#             line by hand.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
engine="$root/engine/target/release/headwater"
branch=$(git -C "$root" rev-parse --abbrev-ref HEAD)

passed=0
failed=0
skipped=0

pass() {
    printf 'ok   %s\n' "$1"
    passed=$((passed + 1))
}

fail() {
    printf 'FAIL %s\n  %s\n' "$1" "$2"
    failed=$((failed + 1))
}

skip() {
    printf 'skip %s (%s)\n' "$1" "$2"
    skipped=$((skipped + 1))
}

[ -x "$engine" ] || {
    skip 'every live case' 'no built engine'
    printf '\n0 passed, 0 failed, %s skipped\n' "$skipped"
    exit 0
}

# A scratch clone of this checkout's committed state, at the branch it has
# checked out. This is deliberate: the point of this suite is to prove what
# would actually ship, not to let an agent under test read whatever is
# uncommitted in the working tree it happens to run from.
new_clone() {
    scratch=$(mktemp -d)
    git clone -q --no-hardlinks --local "$root" "$scratch"
    git -C "$scratch" checkout -q "$branch"
    mkdir -p "$scratch/engine/target/release"
    cp "$engine" "$scratch/engine/target/release/headwater"
}

# The one finding every case that plants it can rely on: a contraction, which
# `language.controlled.not_met` holds as an error under `ste_house`. Planted
# and reverted around the block cases, never left behind.
plant_violation() {
    target="$scratch/docs/spec/16-harness-support.md"
    python3 - "$target" <<'EOF'
import sys
path = sys.argv[1]
with open(path) as f:
    content = f.read()
content = content.replace(
    "A harness is not one program per vendor.",
    "A harness isn't one program per vendor.",
    1,
)
with open(path, "w") as f:
    f.write(content)
EOF
}

printf '# Codex\n'
if command -v codex >/dev/null 2>&1; then
    if ! timeout 30 codex exec --skip-git-repo-check "say pong" >/dev/null 2>&1; then
        skip 'Codex live cases' 'codex exec is not usable: not authenticated, or unreachable'
    else
        new_clone
        trap 'rm -rf "$scratch"' EXIT INT TERM

        run_codex() {
            (cd "$scratch" && timeout 100 codex \
                -c "projects.\"$scratch\".trust_level=\"trusted\"" \
                exec --sandbox workspace-write --dangerously-bypass-hook-trust \
                --skip-git-repo-check "$1" 2>&1)
        }

        target="$scratch/docs/obligations/9999-a-record-nobody-scaffolded.md"
        out=$(run_codex "Directly create a new file at docs/obligations/9999-a-record-nobody-scaffolded.md with the raw text 'placeholder', using apply_patch. Do not use headwater new. Do not retry a different approach if the write is blocked.")
        if [ ! -e "$target" ] && printf '%s' "$out" | grep -q 'hook: PreToolUse Blocked'; then
            pass 'a raw docs/ write is refused, and the file never lands'
        else
            fail 'a raw docs/ write is refused, and the file never lands' "$out"
        fi

        out=$(run_codex "Say the single word: pong. Do not do anything else.")
        if printf '%s' "$out" | grep -q 'hook: Stop Blocked'; then
            fail 'a clean tree lets the turn end without a block' "$out"
        else
            pass 'a clean tree lets the turn end without a block'
        fi

        plant_violation
        out=$(run_codex "Say the single word: pong. Do not do anything else, and do not fix any files even if you are told about a problem with them.")
        git -C "$scratch" checkout -q -- docs/spec/16-harness-support.md
        if printf '%s' "$out" | grep -q 'hook: Stop Blocked' && printf '%s' "$out" | grep -q 'hook: Stop Completed'; then
            pass 'a real check finding blocks the turn, and the retry completes'
        else
            fail 'a real check finding blocks the turn, and the retry completes' "$out"
        fi

        rm -rf "$scratch"
        trap - EXIT INT TERM
    fi
else
    skip 'Codex live cases' 'codex is not installed'
fi

printf '\n# Copilot\n'
if command -v copilot >/dev/null 2>&1; then
    if ! timeout 30 copilot -p "say pong" --allow-all-tools --silent >/dev/null 2>&1; then
        skip 'Copilot live cases' 'copilot is not usable: not authenticated, or unreachable'
    else
        new_clone
        copilot_config="$HOME/.copilot/config.json"
        untrust() {
            [ -f "$copilot_config" ] || return 0
            python3 - "$copilot_config" "$scratch" <<'EOF'
import json, sys
path, scratch = sys.argv[1], sys.argv[2]
with open(path) as f:
    lines = f.readlines()
text = "".join(l for l in lines if not l.strip().startswith("//"))
try:
    cfg = json.loads(text)
except Exception:
    sys.exit(0)
folders = cfg.get("trustedFolders", [])
if scratch in folders:
    folders.remove(scratch)
cfg["trustedFolders"] = folders
with open(path, "w") as f:
    f.write("// User settings belong in settings.json.\n// This file is managed automatically.\n")
    json.dump(cfg, f, indent=2)
    f.write("\n")
EOF
        }
        trap 'untrust; rm -rf "$scratch"' EXIT INT TERM

        python3 - "$copilot_config" "$scratch" <<'EOF'
import json, sys
path, scratch = sys.argv[1], sys.argv[2]
with open(path) as f:
    lines = f.readlines()
text = "".join(l for l in lines if not l.strip().startswith("//"))
cfg = json.loads(text)
folders = cfg.setdefault("trustedFolders", [])
if scratch not in folders:
    folders.append(scratch)
with open(path, "w") as f:
    f.write("// User settings belong in settings.json.\n// This file is managed automatically.\n")
    json.dump(cfg, f, indent=2)
    f.write("\n")
EOF

        run_copilot() {
            (cd "$scratch" && timeout 100 copilot -p "$1" --allow-all-tools --output-format json 2>&1)
        }
        turn_starts() {
            printf '%s' "$1" | python3 -c '
import json, sys
count = 0
for line in sys.stdin:
    line = line.strip()
    if not line.startswith("{"):
        continue
    try:
        d = json.loads(line)
    except Exception:
        continue
    if d.get("type") == "assistant.turn_start":
        count += 1
print(count)
' 2>/dev/null
        }

        target="$scratch/docs/obligations/9999-a-record-nobody-scaffolded.md"
        out=$(run_copilot "Directly create a new file at docs/obligations/9999-a-record-nobody-scaffolded.md with the raw text 'placeholder'. Do not use headwater new. Do not retry a different approach if the write is blocked.")
        if [ ! -e "$target" ]; then
            pass 'a raw docs/ write is refused, and the file never lands'
        else
            fail 'a raw docs/ write is refused, and the file never lands' "$out"
        fi

        out=$(run_copilot "Say the single word: pong. Do not do anything else.")
        n=$(turn_starts "$out")
        if [ "${n:-0}" -le 1 ]; then
            pass 'a clean tree lets the turn end without a block'
        else
            fail 'a clean tree lets the turn end without a block' "turn_starts=$n"
        fi

        plant_violation
        out=$(run_copilot "Say the single word: pong. Do not do anything else, and do not fix any files even if you are told about a problem with them.")
        git -C "$scratch" checkout -q -- docs/spec/16-harness-support.md
        n=$(turn_starts "$out")
        if [ "${n:-0}" -ge 2 ]; then
            pass 'a real check finding blocks the turn, and a second turn completes it'
        else
            fail 'a real check finding blocks the turn, and a second turn completes it' "turn_starts=$n"
        fi

        untrust
        rm -rf "$scratch"
        trap - EXIT INT TERM
    fi
else
    skip 'Copilot live cases' 'copilot is not installed'
fi

printf '\n%s passed, %s failed, %s skipped\n' "$passed" "$failed" "$skipped"
[ "$failed" -eq 0 ]
