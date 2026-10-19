#!/usr/bin/env python3
"""Read the raw recordings of a probe campaign as one line per event.

    python3 -I tools/probe/recordings.py --batch <batch dir> [--text] <session>...

A batch directory is one of the two batches the campaign of 2026-10-03 left
on one host, `/home/james/.local/share/headwater/hw-1659/a` or `.../b`. The
recordings are not committed, and nothing in the repository can re-derive
them. The owner ruled on 2026-10-19 that they stay on that host with no end
date (#1776). They are read-only: this reader opens every file for reading,
writes nothing beside them, and a fixture in `probe-record-fixtures.sh` holds
that by a checksum of a synthetic batch taken before and after a run.

Each <session> is a session id or a shell glob over the ids, such as
`L3-*` or `L1-campaign-present-p4-r*`. A glob that matches no session is an
error, exit 2.

## The two sources

stream  `<batch>/sessions/<id>/raw.jsonl`, the stream-json the recorder read.
        It holds the tool calls, `assistant` -> `tool_use`, and their results.
log     `<batch>/config/<id>/projects/**/*.jsonl`, the harness's own session
        log, with the subagents' logs beside it. It holds the attachments the
        stream does not carry: `hook_success`, `hook_non_blocking_error`,
        `hook_additional_context`, `instructions` (the CLAUDE.md the session
        was given), `skill_listing`, and the rest.

Every path inside both sources names the original root `/var/tmp/hw-1659/`,
not the current one. The workspace of session <id> of batch <b> is
`/var/tmp/hw-1659/<b>/ws/<id>`, and that tree no longer exists, so what a
session opened is read from its calls and never from its tree.

## The event form

One TSV line per event, seven columns, or eight with `--text`:

    session  source  kind  name  target  rendered  sidechain  [text]

source     `stream` or `log`.
kind       `tool_use` for a call from the stream; `skill` for a `Skill` call,
           in place of its `tool_use` line; otherwise the attachment's own
           type from the log, such as `hook_non_blocking_error`.
name       the tool for a `tool_use`, the skill for a `skill`, the hook name
           for a hook attachment, and `-` for any other attachment.
target     for a call: the file path, the path, the skill, the Bash command,
           the pattern, the subagent type or the URL, in that order of
           preference, else the input as compact JSON. For `instructions`, the
           paths of its files joined by `,`. For `skill_listing`, the skill
           names joined by `,`. Otherwise `-`.
rendered   for an attachment, `yes` when its log line carries a `rendered`
           key, which is text the model was shown, and `no` when it does not.
           `-` for a call.
sidechain  `yes` for a call inside a subagent (the stream line names a
           `parent_tool_use_id`) and for an attachment of a subagent's log or
           a log line marked `isSidechain`; `no` otherwise.
text       with `--text`, an attachment's rendered text, else its `content`,
           else its `stderr`; `-` for a call.

A tool **result** is never an event, and neither is a text or thinking
block. So a session that read a file which quotes a hook's error message has
a `Read` event and no error event: the error is evidence only as the
harness's `hook_non_blocking_error` attachment. Tabs, newlines and
backslashes in a field print as `\\t`, `\\n` and `\\\\`.
"""

import argparse
import fnmatch
import json
import os
import sys

CALL_KEYS = ("file_path", "path", "notebook_path", "skill", "command", "pattern", "subagent_type", "url")


def field(value):
    if value is None or value == "":
        return "-"
    if not isinstance(value, str):
        value = json.dumps(value, separators=(",", ":"), ensure_ascii=False)
    return value.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")


def lines(path):
    with open(path, "r", encoding="utf-8", errors="replace") as handle:
        for raw in handle:
            raw = raw.strip()
            if not raw:
                continue
            try:
                record = json.loads(raw)
            except ValueError:
                continue
            if isinstance(record, dict):
                yield record


def call_target(tool_input):
    if not isinstance(tool_input, dict):
        return tool_input
    for key in CALL_KEYS:
        if tool_input.get(key) not in (None, ""):
            return tool_input[key]
    return tool_input


def stream_events(session, path, text):
    for record in lines(path):
        if record.get("type") != "assistant":
            continue
        content = (record.get("message") or {}).get("content")
        if not isinstance(content, list):
            continue
        side = "yes" if record.get("parent_tool_use_id") else "no"
        for block in content:
            if not isinstance(block, dict) or block.get("type") != "tool_use":
                continue
            name = block.get("name")
            tool_input = block.get("input") or {}
            if name == "Skill" and isinstance(tool_input, dict) and tool_input.get("skill"):
                row = [session, "stream", "skill", tool_input["skill"], tool_input["skill"], "-", side]
            else:
                row = [session, "stream", "tool_use", name, call_target(tool_input), "-", side]
            if text:
                row.append("-")
            yield row


def log_files(config_dir):
    projects = os.path.join(config_dir, "projects")
    found = []
    for here, dirs, files in os.walk(projects):
        dirs.sort()
        for name in sorted(files):
            if name.endswith(".jsonl"):
                found.append(os.path.join(here, name))
    # The session's own log first, then its subagents' logs.
    found.sort(key=lambda p: (os.sep + "subagents" + os.sep in p, p))
    return found


def log_events(session, config_dir, text):
    for path in log_files(config_dir):
        sub = os.sep + "subagents" + os.sep in path
        for record in lines(path):
            if record.get("type") != "attachment":
                continue
            att = record.get("attachment") or {}
            kind = att.get("type") or "-"
            name = att.get("hookName") or "-"
            target = "-"
            if kind == "instructions":
                files = att.get("files") or []
                target = ",".join(f.get("path", "") for f in files if isinstance(f, dict)) or "-"
            elif kind == "skill_listing":
                target = ",".join(att.get("names") or []) or "-"
            rendered = "yes" if "rendered" in record else "no"
            side = "yes" if (sub or record.get("isSidechain")) else "no"
            row = [session, "log", kind, name, target, rendered, side]
            if text:
                shown = record.get("rendered")
                if not isinstance(shown, str) or not shown:
                    shown = att.get("content") or att.get("stderr")
                row.append(shown)
            yield row


def sessions(batch, patterns):
    known = set()
    for sub in ("sessions", "config"):
        where = os.path.join(batch, sub)
        if os.path.isdir(where):
            known.update(n for n in os.listdir(where) if os.path.isdir(os.path.join(where, n)))
    chosen = []
    for pattern in patterns:
        hits = sorted(n for n in known if fnmatch.fnmatchcase(n, pattern))
        if not hits:
            raise LookupError(pattern)
        chosen.extend(h for h in hits if h not in chosen)
    return chosen


def main(argv):
    parser = argparse.ArgumentParser(description="Print the events of raw probe recordings, one TSV line each.")
    parser.add_argument("--batch", required=True, help="a batch directory holding sessions/ and config/")
    parser.add_argument("--text", action="store_true", help="add an eighth column, the attachment's text")
    parser.add_argument("session", nargs="+", help="a session id or a glob over ids")
    args = parser.parse_args(argv)
    try:
        chosen = sessions(args.batch, args.session)
    except LookupError as missing:
        print("recordings.py: no session of %s matches %s" % (args.batch, missing), file=sys.stderr)
        return 2
    out = sys.stdout
    for session in chosen:
        stream = os.path.join(args.batch, "sessions", session, "raw.jsonl")
        rows = []
        if os.path.isfile(stream):
            rows.extend(stream_events(session, stream, args.text))
        rows.extend(log_events(session, os.path.join(args.batch, "config", session), args.text))
        for row in rows:
            out.write("\t".join(field(v) for v in row) + "\n")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except BrokenPipeError:
        sys.exit(0)
