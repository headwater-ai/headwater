// SPDX-License-Identifier: Apache-2.0
//! **Item 3, first half.** The core crate as a native Node addon.
//!
//! The point of this crate is what it does *not* contain. There is no child
//! process, no stdio protocol, no JSON envelope over a pipe, and no server.
//! Node loads this as a shared library and calls into it, which is the
//! arrangement spec 6 requires of editor integrations, the MCP server, and CI
//! adapters:
//!
//! > Editor integrations, the MCP server, and CI adapters all consume the
//! > library directly. They do not start a subprocess and parse text.

use headwater_core::finding::Finding;
use napi_derive::napi;

#[napi(object)]
pub struct JsFinding {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub severity: String,
    pub check: String,
    pub message: String,
    pub remediation: String,
    /// Present only when the fix is mechanical and total.
    pub fix: Option<String>,
}

fn convert(f: &Finding) -> JsFinding {
    JsFinding {
        path: f.path.clone(),
        line: f.span.start.line as u32,
        column: f.span.start.col as u32,
        severity: f.severity.as_str().to_string(),
        check: f.check.to_string(),
        message: f.message.clone(),
        remediation: f.remediation.clone(),
        fix: f.fix.as_ref().map(|x| x.replacement.clone()),
    }
}

/// Check one document. This is the call an editor makes on every keystroke
/// pause, which is why it must not cost a process start.
///
/// Note what this wrapper cannot do: it cannot construct a view, because
/// `DocumentView::new` is `pub(crate)` and this is a different crate. It asks
/// the core to perform a task. That is item 2's enforcement holding at the
/// embedding boundary, which is where foreign code actually lives.
#[napi]
pub fn check_document(path: String, source: String) -> Vec<JsFinding> {
    headwater_core::check_document_source(&path, &source)
        .iter()
        .map(convert)
        .collect()
}

/// Check a whole corpus that the host already has in memory.
#[napi]
pub fn check_corpus(files: Vec<Vec<String>>) -> Vec<JsFinding> {
    let pairs: Vec<(String, String)> = files
        .into_iter()
        .filter(|f| f.len() == 2)
        .map(|f| (f[0].clone(), f[1].clone()))
        .collect();
    let report = headwater_core::check_sources(&pairs, 0xfeed_face_dead_beef);
    report.findings.iter().map(convert).collect()
}
