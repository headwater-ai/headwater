#!/usr/bin/env python3
# The leak check of `tools/probe/seal.sh --leak` (#1472). `seal.sh` execs this
# file and states the contract in its own header: the lines it prints, and the
# exit status 0, 1, 2 or 3.
#
#     python3 tools/probe/leak.py <checkout> <workspace> <probe>...
#
# It reads every YAML it needs with PyYAML, never with a hand-written reader.
# Verify rounds 1 to 3 each found one more form a hand-written reader of front
# matter missed (a folded or quoted multi-line description, `when-to-use`, a
# carriage return), and the parent ruled that a real parser replace it. PyYAML
# is on this host and on CI: the MkDocs install of the `headwater` job supplies
# it before the recorder fixtures run.
#
# The always-loaded set is the text Claude Code puts into every session of a
# workspace, as verify round 3 read it from the installed harness, CLI 2.1.285:
#
# - the memory files: `CLAUDE.md`, `.claude/CLAUDE.md`, `CLAUDE.local.md`,
#   `AGENTS.md`, `.claude/AGENTS.md`, each `.md` file under `.claude/rules/`
#   at any depth, and each file one of them imports with `@<path>`, to a depth
#   of five imports. A file two names reach, such as `AGENTS.md` as a link to
#   `CLAUDE.md`, is read once, under the first name.
# - for each skill (`.claude/skills/*/SKILL.md`), agent definition
#   (`.claude/agents/**/*.md`) and command (`.claude/commands/**/*.md`): the
#   `name`, `description` and `when_to_use` keys of its front matter, with
#   `when-to-use` read as `when_to_use` as the harness normalizes it. A skill
#   or a command with no description is listed by the first non-empty line of
#   its body, less leading `#`. Front matter that does not parse is read whole,
#   so a leak string in it is still found.
# - where the workspace declares a project MCP server in `.mcp.json`, the
#   description of each tool `headwater mcp` lists, and the descriptions of
#   its parameters.
#
# A `.claude/rules/` file with a `paths:` key loads only when the session
# touches a matching path. The check reads it anyway, because a check that
# guessed the session's paths could miss a leak, and a false leak costs one
# line of review. Text under `~/.claude` of the host is not read: that is
# confining a session to its workspace (#1467).
#
# Every text is compared with its whitespace runs, line breaks included, read
# as one space, and so is every leak string. A literal block and a folded one
# then match alike.
import json
import os
import re
import subprocess
import sys

try:
    import yaml
except ImportError:
    print(
        "seal: python3 has no PyYAML, so the always-loaded text cannot be read and the leak check does not run",
        file=sys.stderr,
    )
    sys.exit(3)

MEMORY = ["CLAUDE.md", ".claude/CLAUDE.md", "CLAUDE.local.md", "AGENTS.md", ".claude/AGENTS.md"]
IMPORT_DEPTH = 5


def flat(text):
    return " ".join(text.split())


def read_text(path):
    # `utf-8-sig` drops a byte-order mark, and universal newlines read a CRLF
    # file as an LF one.
    with open(path, encoding="utf-8-sig", errors="replace", newline=None) as handle:
        return handle.read()


def scalar(value):
    if value is None:
        return ""
    if isinstance(value, list):
        return " ".join(scalar(item) for item in value)
    if isinstance(value, dict):
        return " ".join(f"{key} {scalar(item)}" for key, item in value.items())
    return str(value)


def imports(text):
    # `@<path>` after the start of a line or whitespace, outside a fenced code
    # block and outside an inline code span, as the harness reads an import.
    found = []
    fenced = False
    for line in text.split("\n"):
        if re.match(r"^\s*(```|~~~)", line):
            fenced = not fenced
            continue
        if fenced:
            continue
        line = re.sub(r"`[^`]*`", "", line)
        for match in re.finditer(r"(?:^|\s)@((?:\\ |[^\s])+)", line):
            found.append(match.group(1).replace("\\ ", " "))
    return found


class Loaded:
    def __init__(self, workspace):
        self.workspace = workspace
        self.records = []
        self.seen = set()

    def where(self, path):
        relative = os.path.relpath(path, self.workspace)
        return path if relative.startswith("..") else relative

    def add(self, path, text):
        self.records.append((self.where(path), flat(text)))

    def memory(self, path, depth):
        if not os.path.isfile(path):
            return
        real = os.path.realpath(path)
        if real in self.seen:
            return
        self.seen.add(real)
        text = read_text(path)
        self.add(path, text)
        if depth >= IMPORT_DEPTH:
            return
        for name in imports(text):
            if name.startswith("~/"):
                continue
            target = name if os.path.isabs(name) else os.path.join(os.path.dirname(path), name)
            self.memory(os.path.normpath(target), depth + 1)

    def listed(self, path, fallback):
        text = read_text(path)
        lines = text.split("\n")
        body = lines
        fields = {}
        if lines and lines[0].rstrip() == "---":
            close = next((i for i in range(1, len(lines)) if lines[i].rstrip() == "---"), None)
            if close is not None:
                matter = "\n".join(lines[1:close])
                body = lines[close + 1 :]
                try:
                    parsed = yaml.safe_load(matter)
                except yaml.YAMLError:
                    parsed = None
                if not isinstance(parsed, dict):
                    if matter.strip():
                        self.add(path, matter)
                    parsed = {}
                fields = {str(key).lower().replace("-", "_"): value for key, value in parsed.items()}
        description = scalar(fields.get("description")).strip()
        if not description and fallback:
            first = next((line for line in body if line.strip()), "")
            description = first.strip().lstrip("#").strip()
        parts = [scalar(fields.get("name")).strip(), description]
        when = scalar(fields.get("when_to_use")).strip()
        text = " ".join(part for part in parts if part)
        if when:
            text = f"{text} - {when}"
        if text.strip():
            self.add(path, text)


def markdown_under(directory):
    found = []
    for base, directories, files in os.walk(directory, followlinks=True):
        directories.sort()
        for name in sorted(files):
            if name.endswith(".md"):
                found.append(os.path.join(base, name))
    return found


def mcp_tools(checkout, workspace):
    engine = os.path.join(checkout, "engine/target/dev-release/headwater")
    if not os.access(engine, os.X_OK):
        engine = os.path.join(checkout, "engine/target/release/headwater")
    if not os.access(engine, os.X_OK):
        return None
    requests = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05"}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
    ]
    try:
        ran = subprocess.run(
            [engine, "mcp", "--root", workspace],
            input="".join(json.dumps(request) + "\n" for request in requests),
            capture_output=True,
            text=True,
            timeout=120,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    for line in ran.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("id") != 2:
            continue
        tools = (message.get("result") or {}).get("tools") or []
        listed = []
        for tool in tools:
            words = [scalar(tool.get("description"))]
            properties = ((tool.get("inputSchema") or {}).get("properties")) or {}
            for prop in properties.values():
                if isinstance(prop, dict):
                    words.append(scalar(prop.get("description")))
            listed.append((f"mcp:{tool.get('name')}", flat(" ".join(words))))
        return listed or None
    return None


def main(argv):
    if len(argv) < 3:
        print("usage: sh tools/probe/seal.sh --leak <workspace> <probe-id>...", file=sys.stderr)
        return 2
    checkout, workspace, probes = argv[0], argv[1], argv[2:]
    workspace = os.path.realpath(workspace)
    if not os.path.isdir(workspace):
        print(f"seal: no workspace directory at {argv[1]}", file=sys.stderr)
        return 2
    declaration = os.environ.get("HW_PROBE_YML") or os.path.join(checkout, ".headwater/probe.yml")
    try:
        declared = yaml.safe_load(read_text(declaration)) or {}
    except (OSError, yaml.YAMLError) as error:
        print(f"seal: {declaration} did not read: {error}", file=sys.stderr)
        return 2
    if not isinstance(declared, dict):
        print(f"seal: {declaration} is not a mapping", file=sys.stderr)
        return 2
    leaks = declared.get("leaks") or {}
    kept = declared.get("leaks_kept") or []
    if not isinstance(leaks, dict) or not isinstance(kept, list):
        print(f"seal: `leaks` in {declaration} is not a mapping, or `leaks_kept` is not a sequence", file=sys.stderr)
        return 2

    loaded = Loaded(workspace)
    for name in MEMORY:
        loaded.memory(os.path.join(workspace, name), 0)
    for path in markdown_under(os.path.join(workspace, ".claude/rules")):
        loaded.memory(path, 0)
    skills = os.path.join(workspace, ".claude/skills")
    if os.path.isdir(skills):
        for name in sorted(os.listdir(skills)):
            path = os.path.join(skills, name, "SKILL.md")
            if os.path.isfile(path):
                loaded.listed(path, fallback=True)
    for path in markdown_under(os.path.join(workspace, ".claude/agents")):
        loaded.listed(path, fallback=False)
    for path in markdown_under(os.path.join(workspace, ".claude/commands")):
        loaded.listed(path, fallback=True)
    if os.path.isfile(os.path.join(workspace, ".mcp.json")):
        tools = mcp_tools(checkout, workspace)
        if tools is None:
            print(
                f"seal: {workspace} declares an MCP server, and no engine listed its tools, so the always-loaded text is not whole.",
                file=sys.stderr,
            )
            return 3
        loaded.records.extend(tools)

    found = 0
    for probe in probes:
        strings = leaks.get(probe)
        if isinstance(strings, str):
            strings = [strings]
        strings = [flat(scalar(string)) for string in (strings or []) if flat(scalar(string))]
        if not strings:
            print(f"undeclared {probe}")
            continue
        verdict = "kept" if probe in [str(entry) for entry in kept] else "leak"
        seen = set()
        for string in strings:
            for where, text in loaded.records:
                if string in text and (where, string) not in seen:
                    seen.add((where, string))
                    print(f"{verdict} {probe} {where} {string}")
                    if verdict == "leak":
                        found = 1
    return found


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
