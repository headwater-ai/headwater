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
# line of review. Text under `~/.claude` of the host is not read, because a
# recorded session runs under a configuration directory of its own (#1467).
# `--config <dir>` names that directory, and the check then reads its
# `CLAUDE.md` and the name and description of each `skills/*/SKILL.md` too,
# under `<where>` = `config:<path>`. A campaign's directory is empty by
# construction, so the check finds nothing there unless something was planted.
#
# Every text is compared with its whitespace runs, line breaks included, read
# as one space, and so is every leak string. A literal block and a folded one
# then match alike.
#
# **The scenario channels (#1774, clause 5).** `--scenario` checks the tree of
# one scenario of the additive campaign, which the clause holds to more than
# the always-loaded set:
#
# - each skill's whole `SKILL.md`, body and all, under its path. The clause
#   names "every skill description and body". An agent body is not named, and
#   it is not read, so a leak string there is the known limit:
#   `.claude/agents/hw-adjudicate.md` holds HW-DR-0097 on 2026-10-18.
# - where the tree holds `.claude/hooks/intent.sh`, the text it injects for
#   each probe checked. The hook runs as the harness runs it: from the tree,
#   with `CLAUDE_PROJECT_DIR` set to it, the tree's `headwater` first on
#   `PATH`, and a `UserPromptSubmit` envelope on its standard input whose
#   prompt is the probe's `## Task`. The task is read from this checkout's
#   `docs/probes/<slug>.md`, because the seal removed the probe documents from
#   the tree. Its standard output is the channel `hook:<probe>`, read against
#   that probe's strings alone, because a session of one probe meets the text
#   the hook injects for its own task. Its standard error is discarded. A probe
#   whose task this checkout does not hold leaves the text not whole, and the
#   check exits 3.
# - a hit whose `<where>` is a path of the `tooling` list in the declaration
#   (#1872), or a file under one, is this repository's own harness text, which
#   the seal cannot remove and every scenario removes, so a scenario tree holds
#   it only where the tree was not built from the declaration. It prints as `harness <probe> <where> <string>` and does
#   not fail the check. The dry run of the additive spec refuses a line that
#   pools such a probe with another, as it refuses a pooled `leaks_kept` probe.
#   A kept probe stays `kept`.
#
# **The bytes the layers hand the model (#1776).** `--bytes` measures in place
# of checking, for the bytes column of HW-DR-0111's table. It counts each text
# of the always-loaded set above, as this file assembles it, in raw UTF-8 bytes:
# the text as read, before `flat()` folds its whitespace, because the harness
# hands the model the text and not its folded form. A listing counts as the text
# the harness lists, `<name> <description>` and ` - <when_to_use>` where there is
# one. An MCP tool counts as its whole definition in compact JSON, name,
# description and input schema, as the server's `tools/list` gives it. An
# imported memory file, a file under `.claude/rules` and the texts of the
# `--config` directory count as the check reads them. A skill body loads on demand when a session invokes the skill, so it is
# not counted, even with `--scenario`, and neither is a document of the corpus.
# Where the tree holds `.claude/hooks/intent.sh`, the hook's standard output
# for each probe's task prints on a line of its own and stays out of the total,
# because a session meets one probe's hook text and not every probe's.
#
# **`[]` is declared with no answer string.** A probe whose `leaks:` entry is
# the empty sequence prints nothing, because its key is an act, such as an
# `opened` key, and no string states it. A probe with no entry, or an entry
# with no value, still prints `undeclared`.
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
            # The harness cuts an import at its first `#`, so `@extra.md#top`
            # loads `extra.md` (verify round 4, read from CLI 2.1.285).
            name = match.group(1).replace("\\ ", " ").split("#", 1)[0]
            if name:
                found.append(name)
    return found


class Loaded:
    def __init__(self, workspace, config=None):
        self.workspace = workspace
        self.config = config
        self.records = []
        # (where, raw text) of each always-loaded text, for `--bytes`.
        self.raw = []
        self.seen = set()

    def where(self, path):
        if self.config is not None:
            in_config = os.path.relpath(path, self.config)
            if not in_config.startswith(".."):
                return f"config:{in_config}"
        relative = os.path.relpath(path, self.workspace)
        return path if relative.startswith("..") else relative

    def add(self, path, text, always=True):
        # `always=False` is text the check reads that the harness does not load
        # into every session, such as a skill body, so `--bytes` leaves it out.
        self.records.append((self.where(path), flat(text)))
        if always:
            self.raw.append((self.where(path), text))

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
            # The leak check reads the descriptions. `--bytes` counts the whole
            # definition as the server lists it, name, description and input
            # schema, in compact JSON, because the harness hands the model the
            # schema with the descriptions (verify round 1 of #1776).
            definition = json.dumps(tool, separators=(",", ":"), ensure_ascii=False)
            listed.append((f"mcp:{tool.get('name')}", " ".join(words), definition))
        return listed or None
    return None


def task_of(checkout, probe):
    # The probe's `## Task` section, from this checkout's probe document. The
    # seal removed the probe documents from the tree the hook runs in.
    slug = probe[len("HW-PROBE-") :] if probe.startswith("HW-PROBE-") else probe
    path = os.path.join(checkout, "docs/probes", f"{slug}.md")
    if not os.path.isfile(path):
        return None
    lines = []
    inside = False
    for line in read_text(path).split("\n"):
        if re.match(r"^##\s+Task\s*$", line):
            inside = True
            continue
        if inside and re.match(r"^#{1,2}\s", line):
            break
        if inside:
            lines.append(line)
    task = "\n".join(lines).strip()
    return task or None


def hook_text(workspace, task, probe):
    # The text `.claude/hooks/intent.sh` injects for one prompt, run as the
    # harness runs a `UserPromptSubmit` hook. `None` is a hook that did not run.
    hook = os.path.join(workspace, ".claude/hooks/intent.sh")
    env = dict(os.environ)
    env["CLAUDE_PROJECT_DIR"] = workspace
    engine_dirs = [
        os.path.join(workspace, "engine/target/dev-release"),
        os.path.join(workspace, "engine/target/release"),
    ]
    env["PATH"] = os.pathsep.join(engine_dirs + [env.get("PATH", "")])
    env.pop("HEADWATER_HOOK_ROOT", None)
    envelope = {
        "hook_event_name": "UserPromptSubmit",
        "session_id": f"leak-check-{probe}",
        "cwd": workspace,
        "prompt": task,
    }
    try:
        ran = subprocess.run(
            ["sh", hook],
            input=json.dumps(envelope) + "\n",
            capture_output=True,
            text=True,
            cwd=workspace,
            env=env,
            timeout=120,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    return ran.stdout


def harness_paths(declared):
    paths = declared.get("tooling") if isinstance(declared, dict) else None
    paths = paths if isinstance(paths, list) else []
    return [str(path).rstrip("/") for path in paths]


def under(where, paths):
    return any(where == path or where.startswith(path + "/") for path in paths)


def main(argv):
    usage = "usage: sh tools/probe/seal.sh --leak [--config <dir>] [--scenario] [--bytes] <workspace> <probe-id>..."
    config = None
    scenario = False
    measure = False
    checkout, rest = (argv[0], argv[1:]) if argv else ("", [])
    while rest and rest[0] in ("--config", "--scenario", "--bytes"):
        if rest[0] in ("--scenario", "--bytes"):
            scenario = scenario or rest[0] == "--scenario"
            measure = measure or rest[0] == "--bytes"
            rest = rest[1:]
            continue
        if len(rest) < 2:
            print(usage, file=sys.stderr)
            return 2
        config = rest[1]
        rest = rest[2:]
    argv = [checkout] + rest
    if len(argv) < 3:
        print(usage, file=sys.stderr)
        return 2
    checkout, workspace, probes = argv[0], argv[1], argv[2:]
    workspace = os.path.realpath(workspace)
    if not os.path.isdir(workspace):
        print(f"seal: no workspace directory at {argv[1]}", file=sys.stderr)
        return 2
    if config is not None:
        config = os.path.realpath(config)
        if not os.path.isdir(config):
            print(f"seal: no configuration directory at {config}", file=sys.stderr)
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

    loaded = Loaded(workspace, config)
    for name in MEMORY:
        loaded.memory(os.path.join(workspace, name), 0)
    # The host level (#1467, clause 7): the configuration directory the
    # session runs under, which the harness reads as the user's `~/.claude`.
    # Its `CLAUDE.md` loads into every session, and so do the name and
    # description of each of its skills.
    if config is not None:
        loaded.memory(os.path.join(config, "CLAUDE.md"), 0)
        config_skills = os.path.join(config, "skills")
        if os.path.isdir(config_skills):
            for name in sorted(os.listdir(config_skills)):
                path = os.path.join(config_skills, name, "SKILL.md")
                if os.path.isfile(path):
                    loaded.listed(path, fallback=True)
    for path in markdown_under(os.path.join(workspace, ".claude/rules")):
        loaded.memory(path, 0)
    skills = os.path.join(workspace, ".claude/skills")
    if os.path.isdir(skills):
        for name in sorted(os.listdir(skills)):
            path = os.path.join(skills, name, "SKILL.md")
            if os.path.isfile(path):
                loaded.listed(path, fallback=True)
    # The skill bodies of a scenario tree (#1774, clause 5): the whole file,
    # under its path, beside the listing above.
    if scenario and os.path.isdir(skills):
        for name in sorted(os.listdir(skills)):
            path = os.path.join(skills, name, "SKILL.md")
            if os.path.isfile(path):
                loaded.add(path, read_text(path), always=False)
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
        loaded.records.extend((where, flat(text)) for where, text, _ in tools)
        loaded.raw.extend((where, definition) for where, _, definition in tools)

    # The hook channel: one text per probe, read against that probe alone.
    hooked = {}
    hook_raw = {}
    if (scenario or measure) and os.path.isfile(os.path.join(workspace, ".claude/hooks/intent.sh")):
        for probe in probes:
            task = task_of(checkout, probe)
            if task is None:
                print(
                    f"seal: {probe} has no `## Task` in {checkout}/docs/probes, so the text the hook injects for it cannot be read and the scenario text is not whole.",
                    file=sys.stderr,
                )
                return 3
            text = hook_text(workspace, task, probe)
            if text is None:
                print(f"seal: the hook of {workspace} did not run for {probe}, so the scenario text is not whole.", file=sys.stderr)
                return 3
            hooked[probe] = (f"hook:{probe}", flat(text))
            hook_raw[probe] = text
    harness = harness_paths(declared) if scenario else []

    if measure:
        total = 0
        for where, text in loaded.raw:
            size = len(text.encode("utf-8"))
            total += size
            print(f"bytes {where} {size}")
        for probe in probes:
            if probe in hook_raw:
                print(f"bytes hook:{probe} {len(hook_raw[probe].encode('utf-8'))}")
        print(f"bytes total {total}")
        return 0

    found = 0
    for probe in probes:
        entry = leaks.get(probe)
        # `[]` is a declaration with no answer string; a missing entry or one
        # with no value is undeclared.
        if isinstance(entry, list) and not entry:
            continue
        strings = [entry] if isinstance(entry, str) else entry
        strings = [flat(scalar(string)) for string in (strings or []) if flat(scalar(string))]
        if not strings:
            print(f"undeclared {probe}")
            continue
        is_kept = probe in [str(item) for item in kept]
        records = loaded.records + ([hooked[probe]] if probe in hooked else [])
        seen = set()
        for string in strings:
            for where, text in records:
                if string in text and (where, string) not in seen:
                    seen.add((where, string))
                    if is_kept:
                        verdict = "kept"
                    elif under(where, harness):
                        verdict = "harness"
                    else:
                        verdict = "leak"
                    print(f"{verdict} {probe} {where} {string}")
                    if verdict == "leak":
                        found = 1
    return found


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
