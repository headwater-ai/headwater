//! **Item 3, second half.** The core crate on `wasm32-unknown-unknown`.
//!
//! Deliberately no `wasm-bindgen`. The claim under test is that *the core
//! compiles and runs on wasm32*, and glue code would both inflate the measured
//! artifact and hide whether the core itself is portable. A raw C ABI over
//! linear memory is the smallest thing that proves it, and it is what a browser
//! or an editor host instantiates directly.
//!
//! The core is built here with `default-features = false`, so the runner's
//! thread fan-out is compiled out. That is the one real concession wasm32
//! extracts: no threads without a cross-origin-isolated host and a shared
//! memory build.

use std::cell::RefCell;

thread_local! {
    /// Rendered findings from the last call, held so the host can read them out
    /// of linear memory. Single-threaded by construction on this target.
    static RESULT: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Allocate `len` bytes the host can write into.
///
/// # Safety
/// The host must pass the same `len` to `hw_dealloc`, or hand the pointer to a
/// function that consumes it.
#[no_mangle]
pub extern "C" fn hw_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// # Safety
/// `ptr` must come from `hw_alloc` with the same `len`.
#[no_mangle]
pub unsafe extern "C" fn hw_dealloc(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(Vec::from_raw_parts(ptr, 0, len));
    }
}

/// Check one document. Returns the byte length of the rendered result, which
/// the host then reads from `hw_result_ptr`.
///
/// # Safety
/// Both pointers must reference `hw_alloc`ed regions of the given lengths
/// holding valid UTF-8.
#[no_mangle]
pub unsafe extern "C" fn hw_check_document(
    path_ptr: *const u8,
    path_len: usize,
    src_ptr: *const u8,
    src_len: usize,
) -> usize {
    let path = std::str::from_utf8(std::slice::from_raw_parts(path_ptr, path_len)).unwrap_or("");
    let source = std::str::from_utf8(std::slice::from_raw_parts(src_ptr, src_len)).unwrap_or("");

    let findings = headwater_core::check_document_source(path, source);
    let rendered = findings
        .iter()
        .map(|f| f.render())
        .collect::<Vec<_>>()
        .join("\n");

    RESULT.with(|r| {
        let mut r = r.borrow_mut();
        *r = rendered;
        r.len()
    })
}

#[no_mangle]
pub extern "C" fn hw_result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

/// Number of documents the core can parse, exposed so a host can smoke-test the
/// module without constructing input.
#[no_mangle]
pub extern "C" fn hw_selftest() -> u32 {
    let doc = "---\nid: X\nkind: decision\nstatus: bogus\n---\n\n# X\n";
    headwater_core::check_document_source("x.md", doc).len() as u32
}
