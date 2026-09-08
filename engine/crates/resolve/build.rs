// SPDX-License-Identifier: Apache-2.0
//
// [HW-DR-0060](../../../docs/decisions/0060-the-engine-s-version-stays-one-number-and-a-build-s-exact-commit-is-a-separate-unwired-fact.md)
// is why this exists and what it does not wire into. `git describe` names the
// commit this build ran from, when one is there to name: a checkout of this
// repository has one, and the tarball `cargo publish` uploads does not,
// because that tarball carries no `.git` at all. Both cases are ordinary, so
// neither one fails this build. A missing `git` binary, a directory outside
// any repository and a `git describe` that finds no tag all take the same
// path: emit nothing, and `option_env!("HEADWATER_BUILD_DESCRIBE")` in
// `release.rs` reads that absence as `None` rather than as an empty string.
use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["describe", "--always", "--dirty", "--tags"])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let describe = String::from_utf8_lossy(&output.stdout);
            let describe = describe.trim();
            if !describe.is_empty() {
                println!("cargo:rustc-env=HEADWATER_BUILD_DESCRIBE={describe}");
            }
        }
    }
}
