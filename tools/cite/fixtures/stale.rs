// The decisive case, and the one the issue was filed about.
//
// This file is planted at `engine/crates/query/src/mcp.rs`, which
// `docs/interfaces/headwater-mcp.md` really does govern. The citation names
// spec 5, which really does resolve. Both halves of a naive check pass: the
// identifier is not invented, and the file is not ungoverned. What is wrong is
// the pair — spec 5 governs `.claude/hooks/write.sh` and not this file, so the
// citation was either never true or stopped being true when an edge moved.
//
// A checker that found only the invented class would report nothing here and
// still look green.
// per HW-SPEC-ai-integration (docs/spec/05-ai-integration.md)
pub fn nothing() {}
