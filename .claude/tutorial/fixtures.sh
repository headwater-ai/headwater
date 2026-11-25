#!/bin/sh
# The tutorial says what a reader should now see, forty-odd times. This runs it.
#
# `docs/tutorials/your-first-governed-corpus.md` is the on-ramp, and its bar is
# that an agent can drive it end to end with no context beyond the page. Every
# output block in it is a claim about this engine, and a claim in prose goes
# stale the day the engine changes under it. `.claude/tutorial/drive.py` reads the
# commands out of the document, runs them against a scratch repository under the
# temporary directory, and diffs each result against the block the document
# prints.
#
# It also runs `.claude/tutorial/adopter_interface.py` over that document and
# `README.md`, which checks a narrower and separate claim: that neither one
# hands a newcomer a script this repository wrote in place of the verb that
# does the same job (HW-DR-0072, #933). That check reads no engine output, so
# it runs first and needs no binary of its own.
#
# It blocks in CI. A tutorial that a newcomer cannot follow is worse than no
# tutorial, because the newcomer concludes the tool is broken rather than the
# page, and nothing else here reports the drift.
#
# It writes nothing inside this checkout: the scratch repository is a temporary
# directory, `HOME` is redirected into it, and the directory is removed at the
# end.
# The engine this suite drives, resolved the way `tools/repo/resolve-engine.sh`
# resolves it: either profile counts and the newer answers, `HEADWATER_BIN`
# still overrides both. This suite read the `release` path alone until #647,
# so a worktree built the way the build order tells a session to build it —
# `--profile dev-release` and not `--release` — had no `release` binary,
# this suite reported "no engine" against a real one sitting beside it, and
# CI's own gate never noticed because CI builds `--release` for itself.
#
# `HEADWATER_TUTORIAL_RELEASE=1` holds the page to the release it installs and
# not to the engine of this tree (#1985). The page's first block downloads one
# release archive, and a reader who follows it runs that release. So a page
# that matches the default branch and not that release is wrong for every
# reader. In this mode the script reads the tag out of the page's own download
# line, downloads the Linux archive and its `.sha256` file, verifies the
# archive, and drives the tutorial with the binary inside it. `drive.py` then
# also requires that `headwater --version` prints the version the page's
# "This installs version" line names. CI's tutorial step runs this mode. A
# `HEADWATER_BIN` set by the caller still answers in this mode, so a person
# can hold a page to a binary they already have, and the version claim says
# whether it is the release the page installs. Without the switch the script
# drives the engine of this tree, as before.
set -eu

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
. "$root/tools/repo/resolve-engine.sh"
python3 "$root/.claude/tutorial/adopter_interface.py" "$root"

# Release mode holds its own version claim first: a binary that prints a
# version the page does not install must stop the walk with the claim named.
# The stand-in answers every command with `0.0.0`, and the walk runs no
# command before `headwater --version`, so this case reaches no network. The
# walk must stop there and not reach step 1.
if [ "${HEADWATER_TUTORIAL_RELEASE:-}" = 1 ]; then
  wrong=$(mktemp -d "${TMPDIR:-/tmp}/headwater-tutorial-wrong.XXXXXX")
  printf '#!/bin/sh\necho 0.0.0\n' > "$wrong/headwater"
  chmod +x "$wrong/headwater"
  refused=0
  env -u HEADWATER_TUTORIAL_KEEP HEADWATER_BIN="$wrong/headwater" python3 "$root/.claude/tutorial/drive.py" "$root" > "$wrong/out" 2>&1 || refused=1
  if [ "$refused" = 1 ] && grep -qF 'FAIL before you start: the engine is the release the page installs' "$wrong/out" &&
    ! grep -q '^[a-zA-Z ]*step 1:' "$wrong/out"; then
    echo "ok   release mode: a binary that prints 0.0.0 stops the walk at the version claim"
    rm -rf "$wrong"
  else
    echo "FAIL release mode: a binary that prints 0.0.0 did not stop the walk at the version claim" >&2
    sed 's/^/  /' "$wrong/out" >&2
    rm -rf "$wrong"
    exit 1
  fi
fi

bin=${HEADWATER_BIN:-}
fetched=
if [ -z "$bin" ] && [ "${HEADWATER_TUTORIAL_RELEASE:-}" = 1 ]; then
  page="$root/docs/tutorials/your-first-governed-corpus.md"
  # The first archive URL on the page is block 0's `curl` line.
  url=$(grep -o 'https://github.com/headwater-ai/headwater/releases/download/v[0-9][0-9.]*/headwater-v[0-9][0-9.]*-x86_64-unknown-linux-musl\.tar\.gz' "$page" | head -n 1) || url=
  if [ -z "$url" ]; then
    echo "tutorial fixtures: release mode found no x86_64-unknown-linux-musl archive URL in $page" >&2
    exit 1
  fi
  fetched=$(mktemp -d "${TMPDIR:-/tmp}/headwater-tutorial-release.XXXXXX")
  archive=${url##*/}
  echo "tutorial fixtures: release mode, driving $archive" >&2
  if ! (cd "$fetched" && curl -fsSLO "$url" && curl -fsSLO "$url.sha256" && sha256sum -c "$archive.sha256" >/dev/null && tar -xzf "$archive" headwater); then
    echo "tutorial fixtures: could not download, verify and unpack $url" >&2
    rm -rf "$fetched"
    exit 1
  fi
  bin=$fetched/headwater
fi
if [ -z "$bin" ]; then
  bin=$(hw_resolve_engine_bin "$root") || bin=
fi

if [ -z "$bin" ] || [ ! -x "$bin" ]; then
  echo "tutorial fixtures:" >&2
  hw_resolve_engine_missing_message "$root" >&2
  exit 1
fi

# `HEADWATER_TUTORIAL_KEEP_ENGINE` names a directory to copy the engine this
# walk drives into, beside the corpus `HEADWATER_TUTORIAL_KEEP` keeps. A step
# that reads the kept corpus runs this engine, because the corpus asks for the
# engine range of the binary that wrote it. In release mode that is the
# release the page installs, and the engine of this tree refuses that range
# once a version bump moves it, so the kept corpus and the engine of this tree
# disagree for every release pull request.
if [ -n "${HEADWATER_TUTORIAL_KEEP_ENGINE:-}" ]; then
  mkdir -p "$HEADWATER_TUTORIAL_KEEP_ENGINE"
  cp "$bin" "$HEADWATER_TUTORIAL_KEEP_ENGINE/headwater"
fi

if [ -z "$fetched" ]; then
  HEADWATER_BIN=$bin exec python3 "$root/.claude/tutorial/drive.py" "$root"
fi
status=0
HEADWATER_BIN=$bin python3 "$root/.claude/tutorial/drive.py" "$root" || status=$?
rm -rf "$fetched"
exit "$status"
