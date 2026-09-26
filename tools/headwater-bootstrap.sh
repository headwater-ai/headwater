#!/bin/sh
# headwater-bootstrap.sh — retired. It fetches nothing and runs nothing.
#
# This script fetched a tagged release over the network and handed the package
# to `headwater taxonomy vendor` as a directory, because `vendor` took a path
# and never a location. From v0.2.1, `vendor` takes the `https://` location of
# a published zip, fetches it itself, checks it against `--expect`, and writes
# the digest pin (#1063). So the script has no job left.
#
# It stays at this path for one release, because `site/tutorial/index.html`
# published a `curl` of its raw URL on `main` and people have copied that line
# (HW-DR-0077, `tools/README.md`). A reader who runs the old line gets the
# `vendor` command to run instead, and a non-zero exit, so that nothing that
# pipes this into `sh` reads a success it did not get.
#
# When the arguments name a `taxonomy/<package>/v<version>` tag, the command
# it prints is complete: a release of that tag names its asset
# `<package>-<version>.zip`, as `.github/workflows/release-taxonomy.yml`
# attaches it.

tag=
digest=
root=
while [ $# -gt 0 ]; do
    case $1 in
        --tag) tag=${2-} ;;
        --expect) digest=${2-} ;;
        --root) root=${2-} ;;
        *) shift; continue ;;
    esac
    [ $# -ge 2 ] && shift 2 || shift
done

base=https://github.com/headwater-ai/headwater/releases/download
case $tag in
    taxonomy/*/v*)
        rest=${tag#taxonomy/}
        package=${rest%%/*}
        version=${rest#*/v}
        url="$base/$tag/$package-$version.zip"
        ;;
    *)
        url="$base/taxonomy/<package>/v<version>/<package>-<version>.zip"
        ;;
esac

line="headwater taxonomy vendor $url --expect ${digest:-<digest>}"
if [ -n "$root" ]; then
    line="$line --root $root"
fi

{
    echo "headwater-bootstrap: this script is retired, and it fetched nothing."
    echo "  headwater v0.2.1 and later fetch a published taxonomy themselves. Run:"
    echo ""
    echo "    $line"
    echo ""
    echo "  Without network access, get the zip by a route your organization allows,"
    echo "  unpack it, and pass that directory in place of the location."
} >&2
exit 2
