#!/bin/sh
# The write-time hook. Spec 5 gives this moment two mechanisms, and they want
# opposite postures, so this file registers on two events and dispatches on the
# one the harness names.
#
#   PreToolUse  Write|Edit   backfill. A new document written straight to disk
#                            invents its front matter, its identifier, its
#                            placement and its sections. `headwater new` derives
#                            all four from the committed lock and refuses
#                            eighteen ways before it writes a byte. So this
#                            refuses the raw write and names the verb.
#
#   PostToolUse Write|Edit   impact detection. A document may declare that it
#                            governs a code path. An edit to that path raises an
#                            advisory prompt that names the documents at risk.
#                            Spec 5 makes it advisory on purpose: a gate here
#                            trains an author to answer "no doc impact" by
#                            reflex, and that destroys the signal.
#
# What it passes to the engine: one path. What it gets back: for the refusal,
# nothing — the hook decides from the corpus descriptor the engine generated.
# For the advisory, the pointers `headwater route` resolves from the anchor.
#
# What a refusal means: the harness does not run the tool call, and the agent
# reads the reason. What happens when the harness ignores it: the write lands.
# A `Bash` call that writes the same file matches no matcher here and this hook
# never sees it. That bypass is one tool call away and it leaves no trace. The
# commit hook and the CI job are what hold the result, and this position holds
# nothing on its own.
#
# It fails open, silently, on every path it cannot decide.
#
# This is the one script spec 16 registers under three names: `.claude/`,
# `.codex/hooks.json` and `.github/hooks/*.json` all point a `PreToolUse` and
# a `PostToolUse` position at this file, because a binding calls a verb that
# ships and carries no rule of its own. `hook_event_name` and `tool_name`
# agree across all three, confirmed live, and the one path a call names is the
# one field that does not: Claude Code passes `tool_input.file_path`, Copilot
# passes `tool_input.path` on the same `Write`/`Edit` tool names, and Codex's
# edit tool is `apply_patch`, which passes the patch text under
# `tool_input.command` rather than a bare path at all. `hw_patch_path` in
# `lib.sh` is the one place that reads that last shape.

. "$(dirname "$0")/lib.sh"

input=$(cat)
event=$(hw_field "$input" hook_event_name) || exit 0
path=$(hw_field "$input" tool_input file_path) \
    || path=$(hw_field "$input" tool_input path) \
    || path=$(hw_patch_path "$input") \
    || exit 0
[ -n "$path" ] || exit 0

# The path as the corpus names it: relative to the repository root.
case $path in
    "$hw_root"/*) rel=${path#"$hw_root"/} ;;
    /*) exit 0 ;;
    *) rel=$path ;;
esac

case $event in
PreToolUse)
    # Only a file that does not exist yet. An edit to a document that already
    # has front matter is what `check --fix` and the commit hook hold, and
    # refusing it here would refuse every edit this repository is made of.
    [ -e "$hw_root/$rel" ] && exit 0
    case $rel in *.md) ;; *) exit 0 ;; esac

    # Whether the corpus claims this path is read from `.headwater/corpus.json`,
    # which `headwater generate` writes and `generate --check` holds. The hook
    # keeps no second copy of where the corpus is.
    command -v python3 >/dev/null 2>&1 || exit 0
    claimed=$(HW_REL="$rel" HW_ROOT="$hw_root" python3 -c '
import fnmatch, json, os, sys
rel = os.environ["HW_REL"]
try:
    with open(os.path.join(os.environ["HW_ROOT"], ".headwater", "corpus.json")) as f:
        descriptor = json.load(f)
except Exception:
    sys.exit(1)
for corpus in descriptor.get("corpora", []):
    root = corpus.get("root")
    if not root or not (rel == root or rel.startswith(root + "/")):
        continue
    for exclusion in corpus.get("excluded", []):
        pattern = exclusion.get("path", "")
        base = pattern[:-3] if pattern.endswith("/**") else pattern
        if rel == base or rel.startswith(base + "/") or fnmatch.fnmatch(rel, pattern):
            sys.exit(1)
    sys.stdout.write(root)
    sys.exit(0)
sys.exit(1)
' 2>/dev/null) || exit 0

    reason="\`$rel\` is a new document under the corpus root \`$claimed\`, and a raw write invents what the taxonomy already decides.

Run \`headwater new <kind> --title \"<title>\"\` instead. It reads .headwater/taxonomy.lock and derives the shelf that fixes the path, the front matter the kind requires, an identifier under the kind's scheme, and the sections the contract requires. It refuses rather than guessing, and it never overwrites a document.

Then edit the file it wrote. This refusal is for creation only, and every later edit passes.

If the file is genuinely not a document of any kind this taxonomy declares, it does not belong under \`$claimed\`."
    quoted=$(hw_quote "$reason") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":%s}}\n' "$quoted"
    exit 0
    ;;
PostToolUse)
    engine=$(hw_engine) || exit 0
    route=$("$engine" route --root "$hw_root" "$rel" 2>/dev/null) || exit 0
    printf '%s\n' "$route" | grep -q 'names the anchor' || exit 0
    pointers=$(printf '%s\n' "$route" | grep ' — ')
    [ -n "$pointers" ] || exit 0

    advisory="Headwater impact detection: a document in this corpus declares that it governs \`$rel\`, which you just changed.

$pointers

This is advisory. Read each one and say whether the change invalidated it. Nothing here blocks the edit."
    quoted=$(hw_quote "$advisory") || exit 0
    printf '{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":%s}}\n' "$quoted"
    exit 0
    ;;
*)
    exit 0
    ;;
esac
