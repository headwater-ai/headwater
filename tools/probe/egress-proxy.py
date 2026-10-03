#!/usr/bin/env python3
"""The one route out of a recorded session's network (#1467, clause 5).

`tools/probe/probe-record.sh` runs each session under `bwrap --unshare-net`,
so the session's network namespace holds a loopback interface and nothing
else. This file is both ends of the one route the session keeps:

    python3 egress-proxy.py hosts
    python3 egress-proxy.py serve SOCKET LOG
    python3 egress-proxy.py forward PORT SOCKET -- COMMAND [ARGUMENT...]

`hosts` prints ALLOWED, one host to a line, for the transcript to name.

`serve` runs on the host, outside the namespace. It listens on the unix
socket SOCKET, in a directory the driver binds into the session, and it
answers HTTP requests. It opens no tunnel: it refuses every `CONNECT` with
403, to any host, because a tunnel is opaque and a call inside one to the
provider API could ask the provider to reach any host for the session.

It forwards one kind of request. The driver sets `ANTHROPIC_BASE_URL` to
`http://127.0.0.1:PORT`, so the harness sends its calls to the provider API in
plain HTTP to its own forwarder, and the proxy reads each one whole before it
opens any connection upstream. It forwards the request over TLS, with a
verifying context, to the host of ALLOWED, when all of these hold:

- the target is a path, or an `http` URL whose host is `127.0.0.1` or
  `localhost` on FORWARDER_PORT, the forwarder the base URL names; the
  driver's `egress_port` is the same number
- the head holds no control character but the `\\r\\n` that ends each line
  and a tab, so no line of the session's choice reaches the forwarded head
- the method is `POST` and the path, less its query, is one of API_PATHS
- the body has one `Content-Length` in ASCII digits of at most BODY_LIMIT, no
  `Transfer-Encoding`, no `Content-Encoding`, and parses as a JSON object
  that names no key twice in any object, because a parser upstream that
  keeps the first value would read one the proxy did not inspect
- the forward verifies the API's certificate and host name
- no entry of its `tools` has a `type` that starts with one of
  REFUSED_TOOL_PREFIXES, the body has no `mcp_servers` key, and no object
  anywhere in it has a `source` whose `type` is `url`

Each of those asks the provider to fetch from a host for the session: its web
search and web fetch tools, its MCP connector, and a document or image block
it fetches by URL. Every other request is refused with 403. It appends one
line to LOG for each decision, before it opens any connection upstream:

    allowed api.anthropic.com:443
    refused 127.0.0.1:8000
    refused-tool web_search_20250305 api.anthropic.com:443
    refused-path /v1/files api.anthropic.com:443
    refused-body chunked api.anthropic.com:443
    refused-head control-character api.anthropic.com:443

It exits when the process that started it exits, so a driver killed by a
signal leaves no proxy behind.

`forward` runs inside the session. It listens on `127.0.0.1:PORT`, then runs
COMMAND in its own place, so the harness starts only after the port accepts
connections. A child process copies each connection on the port to SOCKET,
and it exits when COMMAND does. The driver sets `HTTPS_PROXY` to
`http://127.0.0.1:PORT` for the harness.

ALLOWED and the rules above are written here and read from no variable or
argument, because a switch that widens them is a switch that opens the channel
#1467 closes. They were measured on `claude` 2.1.288 on 2026-10-03, with one
session of `haiku` that answered a prompt: the harness sent its one call to
the API, `POST /v1/messages?beta=true` with a `Content-Length` body and 23
tools, none with a `type`, through the base URL over plain HTTP with an OAuth
credential. It also asked for a `CONNECT` to `api.anthropic.com` three times,
and it answered the prompt with all three refused. The host that refreshes an
OAuth credential, `platform.claude.com`, answers `POST /v1/messages` too, and
the harness reaches it only through a tunnel, so it is refused, and a session
cannot refresh its credential. Telemetry and the update check need no host,
because the driver sets `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`. Change a
rule here only with the measurement that shows the harness needs it, and say
so in spec 15.
"""

import json
import os
import re
import socket
import ssl
import sys
import threading
import time
from urllib.parse import urlsplit

ALLOWED = ("api.anthropic.com",)
ALLOWED_PORT = 443
API_PATHS = ("/v1/messages", "/v1/messages/count_tokens")
REFUSED_TOOL_PREFIXES = ("web_search_", "web_fetch_", "mcp_")
FORWARDER_HOSTS = ("127.0.0.1", "localhost")
FORWARDER_PORT = 3128

HEAD_LIMIT = 65536
BODY_LIMIT = 32 * 1024 * 1024
HOST_SHAPE = re.compile(r"^[A-Za-z0-9.\-]+$|^\[[0-9A-Fa-f:.]+\]$")
WORD_SHAPE = re.compile(r"^[A-Za-z0-9_./\-]{1,128}$")
ASCII_DIGITS = re.compile(r"^[0-9]{1,20}$")
CONTROL = re.compile(r"[\x00-\x08\x0a-\x1f\x7f]")
FORBIDDEN = b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
DROPPED_HEADERS = ("host", "connection", "keep-alive", "expect", "upgrade", "te", "trailer")


def watch_parent():
    """Exit this process when the process that started it is gone."""
    parent = os.getppid()

    def loop():
        while True:
            time.sleep(1)
            if os.getppid() != parent:
                os._exit(0)

    threading.Thread(target=loop, daemon=True).start()


def pipe(src, dst):
    """Copy src to dst until src ends, then end dst's write side."""
    try:
        while True:
            data = src.recv(65536)
            if not data:
                break
            dst.sendall(data)
    except OSError:
        pass
    try:
        dst.shutdown(socket.SHUT_WR)
    except OSError:
        pass


def splice(a, b):
    """Copy both ways between a and b, and close both when both ends end."""
    other = threading.Thread(target=pipe, args=(b, a), daemon=True)
    other.start()
    pipe(a, b)
    other.join()
    a.close()
    b.close()


def read_head(conn):
    """Read an HTTP request head. Return it and any bytes after it."""
    data = b""
    while b"\r\n\r\n" not in data:
        if len(data) > HEAD_LIMIT:
            return None, b""
        chunk = conn.recv(4096)
        if not chunk:
            return None, b""
        data += chunk
    head, _, rest = data.partition(b"\r\n\r\n")
    return head.decode("latin-1"), rest


def target_of(head):
    """Name the host and port a request asks for, and whether it is a CONNECT."""
    line = head.split("\r\n", 1)[0]
    parts = line.split(" ")
    if len(parts) != 3:
        return None, None, False
    method, target, _ = parts
    if method == "CONNECT":
        host, sep, port = target.rpartition(":")
        if not sep or not port.isdigit():
            return target, None, True
        return host, int(port), True
    try:
        url = urlsplit(target)
        host, port = url.hostname, url.port or (443 if url.scheme == "https" else 80)
    except ValueError:
        host, port = None, None
    if not host:
        for header in head.split("\r\n")[1:]:
            name, _, value = header.partition(":")
            if name.strip().lower() == "host":
                host, port = value.strip(), 80
                break
    return host, port, False


class Log:
    def __init__(self, path):
        self.path = path
        self.lock = threading.Lock()

    def write(self, decision, host, port, what=None):
        shown = host if host and HOST_SHAPE.match(host) else "an-unreadable-host"
        if what is not None:
            decision += " " + (what if WORD_SHAPE.match(what) else "an-unreadable-name")
        line = "%s %s:%s\n" % (decision, shown, port if port is not None else "-")
        with self.lock, open(self.path, "a", encoding="utf-8") as out:
            out.write(line)
            out.flush()


def headers_of(head):
    """The header lines of a request head, as (name, line) pairs in order."""
    pairs = []
    for line in head.split("\r\n")[1:]:
        name, _, _ = line.partition(":")
        pairs.append((name.strip().lower(), line))
    return pairs


def header(pairs, wanted):
    """The values of every header named wanted."""
    return [line.partition(":")[2].strip() for name, line in pairs if name == wanted]


def forwarded_path(head):
    """The path a request to the session's forwarder asks for, or None.

    The target is origin-form, or an `http` URL whose host is the forwarder
    the base URL names. Any other target is a request for another host."""
    parts = head.split("\r\n", 1)[0].split(" ")
    if len(parts) != 3:
        return None
    target = parts[1]
    if target.startswith("/"):
        return target
    try:
        url = urlsplit(target)
        host, port = url.hostname, url.port
    except ValueError:
        return None
    if url.scheme != "http" or host not in FORWARDER_HOSTS or port != FORWARDER_PORT:
        return None
    return (url.path or "/") + ("?" + url.query if url.query else "")


def reaches_out(body):
    """Name what in a request body asks the provider to reach another host.

    Return None when nothing does."""
    tools = body.get("tools")
    if tools is not None:
        if not isinstance(tools, list):
            return "tools"
        for tool in tools:
            kind = tool.get("type") if isinstance(tool, dict) else None
            if isinstance(kind, str) and kind.startswith(REFUSED_TOOL_PREFIXES):
                return kind
    if "mcp_servers" in body:
        return "mcp_servers"
    stack = [body]
    while stack:
        item = stack.pop()
        if isinstance(item, dict):
            source = item.get("source")
            if isinstance(source, dict) and source.get("type") == "url":
                return "url-source"
            stack.extend(item.values())
        elif isinstance(item, list):
            stack.extend(item)
    return None


class DuplicateKey(Exception):
    pass


def unique_keys(pairs):
    """Build a JSON object, and refuse one that names a key twice.

    Here the last value of a key wins. A parser upstream that keeps the first
    would read a value the proxy never inspected."""
    names = [name for name, _ in pairs]
    if len(set(names)) != len(names):
        raise DuplicateKey()
    return dict(pairs)


def tls_context():
    """The context of the forward: it verifies the API's certificate and name."""
    return ssl.create_default_context()


def read_body(conn, pairs, rest):
    """Read a request body the proxy can inspect. Return it, or the reason it cannot."""
    if header(pairs, "transfer-encoding"):
        return None, "chunked"
    if [v for v in header(pairs, "content-encoding") if v.lower() != "identity"]:
        return None, "compressed"
    lengths = header(pairs, "content-length")
    if len(set(lengths)) != 1 or not ASCII_DIGITS.match(lengths[0]):
        return None, "no-length"
    length = int(lengths[0])
    if length > BODY_LIMIT:
        return None, "too-long"
    data = rest
    while len(data) < length:
        chunk = conn.recv(min(1 << 20, length - len(data)))
        if not chunk:
            return None, "short"
        data += chunk
    if len(data) != length:
        return None, "too-long"
    try:
        body = json.loads(data.decode("utf-8"), object_pairs_hook=unique_keys)
    except DuplicateKey:
        return None, "duplicate-key"
    except (ValueError, RecursionError):
        return None, "not-json"
    if not isinstance(body, dict):
        return None, "not-json"
    return data, body


def forward_api(conn, log, head, rest):
    """Read one request to the provider API, and forward it or refuse it."""
    api = ALLOWED[0]
    path = forwarded_path(head)
    if path is None:
        return False
    if CONTROL.search(head.replace("\r\n", "")):
        log.write("refused-head", api, ALLOWED_PORT, "control-character")
        conn.sendall(FORBIDDEN)
        return True
    method = head.split(" ", 1)[0]
    if method != "POST":
        log.write("refused-path", api, ALLOWED_PORT, method)
        conn.sendall(FORBIDDEN)
        return True
    if path.split("?", 1)[0] not in API_PATHS:
        log.write("refused-path", api, ALLOWED_PORT, path.split("?", 1)[0])
        conn.sendall(FORBIDDEN)
        return True
    pairs = headers_of(head)
    data, body = read_body(conn, pairs, rest)
    if data is None:
        log.write("refused-body", api, ALLOWED_PORT, body)
        conn.sendall(FORBIDDEN)
        return True
    what = reaches_out(body)
    if what is not None:
        log.write("refused-tool", api, ALLOWED_PORT, what)
        conn.sendall(FORBIDDEN)
        return True
    log.write("allowed", api, ALLOWED_PORT)
    try:
        raw = socket.create_connection((api, ALLOWED_PORT), timeout=15)
        upstream = tls_context().wrap_socket(raw, server_hostname=api)
    except OSError:
        conn.sendall(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
        return True
    lines = ["POST %s HTTP/1.1" % path, "Host: %s" % api]
    for name, line in pairs:
        if name not in DROPPED_HEADERS and not name.startswith("proxy-"):
            lines.append(line)
    lines.append("Connection: close")
    upstream.settimeout(None)
    conn.settimeout(None)
    upstream.sendall(("\r\n".join(lines) + "\r\n\r\n").encode("latin-1") + data)
    pipe(upstream, conn)
    upstream.close()
    return True


def handle(conn, log):
    try:
        conn.settimeout(30)
        head, rest = read_head(conn)
        if head is None:
            log.write("refused", None, None)
            conn.close()
            return
        if forward_api(conn, log, head, rest):
            conn.close()
            return
        host, port, _ = target_of(head)
        log.write("refused", host, port)
        conn.sendall(FORBIDDEN)
        conn.close()
    except OSError:
        conn.close()


def serve(path, log_path):
    watch_parent()
    log = Log(log_path)
    if os.path.exists(path):
        os.unlink(path)
    listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    listener.bind(path)
    listener.listen(64)
    while True:
        conn, _ = listener.accept()
        threading.Thread(target=handle, args=(conn, log), daemon=True).start()


def forward_one(conn, path):
    try:
        upstream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        upstream.connect(path)
    except OSError:
        conn.close()
        return
    splice(conn, upstream)


def forward(port, path, command):
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", port))
    listener.listen(64)
    if os.fork() == 0:
        watch_parent()
        while True:
            conn, _ = listener.accept()
            threading.Thread(target=forward_one, args=(conn, path), daemon=True).start()
    listener.close()
    os.execvp(command[0], command)


def main(argv):
    if argv == ["hosts"]:
        sys.stdout.write("".join(host + "\n" for host in ALLOWED))
    elif len(argv) == 3 and argv[0] == "serve":
        serve(argv[1], argv[2])
    elif len(argv) >= 5 and argv[0] == "forward" and argv[3] == "--" and argv[1].isdigit():
        forward(int(argv[1]), argv[2], argv[4:])
    else:
        sys.stderr.write(
            "usage: egress-proxy.py hosts\n"
            "       egress-proxy.py serve SOCKET LOG\n"
            "       egress-proxy.py forward PORT SOCKET -- COMMAND [ARGUMENT...]\n"
        )
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
