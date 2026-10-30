# The cases of the `r11-apt-block` oracle, for the probe
# HW-PROBE-a-session-makes-the-script-that-prints-the-apt-block-remove-sudo-with-its-environment-flag-as-a-whole
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The changes are those the probe's Key records under "The
# reference change and the wrong change, checked".

r11_target=tools/repo/readme-apt-block.sh

# The sound change: `sudo -E ` goes first, and then `sudo `.
reference() {
    sed -i "s|^' \"\$readme\" \| sed 's/sudo //g')\$|' \"\$readme\" \| sed 's/sudo -E //g; s/sudo //g')|" "$r11_target" && grep -q "sed 's/sudo -E //g; s/sudo //g'" "$r11_target"
}

# Every `-E ` removed after `sudo `, which also removes the flag of `grep -E`.
wrong() {
    sed -i "s|^' \"\$readme\" \| sed 's/sudo //g')\$|' \"\$readme\" \| sed 's/sudo //g; s/-E //g')|" "$r11_target" && grep -q "s/-E //g'" "$r11_target"
}

# The change written to a new script beside the target. The target is as it
# was.
elsewhere() {
    printf '%s\n' '#!/bin/sh' 'sh "${0%/*}/readme-apt-block.sh" "$@" | sed "s/^-E //"' > tools/repo/readme-apt-block-e.sh
}

# The header says that `sudo -E ` goes as a whole, and no code does it.
commented() {
    sed -i 's/^# and with every `sudo ` removed, because the paragraph$/# and with every `sudo -E ` and then every `sudo ` removed, because the paragraph/' "$r11_target" && grep -q 'every `sudo -E ` and then' "$r11_target"
}
more_wrong="elsewhere commented"
