//! Leak 4: recover the concrete type by downcasting.
//!
//! This is the hole in the Go version of the same design. There, a view is an
//! interface and a type assertion recovers whatever concrete value sits behind
//! it. Here `Any` requires `'static`, and a view borrows for `'g`, so the cast
//! cannot even be attempted.

use headwater_core::view::DocumentView;
use std::any::Any;

fn try_downcast(view: &DocumentView<'_>) -> bool {
    let anyref: &dyn Any = view;
    anyref.is::<DocumentView<'static>>()
}

fn main() {
    let _ = try_downcast;
}
