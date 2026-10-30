# The cases of the `r9-review-marker` oracle, for the probe
# HW-PROBE-a-session-rewrites-the-change-narration-in-the-summary-of-the-obligation-on-the-site-review-marker-and-raises-no-other-finding
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".
target=docs/process/obligations/0217-the-site-review-marker-lives-in-the-common-directory-of-a-clone-so-one-worktree-s-unreviewed-merge-blocks-the-push-of-every-worktree.md
pin='^summary: "The pre-push hook read a site-review marker that every worktree of a clone shared. #1273 removed the marker, its writer and its reader, because the site pages no longer hold a fold."$'
first='The pre-push hook read a site-review marker that every worktree of a clone shared.'

# The sound change.
reference() {
    sed -i "s/$pin/summary: \"$first #1273 removed the marker, its writer and its reader, because the site pages hold no fold.\"/" "$target" && grep -q 'because the site pages hold no fold' "$target"
}

# The narration rewritten as one sentence of 31 words, which the language
# rule reports.
wrong() {
    sed -i "s/$pin/summary: \"$first #1273 removed the marker, its writer and its reader, because the site pages that the generator writes from the corpus hold no fold that a merge of two branches can break.\"/" "$target" && grep -q 'that a merge of two branches can break' "$target"
}

# The second sentence cut, which drops what #1273 removed. The summary line
# of the baseline fails it.
deleted() {
    sed -i "s/$pin/summary: \"$first\"/" "$target" && ! grep -q 'its writer and its reader' "$target"
}

# The second sentence cut, and kept as a comment in the front matter. The
# summary line fails it, because it reads no comment.
commented() {
    deleted && sed -i "s/^summary: \"$first\"\$/&\n# #1273 removed the marker, its writer and its reader/" "$target" && grep -q '^# #1273 removed the marker, its writer and its reader$' "$target"
}

# The mutants of round 4 of #1874.3 (parent-slice-3-stop.md, mutants 1 to 3
# applied to this probe). Each one keeps the text of the second sentence and
# takes it out of the summary, so a reader of lines passes it and the
# summary as the engine parses it does not.
#
# The second sentence cut, and appended as a line of the body.
body_line() {
    deleted && printf '\n#1273 removed the marker, its writer and its reader, because the site pages hold no fold.\n' >> "$target"
}

# The whole summary commented out, and a summary without the second sentence
# written below it.
summary_commented() {
    sed -i "s/$pin/# &\nsummary: \"$first\"/" "$target" && grep -q '^# summary: "The pre-push hook' "$target"
}

# The second sentence cut, and moved into the title.
other_key() {
    deleted && sed -i 's/^title: "The site-review marker lives in the common directory of a clone, so one worktree.s unreviewed merge blocks the push of every worktree"$/title: "The site-review marker blocked the push of every worktree, and #1273 removed the marker, its writer and its reader"/' "$target" && grep -q '^title: .*removed the marker, its writer and its reader' "$target"
}

# The file carries a second change-narration finding, in a body sentence of
# its Discharge. A session that repairs that sentence and leaves the summary
# clears one finding of the same rule and path, so the count matches the
# baseline. The `# summary-not:` line of the baseline fails it, because the
# summary the engine parses still holds the narrated words (#1874.3b).
body_repaired() {
    sed -i 's/so a `site\/\*` page no longer holds a fold and/so a `site\/*` page holds no fold and/' "$target" && grep -q 'page holds no fold and' "$target" && grep -q 'the site pages no longer hold a fold' "$target"
}
more_wrong="deleted commented body_line summary_commented other_key body_repaired"
