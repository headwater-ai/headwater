// SPDX-License-Identifier: Apache-2.0
//! Build the two pinned models into the binary, for the route variant of the
//! hook layer's tuning round 1 (#1777).
//!
//! A probe session runs in a sandbox that holds its workspace and nothing of
//! the host's, with no network (spec 15). `.headwater/models/` is ignored, so
//! no `git archive` carries a model into a workspace, and the campaign copies
//! the binary alone. So `headwater route` can read the embedding model and the
//! cross-encoder inside a session only if both are bytes of the binary.
//!
//! The files come from `HEADWATER_ROUTE_MODELS` when it is set, and otherwise
//! from `<git common dir>/headwater-models`, where `tools/embed/fetch-model.sh`
//! puts them: `model.onnx` and `vocab.txt` for the embedding pin, and
//! `reranker/model.onnx` and `reranker/vocab.txt` for the reranker pin. A
//! missing file fails the build, so no binary of this variant can route
//! without the reranker and fall back in silence. `headwater route` checks
//! each digest against the tree's own pins before it loads the bytes.

use std::path::{Path, PathBuf};
use std::process::Command;

const FILES: [(&str, &str); 4] = [
    ("HEADWATER_EMBED_GRAPH", "model.onnx"),
    ("HEADWATER_EMBED_VOCAB", "vocab.txt"),
    ("HEADWATER_RERANK_GRAPH", "reranker/model.onnx"),
    ("HEADWATER_RERANK_VOCAB", "reranker/vocab.txt"),
];

fn main() {
    println!("cargo:rerun-if-env-changed=HEADWATER_ROUTE_MODELS");
    let dir = models_dir();
    for (variable, name) in FILES {
        let path = dir.join(name);
        if !path.is_file() {
            panic!(
                "headwater-cli: `{}` is not a file. This build carries the pinned models in the \
                 binary (#1777), so fetch them first with `sh tools/embed/fetch-model.sh` and \
                 `sh tools/embed/fetch-model.sh --pin .headwater/reranker.yml`, or name their \
                 directory in HEADWATER_ROUTE_MODELS",
                path.display()
            );
        }
        let path = path
            .canonicalize()
            .expect("a file that exists canonicalizes");
        println!("cargo:rerun-if-changed={}", path.display());
        println!("cargo:rustc-env={variable}={}", path.display());
    }
}

fn models_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("HEADWATER_ROUTE_MODELS").filter(|dir| !dir.is_empty()) {
        return PathBuf::from(dir);
    }
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let common = Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(Path::new(&manifest))
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|dir| !dir.is_empty());
    match common {
        Some(common) => Path::new(&common).join("headwater-models"),
        None => panic!(
            "headwater-cli: this build carries the pinned models in the binary (#1777), and it \
             is not inside a git checkout, so it cannot find `<git common dir>/headwater-models`. \
             Name the directory in HEADWATER_ROUTE_MODELS"
        ),
    }
}
