//! Whether a declared path stays inside the directory it is read from.
//!
//! A taxonomy and a consumer declaration name paths that the engine later joins
//! to the repository root and reads or writes: a shelf path, the corpus root, a
//! projection output, a path a language regime lists outside the corpus root.
//! HW-DR-0084 clause 2 refuses an absolute path and a path that climbs out of
//! the repository, and #1967 holds every declared path to that rule. This is
//! the one test each reader runs, so that resolve, the lock reader and the
//! writers of `generate` and `new` cannot disagree about one path.
//!
//! The test is lexical. It opens no file, so a symlink inside the repository
//! that points outside it is not this test's to see, and a refused path is
//! refused before anything outside the tree is read.

/// How a declared path leaves the directory it is read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escape {
    /// The path starts at `/`, at `\`, or at a drive such as `C:`, so a join
    /// to the root discards the root.
    Absolute,
    /// Counted component by component, the `..` segments climb above the
    /// directory the path starts at.
    Climbs,
}

impl Escape {
    /// The clause a refusal states after the path.
    pub fn reason(self) -> &'static str {
        match self {
            Escape::Absolute => {
                "which is absolute, and a declared path is read from the repository root"
            }
            Escape::Climbs => "which climbs above the repository root with `..`",
        }
    }
}

/// How `path` leaves the directory it is read from, and nothing when it stays
/// inside it.
///
/// Git accepts `x/../y` and refuses a path that climbs above the root, so the
/// depth is counted component by component rather than by a search for `..`.
/// An empty path and `.` name the directory itself, which is inside it.
pub fn escapes(path: &str) -> Option<Escape> {
    let bytes = path.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    if path.starts_with('/') || path.starts_with('\\') || drive {
        return Some(Escape::Absolute);
    }
    let mut depth = 0usize;
    for component in path.split(['/', '\\']) {
        match component {
            "" | "." => {}
            ".." => match depth.checked_sub(1) {
                Some(up) => depth = up,
                None => return Some(Escape::Climbs),
            },
            _ => depth += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_inside_the_root_stays() {
        for path in [
            "docs/decisions/**",
            "docs",
            ".",
            "",
            "docs/../site/**",
            "./docs",
            "{shelf}/README.md",
        ] {
            assert_eq!(escapes(path), None, "{path}");
        }
    }

    #[test]
    fn an_absolute_path_escapes() {
        for path in ["/abs/**", "/etc", "/proc/self", "\\share", "C:/x", "c:x"] {
            assert_eq!(escapes(path), Some(Escape::Absolute), "{path}");
        }
    }

    #[test]
    fn a_climbing_path_escapes() {
        for path in [
            "..",
            "../sibling/**",
            "docs/../../x",
            "{shelf}/../../../pwned-rel.md",
            "docs\\..\\..\\x",
            "./..",
        ] {
            assert_eq!(escapes(path), Some(Escape::Climbs), "{path}");
        }
    }
}
