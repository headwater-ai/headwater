# The cases of the `r5-check-layer-owl` oracle, for the probe
# HW-PROBE-a-session-records-the-owl-worked-example-as-evidence-of-the-check-layer-part-that-the-task-names
# (#1874).
#
# `tools/probe/probe-oracle-fixtures.sh` sources this file and runs each
# function from the root of a copy of the grading tree, with the engine at
# $HW_ORACLE_ENGINE. This file states the answer, so it names its probe here
# and the seal removes it from every workspace. The two changes are those
# the probe's Key records under "The reference change and the wrong change,
# checked".

# Add the target under the relation in the `relations:` block of the file,
# and the relation itself where the file does not declare it.
link() {
    python3 -I - "$1" "$2" "$3" <<'PY'
import sys
path, relation, target = sys.argv[1:]
text = open(path).read()
head, sep, body = text[4:].partition("\n---\n")
lines = head.split("\n")
if "relations:" not in lines:
    lines.append("relations:")
start = lines.index("relations:")
end = start + 1
while end < len(lines) and lines[end].startswith("  "):
    end += 1
block = lines[start + 1:end]
key = "  " + relation + ":"
if key in block:
    at = start + 1 + block.index(key) + 1
    while at < len(lines) and lines[at].startswith("    - "):
        at += 1
    lines.insert(at, "    - " + target)
else:
    lines[end:end] = [key, "    - " + target]
open(path, "w").write("---\n" + "\n".join(lines) + sep + body)
PY
}

# The sound change: both halves of the edge.
reference() {
    link docs/spec/12-check-layer.md cites_evidence HW-EVAL-owl-skos-worked-example && link docs/evaluations/owl-skos-worked-example.md cited_by HW-SPEC-check-layer
}

# The half on the evaluation, alone.
wrong() {
    link docs/evaluations/owl-skos-worked-example.md cited_by HW-SPEC-check-layer
}

# The mutants of #1874.3b. Each one writes the first half of the edge and
# the second half where the engine parses no edge, so a reader of the
# file's lines passes the commented one and `reciprocal-link.sh`, which
# reads each edge through `headwater explain --json`, fails both.
#
# Turn the line of the target under the relation in the `relations:` block
# into a comment, and the relation's own line too where no other target is
# left under it.
comment_out() {
    python3 -I - "$1" "$2" "$3" <<'PY'
import sys
path, relation, target = sys.argv[1:]
text = open(path).read()
head, sep, body = text[4:].partition("\n---\n")
lines = head.split("\n")
key = "  " + relation + ":"
at = lines.index(key)
end = at + 1
while end < len(lines) and lines[end].startswith("    "):
    end += 1
items = lines[at + 1:end]
item = items.index("    - " + target)
lines[at + 1 + item] = "    # - " + target
if not any(line.startswith("    - ") for line in lines[at + 1:end]):
    lines[at] = "  # " + relation + ":"
open(path, "w").write("---\n" + "\n".join(lines) + sep + body)
PY
}

# The reference change, with the second half as a commented relation line.
commented_edge() {
    reference && comment_out docs/evaluations/owl-skos-worked-example.md cited_by HW-SPEC-check-layer && grep -q '^    # - HW-SPEC-check-layer$' docs/evaluations/owl-skos-worked-example.md
}

# The first half, with the second half as a sentence of the body.
body_only() {
    link docs/spec/12-check-layer.md cites_evidence HW-EVAL-owl-skos-worked-example && printf '\nHW-SPEC-check-layer cites this evaluation as evidence.\n' >> docs/evaluations/owl-skos-worked-example.md
}
more_wrong="commented_edge body_only"
