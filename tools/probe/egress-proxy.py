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
answers HTTP proxy requests. It opens a tunnel for a `CONNECT` to a host of
ALLOWED on port 443 and refuses every other request with 403, a `CONNECT`
elsewhere and every plain HTTP request alike. It appends one line to LOG for
each decision, before it opens any connection upstream:

    allowed api.anthropic.com:443
    refused 127.0.0.1:8000

It exits when the process that started it exits, so a driver killed by a
signal leaves no proxy behind.

`forward` runs inside the session. It listens on `127.0.0.1:PORT`, then runs
COMMAND in its own place, so the harness starts only after the port accepts
connections. A child process copies each connection on the port to SOCKET,
and it exits when COMMAND does. The driver sets `HTTPS_PROXY` to
`http://127.0.0.1:PORT` for the harness.

ALLOWED is written here and read from no variable or argument, because a
switch that widens it is a switch that opens the direct channel #1467 closes.
The provider API stays a channel of its own, because a call with the
credential can ask it for a server-side web tool; spec 15 states that. The
hosts are the ones the harness needs to answer a prompt, measured on
`claude` 2.1.287 on 2026-10-03: the provider's API, and the host it refreshes
an OAuth credential at (`TOKEN_URL` in the harness binary). Telemetry and the
update check need no host, because the driver sets
`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`. Add a host here only with the
measurement that shows the harness needs it, and say so in spec 15.
"""

import os
import re
import socket
import sys
import threading
import time
from urllib.parse import urlsplit

ALLOWED = ("api.anthropic.com", "platform.claude.com")
ALLOWED_PORT = 443

HEAD_LIMIT = 65536
HOST_SHAPE = re.compile(r"^[A-Za-z0-9.\-]+$|^\[[0-9A-Fa-f:.]+\]$")


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

    def write(self, decision, host, port):
        shown = host if host and HOST_SHAPE.match(host) else "an-unreadable-host"
        line = "%s %s:%s\n" % (decision, shown, port if port is not None else "-")
        with self.lock, open(self.path, "a", encoding="utf-8") as out:
            out.write(line)
            out.flush()


def handle(conn, log):
    try:
        conn.settimeout(30)
        head, rest = read_head(conn)
        if head is None:
            log.write("refused", None, None)
            conn.close()
            return
        host, port, is_connect = target_of(head)
        name = (host or "").lower().strip("[]")
        if is_connect and name in ALLOWED and port == ALLOWED_PORT:
            log.write("allowed", name, port)
            try:
                upstream = socket.create_connection((name, port), timeout=15)
            except OSError:
                conn.sendall(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")
                conn.close()
                return
            upstream.settimeout(None)
            conn.settimeout(None)
            conn.sendall(b"HTTP/1.1 200 Connection Established\r\n\r\n")
            if rest:
                upstream.sendall(rest)
            splice(conn, upstream)
            return
        log.write("refused", host, port)
        conn.sendall(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
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
