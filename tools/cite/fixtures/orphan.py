# The other half of the stale class. This file is planted where no document
# declares a `governs` edge at all, so the governing set is empty rather than
# wrong, and the finding has to say so in different words: a reader who is told
# "docs/interfaces/headwater-mcp.md governs this file" can move the citation,
# and a reader whose file nothing governs cannot.
#
# It is also where HW-OBL-0104 bites hardest. A corpus that declared one edge
# onto a directory rather than one edge per file reports every file inside it
# exactly like this one, and every report is wrong.
# per HW-SPEC-ai-integration (docs/spec/05-ai-integration.md)
value = 1
