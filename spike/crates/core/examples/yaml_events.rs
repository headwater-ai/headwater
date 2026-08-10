//! The probe that settled two questions the crate documentation answers wrongly
//! or not at all. Kept because both findings are load-bearing for `frontmatter`
//! and neither is guessable.
//!
//!     cargo run --example yaml_events
//!
//! 1. **Columns are 0-indexed**, although `Marker::col`'s doc comment says
//!    "(1-indexed)". Every column in a finding is off by one if you trust it.
//! 2. **`key:` with no value arrives as a plain `~` scalar**, not as an empty
//!    string, while `key: ""` arrives as a double-quoted empty string. Only the
//!    scalar style separates "left blank" from "explicitly empty", and a
//!    required-facet check has to tell them apart.

fn main() {
    let yaml = "id: DR-0001\nempty_facet:\nquoted: \"\"\ntilde: ~\n";
    println!("input:\n{yaml}");
    println!("{:<48} line col", "event");
    for item in saphyr_parser::Parser::new_from_str(yaml) {
        let (ev, span) = item.unwrap();
        println!(
            "{:<48} {:>4} {:>3}",
            format!("{ev:?}"),
            span.start.line(),
            span.start.col()
        );
    }
    println!("\nNote: `id` is at column 0, not 1. The doc comment is wrong.");
}
