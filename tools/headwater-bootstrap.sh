#!/bin/sh
# headwater-bootstrap.sh — fetch a published package over the network, and hand
# it to `headwater taxonomy vendor` as the directory that verb already takes.
#
# WHY THIS SCRIPT EXISTS AND NOT A FLAG ON `vendor`
#
#   HW-REQ-0001 holds that no crate under `engine/` opens a socket, and its
#   scope is the whole of a run and not check time alone: `headwater --version`
#   proves nothing about which subcommand asked, because the property that
#   HW-AC-0001 inspects is a fact about the compiled binary and the locked
#   dependency set, not about which code path a run took. A network-capable
#   crate in `engine/Cargo.lock` would sit in the same binary that
#   `.githooks/pre-commit` runs on every commit, whether or not `vendor` is the
#   verb that reached it. So `vendor` stays what its own `--help` already says:
#   it "reads a fetched artifact into the package area after digest
#   validation," and it fetches nothing.
#
#   Spec 5 draws the boundary this script sits on the far side of. A probe run
#   reaches the network through a recorder, a separate process the engine
#   plans for and reads a transcript back from. A coherence sweep reaches a
#   model through the agent that runs it, and the engine writes the briefing
#   and reads the finding back. Neither puts a socket inside `engine/crates`.
#   This script is the same shape for setup: it is the harness, in the sense
#   HW-REQ-0001 uses that word ("a harness that runs the engine may reach a
#   network for its own reasons, and this requirement says nothing about the
#   harness"), and `vendor` is still the only thing that writes into
#   `packages/`.
#
# WHAT IT DOES
#
#   1. Resolves a tag, the latest release if `--tag` is not given.
#   2. Fetches that tag's source archive from GitHub into a scratch directory
#      nothing here leaves behind, and extracts one package out of it — the
#      same bytes a `git clone` at that tag would have put on disk, with no
#      working tree, no history and no Rust toolchain needed to get them.
#   3. Runs `headwater taxonomy vendor` on the extracted directory, exactly as
#      the README's *Obtaining a named version* section already tells a reader
#      to do by hand.
#
#   It changes nothing about what `vendor` checks. `--expect` is required
#   here for the same reason it is not defaulted from inside the artifact over
#   there: the digest has to come from a channel the artifact does not
#   control. Read it off the release page, the same way a person following the
#   README does, and pass it through.
set -eu

repo="headwater-ai/headwater"
package="packages/headwater-standard"
tag=""
digest=""
root="."
bin="${HEADWATER_BIN:-headwater}"

usage() {
    cat >&2 <<'EOF'
Usage: headwater-bootstrap.sh --expect <digest> [--tag <tag>] [--package <path>] [--root <dir>]

  --expect <digest>   required. The `release.digest` value stated on the
                       release page for --tag, e.g. sha256:...
  --tag <tag>          the release tag to fetch, e.g. v0.1.0. Defaults to the
                       repository's latest release.
  --package <path>     the package's path inside the repository at that tag.
                       Defaults to packages/headwater-standard.
  --root <dir>         the consumer repository to vendor into. Passed to
                       `headwater taxonomy vendor --root`. Defaults to the
                       current directory.

HEADWATER_BIN selects the `headwater` binary this script hands off to. It
defaults to `headwater` on PATH.
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --tag) tag=$2; shift 2 ;;
        --expect) digest=$2; shift 2 ;;
        --package) package=$2; shift 2 ;;
        --root) root=$2; shift 2 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "headwater-bootstrap: unrecognized argument: $1" >&2; usage; exit 1 ;;
    esac
done

if [ -z "$digest" ]; then
    echo "headwater-bootstrap: --expect <digest> is required" >&2
    echo "  read it off the release page named below, not out of anything this script fetches" >&2
    usage
    exit 1
fi

for cmd in curl tar; do
    command -v "$cmd" >/dev/null 2>&1 || {
        echo "headwater-bootstrap: \`$cmd\` is not on the path." >&2
        exit 1
    }
done

command -v "$bin" >/dev/null 2>&1 || {
    echo "headwater-bootstrap: no \`headwater\` binary at \`$bin\`." >&2
    echo "  build or install one first; this script only fetches, it does not build" >&2
    exit 1
}

scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT INT TERM

if [ -z "$tag" ]; then
    command -v jq >/dev/null 2>&1 || {
        echo "headwater-bootstrap: \`jq\` is not on the path, and no \`--tag\` was given." >&2
        echo "  pass --tag explicitly, or install jq so the latest release can be resolved" >&2
        exit 1
    }
    tag=$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | jq -r '.tag_name')
    if [ -z "$tag" ] || [ "$tag" = "null" ]; then
        echo "headwater-bootstrap: could not resolve the latest release tag of $repo" >&2
        exit 1
    fi
    echo "headwater-bootstrap: no --tag given, resolved latest release: $tag" >&2
fi

archive_url="https://codeload.github.com/$repo/tar.gz/refs/tags/$tag"
echo "headwater-bootstrap: fetching $package at $tag" >&2

curl -fsSL "$archive_url" | tar -xz -C "$scratch" --wildcards "*/$package/*" || {
    echo "headwater-bootstrap: fetch of $archive_url failed, or it holds no $package" >&2
    exit 1
}

fetched=$(find "$scratch" -type d -path "*/$package" -print -quit)
if [ -z "$fetched" ] || [ -z "$(ls -A "$fetched" 2>/dev/null)" ]; then
    echo "headwater-bootstrap: $tag carries no $package" >&2
    exit 1
fi

"$bin" taxonomy vendor "$fetched" --expect "$digest" --root "$root"
