#!/bin/sh
# What holds `tools/site/check-live-site.py`.
#
# Spec 12 refuses a check that ships with no failing fixture. Every refusal
# below is provoked on purpose, and so is every pass, because a gate that
# refuses everything is as useless as one that refuses nothing.
# `tools/site/site-noindex-fixtures.sh` is the shape.
#
# Run it from anywhere:
#     sh tools/site/site-live-fixtures.sh
#
# # HOW IT ASKS A SITE WITHOUT THE INTERNET
#
# The checker takes an origin, so each case serves a scratch tree from a small
# server on 127.0.0.1 that behaves as Cloudflare Workers static assets does:
# a file or a directory index answers 200, a directory named without its
# slash answers 307 to the slash, a rule of the tree's `_redirects` answers its
# status and `Location`, and anything else answers 404. A case that needs the
# host to ignore `_redirects` starts the server with `--no-redirects`, which is
# the failure the checker exists to catch after a deploy. `--skip-http-check`
# is passed because a loopback server speaks no TLS, so the http-to-https
# question has no honest answer here and is not asked.
#
# # THE CASE THIS SUITE EXISTS FOR
#
# Case 5: the host does not honour a rule of `_redirects`. Nothing in the
# repository can observe that before a deploy, and it is how a redirect file
# that is correct in every way a build can read goes on serving 404s.
#
# # WHAT IT WRITES
#
# Nothing inside this checkout. Every case runs under `mktemp -d`, and the
# server is stopped when the suite ends.

set -u

root=$(cd "$(dirname "$0")/../.." && pwd)
tool="$root/tools/site/check-live-site.py"

if [ ! -f "$tool" ]; then
    echo "no checker at \`tools/site/check-live-site.py\`." >&2
    exit 1
fi

scratch=$(mktemp -d) || exit 1
server_pid=""
stop() {
    if [ -n "$server_pid" ]; then
        kill "$server_pid" 2>/dev/null
        wait "$server_pid" 2>/dev/null
        server_pid=""
    fi
}
trap 'stop; rm -rf "$scratch"' EXIT HUP INT TERM

passed=0
failed=0

report() {
    name=$1
    want=$2
    got=$3
    ok=yes
    if [ "$want" != "$got" ]; then
        ok=no
        why="expected exit $want, got $got"
    fi
    if [ "$ok" = yes ] && [ $# -ge 5 ] && [ -n "$4" ]; then
        if ! grep -qF -- "$4" "$5"; then
            ok=no
            why="exit $got as expected, but the report never said: $4"
        fi
    fi
    if [ "$ok" = yes ]; then
        passed=$((passed + 1))
        echo "  ok    $name"
    else
        failed=$((failed + 1))
        echo "  FAIL  $name"
        echo "          $why"
    fi
}

cat >"$scratch/server.py" <<'PY'
import http.server, os, sys

root, portfile = sys.argv[1], sys.argv[2]
honour = "--no-redirects" not in sys.argv
rules = {}
path = os.path.join(root, "_redirects")
if honour and os.path.isfile(path):
    for line in open(path):
        p = line.split()
        if len(p) >= 2 and not line.lstrip().startswith("#"):
            rules[p[0]] = (p[1], int(p[2]) if len(p) > 2 else 302)


class H(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def reply(self, code, location=None, body=b""):
        self.send_response(code)
        if location:
            self.send_header("Location", location)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_HEAD(self):
        self.do_GET()

    def do_GET(self):
        p = self.path.split("?")[0]
        if p in rules:
            return self.reply(rules[p][1], rules[p][0])
        target = os.path.join(root, p.lstrip("/"))
        if os.path.isdir(target):
            if not p.endswith("/"):
                return self.reply(307, p + "/")
            target = os.path.join(target, "index.html")
        if os.path.isfile(target):
            return self.reply(200, body=open(target, "rb").read())
        self.reply(404, body=b"not found")


s = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
open(portfile, "w").write(str(s.server_address[1]))
s.serve_forever()
PY

# page DIR RELATIVE-DIR [BODY]
page() {
    mkdir -p "$1/$2"
    printf '<!DOCTYPE html>\n<html><head><title>%s</title></head><body>\n%s\n</body></html>\n' \
        "$2" "${3:-}" >"$1/$2/index.html"
}

# serve DIR [--no-redirects] : start the server on a free port, set $origin
serve() {
    stop
    rm -f "$scratch/port"
    python3 "$scratch/server.py" "$1" "$scratch/port" "${2:-}" >/dev/null 2>&1 &
    server_pid=$!
    n=0
    while [ ! -s "$scratch/port" ] && [ $n -lt 50 ]; do
        sleep 0.1
        n=$((n + 1))
    done
    origin="http://127.0.0.1:$(cat "$scratch/port")"
}

# tree DIR [LINK-HREF] : a three-page site with a sitemap that lists all three
tree() {
    rm -rf "$1"
    page "$1" . '<a href="/spec/">spec</a>'
    page "$1" spec "${2:-<a href=\"/spec/a/\">a</a>}"
    page "$1" spec/a '<a href="/">home</a>'
    printf '<urlset>\n<url><loc>ORIGIN/</loc></url>\n<url><loc>ORIGIN/spec/</loc></url>\n<url><loc>ORIGIN/spec/a/</loc></url>\n</urlset>\n' >"$1/sitemap.xml"
}

# run [ARG...] -> asks $origin, writes $scratch/out and $scratch/err, prints the status
run() {
    python3 "$tool" "$origin" --retries 0 --skip-http-check "$@" >"$scratch/out" 2>"$scratch/err"
    echo $?
}

t="$scratch/site"

# The sitemap names its own origin, which is only known once the server is up,
# so each case writes the tree, starts the server, then fixes the sitemap.
fix_sitemap() {
    sed -i "s|ORIGIN|$origin|g" "$t/sitemap.xml"
}

echo "against a served tree"

# 1. A clean site, with one redirect rule that the host honours.
tree "$t"
printf '/old/ /spec/a/ 301\n' >"$t/_redirects"
serve "$t"
fix_sitemap
status=$(run --redirects "$t/_redirects")
report "a clean site with an honoured redirect passes" 0 "$status"
if grep -qE '^3 sitemap URLs, [0-9]+ further link targets and 1 redirect rules? asked' "$scratch/out"; then
    passed=$((passed + 1)); echo "  ok    it states what it asked"
else
    failed=$((failed + 1)); echo "  FAIL  it states what it asked"
    sed 's/^/          /' "$scratch/out"
fi

# 2. A sitemap URL that is not served.
tree "$t"
sed -i 's|</urlset>|<url><loc>ORIGIN/spec/gone/</loc></url>\n</urlset>|' "$t/sitemap.xml"
: >"$t/_redirects"
serve "$t"
fix_sitemap
status=$(run --redirects "$t/_redirects")
report "a sitemap URL that answers 404 is refused" 1 "$status" "spec/gone/ answered 404" "$scratch/err"

# 3. A page that links to a path the site does not serve.
tree "$t" '<a href="/LICENSE">LICENSE</a>'
: >"$t/_redirects"
serve "$t"
fix_sitemap
status=$(run --redirects "$t/_redirects")
report "a link to a path that answers 404 is refused" 1 "$status" \
    "/LICENSE, which answered 404" "$scratch/err"

# 4. A link answered by a redirect to a served page passes.
tree "$t" '<a href="/old/">old</a>'
printf '/old/ /spec/a/ 301\n' >"$t/_redirects"
serve "$t"
fix_sitemap
status=$(run --redirects "$t/_redirects")
report "a link that a redirect answers passes" 0 "$status"

# 5. The host ignores a rule of `_redirects`.
tree "$t"
printf '/old/ /spec/a/ 301\n' >"$t/_redirects"
serve "$t" --no-redirects
fix_sitemap
status=$(run --redirects "$t/_redirects")
report "a redirect rule the host does not honour is refused" 1 "$status" \
    "redirect /old/ should answer 301 to /spec/a/; the host answered 404" "$scratch/err"

# 6. The host answers the rule with another status: it serves a 302 where the
#    file in the repository says 301.
tree "$t"
printf '/old/ /spec/a/ 302\n' >"$t/_redirects"
serve "$t"
fix_sitemap
printf '/old/ /spec/a/ 301\n' >"$scratch/expected"
status=$(run --redirects "$scratch/expected")
report "a redirect answered with the wrong status is refused" 1 "$status" \
    "should answer 301" "$scratch/err"

# 7. The host sends the rule to another page.
tree "$t"
printf '/old/ /spec/ 301\n' >"$t/_redirects"
serve "$t"
fix_sitemap
printf '/old/ /spec/a/ 301\n' >"$scratch/expected"
status=$(run --redirects "$scratch/expected")
report "a redirect that lands on another page is refused" 1 "$status" \
    "should answer 301 to /spec/a/" "$scratch/err"

# 7b. A host that never honours the rule is waited on for at most `--settle`
#     seconds and then asked anyway, so the wait is bounded and the refusal stands.
tree "$t"
printf '/old/ /spec/a/ 301\n' >"$t/_redirects"
serve "$t" --no-redirects
fix_sitemap
started=$(date +%s)
status=$(run --redirects "$t/_redirects" --settle 6)
elapsed=$(( $(date +%s) - started ))
report "a host that never settles is refused after a bounded wait" 1 "$status" \
    "the host answered 404" "$scratch/err"
if [ "$elapsed" -ge 4 ] && [ "$elapsed" -le 30 ]; then
    passed=$((passed + 1)); echo "  ok    the wait is bounded by --settle ($elapsed s)"
else
    failed=$((failed + 1)); echo "  FAIL  the wait is bounded by --settle ($elapsed s)"
fi

# 7c. A host that has settled is not waited on.
tree "$t"
printf '/old/ /spec/a/ 301\n' >"$t/_redirects"
serve "$t"
fix_sitemap
started=$(date +%s)
status=$(run --redirects "$t/_redirects" --settle 60)
elapsed=$(( $(date +%s) - started ))
report "a host that has settled passes" 0 "$status"
if [ "$elapsed" -le 10 ]; then
    passed=$((passed + 1)); echo "  ok    a settled host costs no wait ($elapsed s)"
else
    failed=$((failed + 1)); echo "  FAIL  a settled host costs no wait ($elapsed s)"
fi

# 7d. A page the deploy added reaches the host after the check starts. Without
#     `--settle` it is refused (case 2). With it the 404 is asked again until the
#     page answers, and a page that never appears is still refused after the wait.
tree "$t"
sed -i 's|</urlset>|<url><loc>ORIGIN/spec/late/</loc></url>\n</urlset>|' "$t/sitemap.xml"
: >"$t/_redirects"
serve "$t"
fix_sitemap
( sleep 5; page "$t" spec/late '<a href="/">home</a>' ) &
late_pid=$!
started=$(date +%s)
status=$(run --redirects "$t/_redirects" --settle 40)
elapsed=$(( $(date +%s) - started ))
wait "$late_pid" 2>/dev/null
report "a page that appears during the settle wait passes" 0 "$status"
if [ "$elapsed" -ge 4 ] && [ "$elapsed" -le 30 ]; then
    passed=$((passed + 1)); echo "  ok    it waited for the page and no longer ($elapsed s)"
else
    failed=$((failed + 1)); echo "  FAIL  it waited for the page and no longer ($elapsed s)"
fi

tree "$t"
sed -i 's|</urlset>|<url><loc>ORIGIN/spec/never/</loc></url>\n</urlset>|' "$t/sitemap.xml"
: >"$t/_redirects"
serve "$t"
fix_sitemap
started=$(date +%s)
status=$(run --redirects "$t/_redirects" --settle 6)
elapsed=$(( $(date +%s) - started ))
report "a page that never appears is refused after a bounded wait" 1 "$status" \
    "spec/never/ answered 404" "$scratch/err"
if [ "$elapsed" -ge 4 ] && [ "$elapsed" -le 30 ]; then
    passed=$((passed + 1)); echo "  ok    the wait for a missing page is bounded by --settle ($elapsed s)"
else
    failed=$((failed + 1)); echo "  FAIL  the wait for a missing page is bounded by --settle ($elapsed s)"
fi

# 8. A site that cannot be read is exit 2, not a pass over nothing.
stop
origin="http://127.0.0.1:9"
status=$(run --redirects "$t/_redirects")
report "an origin that answers nothing exits 2" 2 "$status" "nothing was checked" "$scratch/err"

echo
echo "$passed passed, $failed failed"
[ "$failed" -eq 0 ]
