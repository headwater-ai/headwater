#!/bin/sh
# Install current stable Rust and make it the default, for a CI job.
#
# A "The toolchain this build needs" step of `.github/workflows/` runs this
# script when the job checks out the workflow's own commit: the three jobs of
# `ci.yml` and `yank-crates.yml`. The five steps that check out a tag, an input
# ref or a pull request's head carry the same fallback inline, because that
# ref can predate this script.
#
# Why the fallback: the self-hosted runner starts each job in a fresh
# container whose image already carries a stable toolchain in a read-only
# overlayfs layer. When a new stable is released, `rustup update` renames the
# old component directories into `~/.rustup/tmp`, and overlayfs refuses a
# rename of a directory from a lower layer with EXDEV ("Invalid cross-device
# link", os error 18). An uninstall deletes rather than renames, so the script
# then uninstalls stable and installs it again into the writable layer
# (measured on 2026-10-01, when 1.99.0 replaced the image's 1.98.1).
#
# On a hosted runner, and on the self-hosted runner when the image already
# has current stable, the update succeeds and the fallback costs nothing.
# If the update fails for another reason, such as the network, the install
# fails too and the step is red, as it was before this script.
set -eu

if ! rustup update --no-self-update stable; then
  echo "toolchain.sh: rustup update failed; reinstalling stable (overlayfs EXDEV on a self-hosted runner, see this script's header)" >&2
  rustup toolchain uninstall stable
  rustup toolchain install --no-self-update --profile default stable
fi
rustup default stable
