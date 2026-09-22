// SPDX-License-Identifier: Apache-2.0
//! `Plan::render` reaches the roles HW-DR-0045's palette gives an import
//! report, and `ColorMode::Plain` writes not one escape byte. #479.

mod chain;

use chain::{declaration, tree, Scratch};
use headwater_check::paint::ColorMode;

/// The document the fixture snapshot's one link targets.
const DECLARING: &str = "docs/spec/00-first.md";

#[test]
fn each_line_reaches_the_role_the_palette_gives_it() {
    struct Case {
        what: &'static str,
        opens_with: String,
    }
    let scratch = Scratch::new("palette-import-plan");
    let digest = tree(&scratch);
    let pinned = declaration(&digest);
    let read = chain::read(&scratch);
    let index = read.index();
    let view = headwater_import::Corpus {
        index: &index,
        relations: &read.relations,
        shape: &read.shape,
    };
    let plan = headwater_import::plan(scratch.path(), &pinned, None, &view).expect("it plans");

    let cases = [
        Case {
            what: "the report opens on a heading, bold in the default color",
            opens_with: "\u{1b}[1mimport\u{1b}[0m ado from".to_string(),
        },
        Case {
            what: "the declaring document's path is a path, cyan",
            opens_with: format!("(\u{1b}[36m{DECLARING}\u{1b}[0m)"),
        },
    ];
    let painted = plan.render(ColorMode::Ansi);
    for case in cases {
        assert!(
            painted.contains(&case.opens_with),
            "{}: no `{}` in\n{painted}",
            case.what,
            case.opens_with.escape_debug()
        );
    }
    assert!(!plan.render(ColorMode::Plain).contains('\x1b'));
}
