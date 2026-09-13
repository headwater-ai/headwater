// A citation that rots the commonest way there is.
//
// This file is planted at `engine/crates/query/src/lib.rs`, which
// `docs/interfaces/headwater-explain.md` really governs, and the identifier
// below really resolves to that document. So the invented class is silent and
// the stale class is silent, both correctly. What is wrong is the path in the
// parentheses: it names a document that is not there.
//
// A document that is renamed or moved keeps its identifier and loses its path.
// The identifier still resolving is exactly what hides it, and the first
// version of this checker captured that path and never read it, so a run over
// this file exited 0 and printed "every citation resolves, and every cited
// document governs the file that cites it".
// per HW-IFACE-headwater-explain (docs/decisions/9999-a-document-that-does-not-exist.md)
pub fn nothing() {}
