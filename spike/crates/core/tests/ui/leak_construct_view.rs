//! Leak 2: build the wider view yourself.
//!
//! Suppose a check somehow holds a `&Graph`. It still cannot manufacture a
//! `CorpusView` over it, because the constructors are `pub(crate)`. This is the
//! leak that matters for the plugin surface, where the check is foreign code.

use headwater_core::model::Graph;
use headwater_core::view::{CorpusView, DocumentView};

fn main() {
    let graph = Graph::build(Vec::new());
    let _corpus = CorpusView::new(&graph);
    let _doc = DocumentView::new(graph.doc(0));
}
