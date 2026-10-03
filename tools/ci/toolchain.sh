#!/bin/sh
# Install the Rust toolchain that `tools/ci/rust-version` pins and make it the
# default, for a CI job.
#
# A "The toolchain this build needs" step of `.github/workflows/` runs this
# script when the job checks out the workflow's own commit: the three jobs of
# `ci.yml` and `yank-crates.yml`. The five steps that check out a tag, an input
# ref or a pull request's head carry their own inline install of current
# stable, because that ref can predate this script, and they build a release
# rather than judge a branch.
#
# Why a pin and not "current stable". The jobs that judge a branch used to
# install current stable on every run. A new stable then changed the verdict of
# a branch nobody had touched: its clippy knows lints the last one did not, and
# a host cannot reproduce that, because no host runs the version CI picks on the
# day. `tools/engine/clippy.sh` reads the same file, so a clippy run there is
# the clippy run here (HW-PD-0001's rule: one owner for the number). Moving the
# pin is one reviewed edit to `tools/ci/rust-version`, in a pull request that
# also repairs whatever the new version reports. The floor the crates declare in
# `[workspace.package]` is separate, and `engine/README.md` holds that one.
#
# Why `toolchain install` of an exact version and not `rustup update`: the
# self-hosted runner starts each job in a fresh container whose image already
# carries a toolchain in a read-only overlayfs layer. When a version moves,
# `rustup update` renames the old component directories into `~/.rustup/tmp`,
# and overlayfs refuses a rename of a directory from a lower layer with EXDEV
# ("Invalid cross-device link", os error 18; measured on 2026-10-01, when 1.99.0
# replaced the image's 1.98.1). Installing a named version writes a new
# directory into the writable layer and renames nothing, and the check below
# skips the install when the default toolchain already is that version. The
# cost of a pin: once the runner image's stable moves past it, each job
# downloads the pinned version into its fresh container. That ends when the pin
# moves, or when the image carries the pinned version.
#
# If the install fails for another reason, such as the network, the step is red,
# as it was before this script.
set -eu

here=$(cd "$(dirname "$0")" && pwd)
version=$(tr -d '[:space:]' < "$here/rust-version")
case $version in
  [0-9]*.[0-9]*.[0-9]*) ;;
  *)
    echo "toolchain.sh: tools/ci/rust-version holds '$version', not a version like 1.99.0" >&2
    exit 1
    ;;
esac

if cargo --version 2>/dev/null | grep -q "^cargo $version "; then
  echo "toolchain.sh: the default toolchain is already $version"
  exit 0
fi

rustup toolchain install --no-self-update --profile default "$version"
rustup default "$version"
