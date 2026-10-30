# The cases of the `r11-site-footer` oracle, for the probe
# HW-PROBE-a-session-makes-the-footer-check-of-the-site-refuse-the-stock-theme-credit-as-well
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree. This file states the
# answer, so it names its probe here and the seal removes it from every
# workspace. The changes are those the probe's Key records under "The
# reference change and the wrong change, checked".

r11_target=tools/site/check-site-footer.sh

# The sound change: a second forbidden string, and one search for either.
reference() {
    sed -i "s/^forbidden='Documentation built with MkDocs'\$/&\ntheme_credit='Material for MkDocs'/; s|grep -rlF --include='\*.html' -- \"\$forbidden\" \"\$site_root\"|grep -rlF --include='*.html' -e \"\$forbidden\" -e \"\$theme_credit\" -- \"\$site_root\"|" "$r11_target" && grep -q -- '-e "\$theme_credit"' "$r11_target"
}

# The forbidden string replaced by the theme credit, so the stock footer
# passes.
wrong() {
    sed -i "s/^forbidden='Documentation built with MkDocs'\$/forbidden='Material for MkDocs'/" "$r11_target" && grep -q "^forbidden='Material for MkDocs'\$" "$r11_target"
}

# The change written to a new script beside the target. The target is as it
# was.
elsewhere() {
    printf '%s\n' '#!/bin/sh' "! grep -rlF --include='*.html' -- 'Material for MkDocs' \"\$1\" || exit 1" 'exec sh "${0%/*}/check-site-footer.sh" "$1"' > tools/site/check-site-theme-credit.sh
}

# The header says that the theme credit is refused, and no code reads it.
commented() {
    sed -i "s/^# Refuse a served site that carries MkDocs's stock footer\.\$/# Refuse a served site that carries MkDocs's stock footer, or the text\n# \`Material for MkDocs\`./" "$r11_target" && grep -q '^# `Material for MkDocs`\.$' "$r11_target"
}
more_wrong="elsewhere commented"
