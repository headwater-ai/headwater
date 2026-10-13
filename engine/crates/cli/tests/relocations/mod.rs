// SPDX-License-Identifier: Apache-2.0
//! The network symbols a built binary asks the dynamic linker for.
//!
//! The question `no_fetch.rs` asks of the binary is whether it can open a
//! network socket (#1561). The dynamic symbol table cannot answer it: the
//! precompiled objects of `std` keep `socket`, `connect` and `getaddrinfo` in
//! that table even after `--gc-sections` drops every function that calls them,
//! so `nm -D` lists all three in a binary that reaches none of them. The
//! dynamic relocations answer it. The linker writes a relocation against a
//! symbol only where code that survived the link refers to it, so a binary
//! with none against these three has no path that calls them.
//!
//! The reader is `readelf -r -W` from binutils, which a host that links Rust
//! with the system `cc` has. When the tool is absent the case fails and says
//! so, because a reader that saw nothing would pass every binary.
//!
//! Two suites read the same list. `no_fetch.rs` asks it of the binary built
//! without the fetch and wants none. `network_boundary.rs` asks it of the
//! default binary and wants all three, which is what proves that this reader
//! sees a relocation when there is one.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// The calls that resolve a host name and open a network socket. A binary
/// that refers to none of them cannot make a network connection through libc.
///
/// `socketpair` is not here, and that is deliberate. The child-process code of
/// `std` (`Command::spawn`, which the CLI calls to run `git`) refers to it to
/// make a local pair between parent and child. That pair has no address and
/// reaches no network, and every binary that spawns a process carries it.
pub(crate) const NETWORK: [&str; 3] = ["socket", "connect", "getaddrinfo"];

/// The members of [`NETWORK`] that a dynamic relocation of `binary` names.
pub(crate) fn network_relocations(binary: &Path) -> BTreeSet<String> {
    let output = Command::new("readelf")
        .args(["-r", "-W"])
        .arg(binary)
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "`readelf` does not run ({error}), so the relocations of {} cannot be read; \
                 install binutils",
                binary.display()
            )
        });
    assert!(
        output.status.success(),
        "`readelf -r -W {}` failed: {}",
        binary.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.contains("Relocation section"),
        "`readelf -r -W {}` printed no relocation section, so this reader reads nothing: {text}",
        binary.display()
    );
    names(&text)
        .into_iter()
        .filter(|name| NETWORK.contains(&name.as_str()))
        .collect()
}

/// Every symbol name a relocation line of `readelf -r -W` names, without its
/// version suffix. A line reads `<offset> <info> <type> <value> <name>@<ver> +
/// <addend>`, and a relative relocation names no symbol at all.
fn names(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 5 || !fields[2].starts_with("R_") {
                return None;
            }
            let symbol = fields[4];
            let name = symbol.split('@').next().unwrap_or(symbol);
            Some(name.to_string())
        })
        .collect()
}

#[test]
fn a_versioned_symbol_is_read_by_its_bare_name_and_socketpair_is_not_socket() {
    let text = "\
Relocation section '.rela.dyn' at offset 0x1 contains 3 entries:
    Offset             Info             Type               Symbol's Value  Symbol's Name + Addend
0000000000001000  0000000100000006 R_X86_64_GLOB_DAT      0000000000000000 socket@GLIBC_2.2.5 + 0
0000000000001008  0000000200000006 R_X86_64_GLOB_DAT      0000000000000000 socketpair@GLIBC_2.2.5 + 0
0000000000001010  0000000000000008 R_X86_64_RELATIVE                    2000
";
    let read = names(text);
    assert!(read.contains("socket"), "{read:?}");
    assert!(read.contains("socketpair"), "{read:?}");
    assert!(!read.contains("connect"), "{read:?}");
    assert_eq!(read.len(), 2, "a relative relocation names no symbol: {read:?}");
}
