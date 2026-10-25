#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Count the sessions of one batch that used one layer of the additive campaign.

    python3 tools/probe/usage.py <declaration> <layer> <batch>

`tools/probe/try.sh result` runs it on the batch of each try of the tuning
round (#1838), and `try.sh rank` ranks the screened variants of a round on the
count it prints, before their pass rate. It prints one line:

    used <u> sessions <s>

`s` is the number of sessions under `<batch>/sessions/` that have a `status`
file, so every session the batch recorded, and `u` is the number of them that
used the layer at least once. It reads the two sources of
`tools/probe/recordings.py`, with that file's own readers, and never a
workspace:

skills  a `Skill` call in the stream, `<batch>/sessions/<id>/raw.jsonl`, that
        names a skill the skills layer ships. The shipped skills are the
        directories `.claude/skills/<name>` under `layers.skills.paths` in the
        declaration. A call that names `<plugin>:<name>` counts by its name.
hook    a session that opened a path the intent hook printed as a pointer.
        The hook's text is in the harness's log,
        `<batch>/config/<id>/projects/**/*.jsonl`, as an attachment of one of
        two types. Claude Code 2.1.288, the version the campaign runs, logs
        the output of the `UserPromptSubmit` hook as `hook_success`, and only
        a `hook_success` of that hook counts. Older logs carry it as
        `hook_additional_context`, which still counts. A pointer is a line of
        that text that opens with two spaces and a path, which is a word that
        holds a `/` and ends in a file extension, and the path is followed by
        ` (`, ` — `, ` [` or the end of the line. That is the line
        `headwater route` renders for each document it offers. A path on any
        other line does not count: the `route "<task>"` line and the
        `names the anchor` lines repeat what the task names, and an evidence
        line or a summary only describes a pointer. A pointer to a path that
        the `route "<task>"` line also holds does not count either, because
        the task already sent the session there. A session opened a
        pointer when a `Read` call names it, or a `Bash` call runs `cat`,
        `sed`, `head`, `tail` or `less` on a command that names it.
mcp     a `tool_use` in the stream whose tool name starts with `mcp__`, the
        rule `tools/probe/probe-record.sh` counts calls to a server by.

Any other layer exits 2, because the tuning round tunes these three alone
(#1833). It exits 2 when the batch has no `sessions` directory or the
declaration does not read, and 3 when python3 has no PyYAML.
"""

import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import recordings  # noqa: E402

try:
    import yaml
except ImportError:
    print("usage.py: python3 has no PyYAML, so the declaration cannot be read", file=sys.stderr)
    sys.exit(3)

LAYERS = ("skills", "hook", "mcp")
PATH_WORD = re.compile(r"[A-Za-z0-9_.@-]*/[A-Za-z0-9_./@-]*\.[A-Za-z0-9]+")
POINTER = re.compile(r"^  (" + PATH_WORD.pattern + r")(?: \(| \u2014 | \[|$)")
HOOK_KINDS = ("hook_success", "hook_additional_context")
READERS = re.compile(r"(^|[\s;&|(])(cat|sed|head|tail|less)\s")


def refuse(message):
    print(f"usage.py: {message}", file=sys.stderr)
    sys.exit(2)


def shipped_skills(declaration):
    try:
        with open(declaration, encoding="utf-8-sig") as handle:
            declared = yaml.safe_load(handle) or {}
    except (OSError, yaml.YAMLError) as error:
        refuse(f"{declaration} did not read: {error}")
    layer = ((declared.get("layers") or {}).get("skills") or {}) if isinstance(declared, dict) else {}
    names = set()
    for path in layer.get("paths") or []:
        parts = str(path).strip("/").split("/")
        if len(parts) == 3 and parts[0] == ".claude" and parts[1] == "skills":
            names.add(parts[2])
    if not names:
        refuse(f"`layers.skills.paths` in {declaration} names no `.claude/skills/<name>`, so no Skill call can count")
    return names


def bare(path):
    path = path.strip().strip("'\"")
    while path.startswith("./"):
        path = path[2:]
    return path


def same_path(named, injected):
    named, injected = bare(named), bare(injected)
    return bool(injected) and (named == injected or named.endswith("/" + injected))


def used_skills(stream, names):
    for row in recordings.stream_events("s", stream, False):
        if row[2] == "skill" and row[3].split(":")[-1] in names:
            return True
    return False


def used_mcp(stream):
    for row in recordings.stream_events("s", stream, False):
        if row[2] == "tool_use" and isinstance(row[3], str) and row[3].startswith("mcp__"):
            return True
    return False


def pointers(text):
    """The paths the hook's text offers as pointers, less the paths the task names."""
    found, named = set(), set()
    for line in text.splitlines():
        if line.startswith('route "'):
            named.update(PATH_WORD.findall(line))
            continue
        match = POINTER.match(line)
        if match:
            found.add(match.group(1))
    return {path for path in found if not any(same_path(path, word) or same_path(word, path) for word in named)}


def used_hook(stream, config):
    injected = set()
    for row in recordings.log_events("s", config, True):
        if row[2] not in HOOK_KINDS or not isinstance(row[7], str):
            continue
        if row[2] == "hook_success" and row[3] != "UserPromptSubmit":
            continue
        injected.update(pointers(row[7]))
    if not injected:
        return False
    for row in recordings.stream_events("s", stream, False):
        if row[2] != "tool_use":
            continue
        target = row[4] if isinstance(row[4], str) else ""
        if row[3] == "Read" and any(same_path(target, path) for path in injected):
            return True
        if row[3] == "Bash" and READERS.search(target):
            words = target.replace(";", " ").replace("|", " ").replace("&", " ").split()
            if any(same_path(word, path) for word in words for path in injected):
                return True
    return False


def main(argv):
    if len(argv) != 3:
        refuse("usage: python3 tools/probe/usage.py <declaration> <layer> <batch>")
    declaration, layer, batch = argv
    if layer not in LAYERS:
        refuse(f"the layer {layer} is not one the tuning round tunes, {', '.join(LAYERS)}, so no use of it is counted")
    sessions_dir = os.path.join(batch, "sessions")
    if not os.path.isdir(sessions_dir):
        refuse(f"{batch} has no sessions directory")
    names = shipped_skills(declaration) if layer == "skills" else set()
    used = sessions = 0
    for session in sorted(os.listdir(sessions_dir)):
        here = os.path.join(sessions_dir, session)
        if not os.path.isfile(os.path.join(here, "status")):
            continue
        sessions += 1
        stream = os.path.join(here, "raw.jsonl")
        config = os.path.join(batch, "config", session)
        has_stream = os.path.isfile(stream)
        if layer == "skills":
            hit = has_stream and used_skills(stream, names)
        elif layer == "mcp":
            hit = has_stream and used_mcp(stream)
        else:
            hit = has_stream and used_hook(stream, config)
        used += 1 if hit else 0
    print(f"used {used} sessions {sessions}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
