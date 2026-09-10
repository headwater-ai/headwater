#!/bin/sh
# Scratch probe: where do the escape bytes on a pty run come from?
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
engine="$root/engine/target/dev-release/headwater"
cd "$root" || exit 1
esc=$(printf '\033')
for verb in "capture --root ." "infer --owner probe --root ."; do
    printf '== %s ==\n' "$verb"
    script -qec "$engine $verb 2>/dev/null" /dev/null > /tmp/probe.out 2>/dev/null
    printf 'lines with an escape: %s\n' "$(grep -c "$esc" /tmp/probe.out)"
    grep -n "$esc" /tmp/probe.out | head -3 | cat -v
done
