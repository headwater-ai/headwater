#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
#
# Install the headwater binary:
#
#     curl -fsSL https://headwater.tools/install.sh | sh
#
# It downloads the release archive for this machine from GitHub, checks it
# against the checksum published beside it, and unpacks the one binary into
# ~/.local/bin. It needs `sh`, `curl`, `tar` and `uname`, and it uses
# `sha256sum` or `shasum` to check the archive when either is present. It
# never asks for root and writes nothing outside the install directory and
# one temporary directory that it removes.
#
# Two settings, both optional:
#
#     HEADWATER_VERSION       the release tag to install (default below)
#     HEADWATER_INSTALL_DIR   where the binary goes (default ~/.local/bin)
#
#     curl -fsSL https://headwater.tools/install.sh | HEADWATER_VERSION=v0.5.0 sh
#
# Every other route, apt and cargo among them, is at
# https://headwater.tools/install/.
#
# The body is one function called on the last line, so a download cut off
# part way runs nothing.
#
# For this repository: `site/install.sh` is served byte for byte at
# https://headwater.tools/install.sh. Step 9 of `docs/how-to/cut-a-release.md`
# moves the default tag below, and `tools/repo/readme-fixtures.sh` holds it to
# the release the README installs.

set -eu

main() {
    version=${HEADWATER_VERSION:-v0.5.0}
    dir=${HEADWATER_INSTALL_DIR:-$HOME/.local/bin}

    os=$(uname -s)
    arch=$(uname -m)
    case "$os $arch" in
        "Linux x86_64" | "Linux amd64") target=x86_64-unknown-linux-musl ;;
        "Darwin arm64" | "Darwin aarch64") target=aarch64-apple-darwin ;;
        *)
            say "no release archive is built for $os $arch."
            say "Every other route is at https://headwater.tools/install/."
            exit 1
            ;;
    esac

    archive="headwater-$version-$target.tar.gz"
    url="https://github.com/headwater-ai/headwater/releases/download/$version/$archive"

    # `mkdir` with no -p fails on a path that already exists, so nothing
    # placed there in advance is written through.
    work="${TMPDIR:-/tmp}/headwater-install.$$"
    mkdir "$work"
    trap 'rm -rf "$work"' EXIT
    trap 'exit 1' HUP INT TERM

    say "downloading headwater $version for $target"
    if ! curl -fsSL -o "$work/$archive" "$url"; then
        say "could not download $url"
        say "Check the version: the releases are at https://github.com/headwater-ai/headwater/releases."
        exit 1
    fi
    curl -fsSL -o "$work/$archive.sha256" "$url.sha256"

    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$work" && sha256sum -c "$archive.sha256" >/dev/null) || refuse_checksum
    elif command -v shasum >/dev/null 2>&1; then
        (cd "$work" && shasum -a 256 -c "$archive.sha256" >/dev/null) || refuse_checksum
    else
        say "neither sha256sum nor shasum is installed, so the archive is not checked."
    fi

    mkdir -p "$dir"
    tar -xzf "$work/$archive" -C "$dir" headwater
    say "installed $("$dir/headwater" --version) at $dir/headwater"

    case ":$PATH:" in
        *":$dir:"*) ;;
        *) say "$dir is not on your PATH. Add it, or run $dir/headwater by its full path." ;;
    esac
    say "Next: the tutorial at https://headwater.tools/tutorial/"
}

say() {
    printf 'headwater install: %s\n' "$1"
}

refuse_checksum() {
    say "the archive does not match its published checksum, so nothing was installed."
    exit 1
}

main "$@"
