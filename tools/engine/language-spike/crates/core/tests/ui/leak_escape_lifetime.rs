//! Leak 3: smuggle the borrow out so it outlives the run.
//!
//! This is the class Go cannot express at all. A check that stashes borrowed
//! graph data and reads it on a later invocation would see a stale corpus, and
//! nothing about the stashing is visible at the call site.

use headwater_core::view::DocumentView;

fn smuggle<'g>(view: &DocumentView<'g>) -> &'static str {
    // `path()` borrows for 'g, which is not 'static.
    view.path()
}

fn main() {
    let _ = smuggle;
}
