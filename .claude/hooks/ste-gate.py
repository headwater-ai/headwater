#!/usr/bin/env python3
"""Hook: hold Claude to the ste-editor skill when it edits spec prose.

Three jobs, dispatched on the hook event and tool name:

  PostToolUse / Skill       ste-editor loaded -> drop a per-session marker
  PreToolUse  / Edit|Write  spec file, no marker -> deny, and say why
  PostToolUse / Edit|Write  spec file -> run tools/ste-lint.py on the result

The gate makes the rules present in context. The lint checks the output. Neither
alone is enough: the gate cannot tell whether the rules were applied, and the
lint cannot check the things that need judgment.

Known gap: a spec written through Bash (sed, cat, a heredoc) bypasses the gate.
The pre-commit hook is the backstop for that.

This hook fails open. A broken hook that blocks every edit is worse than a
missing check, and the pre-commit hook still catches what slips through.
"""

import importlib.util
import json
import os
import subprocess
import sys
from pathlib import Path

SKILL = "ste-editor"


def project_dir() -> Path:
    return Path(os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd())


def load_lint(root: Path):
    """Import tools/ste-lint.py so scope lives in exactly one place."""
    path = root / "tools/ste-lint.py"
    if not path.exists():
        return None
    spec = importlib.util.spec_from_file_location("ste_lint", path)
    module = importlib.util.module_from_spec(spec)
    sys.modules["ste_lint"] = module
    spec.loader.exec_module(module)
    return module


def marker_for(root: Path, session_id: str) -> Path:
    return root / ".claude/state" / f"{SKILL}.{session_id or 'unknown'}"


def relative(root: Path, file_path: str) -> str | None:
    try:
        return str(Path(file_path).resolve().relative_to(root.resolve()))
    except (ValueError, OSError):
        return None


def deny(reason: str) -> None:
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
    sys.exit(0)


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        return 0

    event = payload.get("hook_event_name", "")
    tool = payload.get("tool_name", "")
    tool_input = payload.get("tool_input") or {}
    session = payload.get("session_id", "")
    root = project_dir()

    if tool == "Skill":
        if event == "PostToolUse" and str(tool_input.get("skill", "")).endswith(SKILL):
            marker = marker_for(root, session)
            marker.parent.mkdir(parents=True, exist_ok=True)
            marker.touch()
        return 0

    if tool not in ("Edit", "Write", "MultiEdit", "NotebookEdit"):
        return 0

    lint = load_lint(root)
    if lint is None:
        return 0

    file_path = tool_input.get("file_path") or tool_input.get("notebook_path") or ""
    rel = relative(root, file_path) if file_path else None
    if not rel or not lint.in_scope(rel):
        return 0

    if event == "PreToolUse":
        if marker_for(root, session).exists():
            return 0
        deny(
            f"{rel} is spec prose, held to the ASD-STE100 house profile.\n"
            f"Invoke the {SKILL} skill first (Skill tool, skill=\"{SKILL}\"), then "
            f"retry this edit. The skill carries the sentence and paragraph limits, "
            f"the verb and voice rules, and the approved-word lookups.\n"
            f"Check your work as you go with: tools/ste-lint.py " + rel
        )
        return 0

    # PostToolUse: lint what was actually written.
    result = subprocess.run(
        [sys.executable, str(root / "tools/ste-lint.py"), rel, "--quiet", "--errors-only"],
        capture_output=True, text=True, cwd=root)
    if result.returncode == 1:
        print(f"ste-lint found errors in {rel}. Fix them before you continue:\n"
              f"{result.stdout.strip()}\n"
              f"An intentional exception takes a trailing "
              f"<!-- ste-lint: allow <rule> --> comment on the offending line.",
              file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except Exception as error:  # fail open, but say so
        print(f"ste-gate hook error, failing open: {error}", file=sys.stderr)
        sys.exit(0)
