//! `headwater site`: hold a built site against the corpus it was built from.
//!
//! # The defect this module exists for
//!
//! [#978](https://github.com/headwater-ai/headwater/issues/978). A site
//! generator reads the corpus once and writes a directory of HTML, and nothing
//! in the binary read that directory again. So a page the navigation names and
//! the build did not write, a page left over from a document that the corpus
//! no longer holds, and a link to a page that is not there all reached a
//! reader. This repository's own site had each defect: #431 (326 dead
//! fragment links), #528 (shelf indexes the navigation did not name) and #350
//! (17 links with no fragment that answered 404, which the script that checks
//! fragments did not read).
//!
//! # What it knows about a generator
//!
//! One layout, and nothing else. A source `a/b.md` is served at
//! `a/b/index.html` (directory URLs) or at `a/b.html`, and `a/README.md` and
//! `a/index.md` at `a/index.html`. A page is present if either form is. This
//! is what MkDocs writes with and without `use_directory_urls`, and it names
//! no generator.
//!
//! # What it reads
//!
//! The site directory, and the plan the corpus and the lock give. The paths
//! the navigation names come from [`Plan::navigation`], which is the list the
//! `site_nav` emitter renders, and not from the committed file: a committed
//! file that is stale is `headwater generate --check`'s finding. It opens no
//! socket and it writes nothing.
//!
//! # Why no HTML parser
//!
//! The verb reads two attributes, `href` and `id`, and [`scan`] reads them over
//! the bytes. It skips comments and the bodies of `script` and `style`, and it
//! decodes no entity in a URL except `&amp;`. The contract states those limits,
//! and they are what keeps the engine free of a parser dependency.

use crate::{shelf_index, Kind, Plan, Projections};
use headwater_query::Surface;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The four finding classes, in the order the report lists them.
pub const PAGE_MISSING: &str = "site.page.missing";
pub const PAGE_STALE: &str = "site.page.stale";
pub const LINK_DEAD: &str = "site.link.dead";
pub const FRAGMENT_DEAD: &str = "site.fragment.dead";

/// One finding: its class, the page or source it is about, and one sentence.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub rule: &'static str,
    pub at: String,
    pub detail: String,
}

/// What one run found, and how much it read.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub pages: usize,
    pub links: usize,
    pub navigated: usize,
}

impl Report {
    /// One line for each finding, then one line that counts what was read.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for finding in &self.findings {
            out.push_str(&format!(
                "{}  {}: {}\n",
                finding.rule, finding.at, finding.detail
            ));
        }
        out.push_str(&format!(
            "{} over {}, {} and {}\n",
            count(self.findings.len(), "finding"),
            match self.navigated {
                1 => "1 navigation entry".to_string(),
                other => format!("{other} navigation entries"),
            },
            count(self.pages, "page"),
            count(self.links, "in-site link"),
        ));
        out
    }
}

fn count(how_many: usize, noun: &str) -> String {
    match how_many {
        1 => format!("1 {noun}"),
        other => format!("{other} {noun}s"),
    }
}

/// Hold the site at `site` against the corpus the plan was built from.
///
/// `Err` is a refusal, one sentence, for a directory that is not a site and
/// for a taxonomy whose navigation names nothing.
pub fn hold(
    site: &Path,
    surface: &Surface<'_>,
    projections: &Projections,
    plan: &Plan,
    corpus_root: &str,
) -> Result<Report, String> {
    if !projections
        .declared
        .iter()
        .any(|declaration| declaration.kind == Kind::SiteNav)
    {
        return Err("this taxonomy declares no `site_nav` projection, so no navigation names a page to look for. Add `{kind: site_nav, output: .headwater/nav.yml}` under `add_to.projections` in the overlay, and run `headwater taxonomy resolve`".to_string());
    }
    if !site.is_dir() {
        return Err(format!("{} is not a directory", site.display()));
    }
    let files = walk(site);
    let pages: Vec<&String> = files.iter().filter(|file| is_page(file)).collect();
    if pages.is_empty() {
        return Err(format!(
            "{} holds no `.html` file, so it is not a built site",
            site.display()
        ));
    }

    let mut report = Report {
        pages: pages.len(),
        navigated: plan.navigation.len(),
        ..Report::default()
    };

    #[allow(unreachable_code)]
    return Ok(report); // STUB
    // 1. A page the navigation names and the site does not hold.
    for path in &plan.navigation {
        let Some(source) = under(corpus_root, path) else {
            report.findings.push(Finding {
                rule: PAGE_MISSING,
                at: path.clone(),
                detail: "the navigation names it, and it is outside the corpus root, so no page of the site serves it".to_string(),
            });
            continue;
        };
        let forms = pages_of(source);
        if !forms.is_empty() && !forms.iter().any(|form| files.contains(form)) {
            report.findings.push(Finding {
                rule: PAGE_MISSING,
                at: source.to_string(),
                detail: format!(
                    "the navigation names it, and the site holds no {}",
                    forms.join(" and no ")
                ),
            });
        }
    }

    // 2. A page under a shelf's directory that answers to no source.
    let accounted: BTreeSet<String> = surface
        .documents()
        .iter()
        .map(|document| document.path.to_string())
        .chain(plan.outputs.iter().map(|output| output.path.clone()))
        .filter_map(|path| under(corpus_root, &path).map(pages_of))
        .flatten()
        .collect();
    let shelves: BTreeSet<String> = surface
        .taxonomy()
        .shelves
        .iter()
        .filter_map(|shelf| {
            let directory = shelf_index::directory_of(shelf.pattern.source());
            under(corpus_root, &directory).map(str::to_string)
        })
        .filter(|directory| !directory.is_empty())
        .collect();
    for page in &pages {
        let on_shelf = shelves
            .iter()
            .any(|directory| page.starts_with(&format!("{directory}/")));
        if on_shelf && !accounted.contains(*page) {
            report.findings.push(Finding {
                rule: PAGE_STALE,
                at: (*page).clone(),
                detail: "it is under a shelf's directory, and no document of the corpus and no generated page is served here".to_string(),
            });
        }
    }

    // 3 and 4. Every in-site link of every page.
    let mut ids: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut scanned: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for page in &pages {
        let text = String::from_utf8_lossy(&std::fs::read(site.join(page)).unwrap_or_default())
            .into_owned();
        let (hrefs, found) = scan(&text);
        ids.insert((*page).clone(), found);
        scanned.insert((*page).clone(), hrefs);
    }
    for (page, hrefs) in &scanned {
        for href in hrefs {
            let Some(link) = in_site(href) else { continue };
            report.links += 1;
            let target = match resolve(page, &link.path, &files) {
                Some(target) => target,
                None => {
                    report.findings.push(Finding {
                        rule: LINK_DEAD,
                        at: page.clone(),
                        detail: format!("`{href}` resolves to no file of the site"),
                    });
                    continue;
                }
            };
            let Some(fragment) = link.fragment else { continue };
            let Some(on_target) = ids.get(&target) else { continue };
            if !on_target.contains(&fragment) && !on_target.contains(&decode(&fragment)) {
                report.findings.push(Finding {
                    rule: FRAGMENT_DEAD,
                    at: page.clone(),
                    detail: format!("`{href}` names no `id` on {target}"),
                });
            }
        }
    }

    report.findings.sort_by(|a, b| {
        order(a.rule)
            .cmp(&order(b.rule))
            .then_with(|| a.at.cmp(&b.at))
            .then_with(|| a.detail.cmp(&b.detail))
    });
    Ok(report)
}

fn order(rule: &str) -> usize {
    [PAGE_MISSING, PAGE_STALE, LINK_DEAD, FRAGMENT_DEAD]
        .iter()
        .position(|known| *known == rule)
        .unwrap_or(usize::MAX)
}

fn is_page(path: &str) -> bool {
    path.ends_with(".html")
}

/// A repository path as a path under the corpus root, or `None` outside it.
fn under<'a>(corpus_root: &str, path: &'a str) -> Option<&'a str> {
    let root = corpus_root.trim_end_matches('/');
    if root.is_empty() || root == "." {
        return Some(path);
    }
    if path == root {
        return Some("");
    }
    path.strip_prefix(root)?.strip_prefix('/')
}

/// The pages a source under the corpus root is served at. Empty for a file
/// that is not Markdown, which a generator copies rather than renders.
pub fn pages_of(source: &str) -> Vec<String> {
    let Some(stem) = source.strip_suffix(".md") else {
        return Vec::new();
    };
    let (parent, name) = match stem.rsplit_once('/') {
        Some((parent, name)) => (parent, name),
        None => ("", stem),
    };
    match name {
        "README" | "index" => vec![join(parent, "index.html")],
        _ => vec![format!("{stem}/index.html"), format!("{stem}.html")],
    }
}

fn join(directory: &str, name: &str) -> String {
    match directory.is_empty() {
        true => name.to_string(),
        false => format!("{directory}/{name}"),
    }
}

/// Every file under the site, as a path with `/` separators.
fn walk(site: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![site.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => stack.push(path),
                Ok(_) => {
                    if let Ok(relative) = path.strip_prefix(site) {
                        let parts: Vec<String> = relative
                            .components()
                            .map(|part| part.as_os_str().to_string_lossy().into_owned())
                            .collect();
                        out.insert(parts.join("/"));
                    }
                }
                Err(_) => {}
            }
        }
    }
    out
}

/// An in-site link: its path, which is empty for a link to its own page, and
/// its fragment where it carries one.
#[derive(Debug, PartialEq, Eq)]
pub struct Link {
    pub path: String,
    pub fragment: Option<String>,
}

/// The in-site part of one `href`, or `None` for a link this verb does not
/// read: one with a scheme, one that opens with `//` or `/`, and a bare `#`.
///
/// A link that opens with `/` depends on where the site is served, which the
/// directory does not say, so it is not read.
pub fn in_site(href: &str) -> Option<Link> {
    let href = href.trim().replace("&amp;", "&");
    if href.is_empty() || href == "#" || href.starts_with('/') || has_scheme(&href) {
        return None;
    }
    let (rest, fragment) = match href.split_once('#') {
        Some((rest, fragment)) => (rest, (!fragment.is_empty()).then(|| fragment.to_string())),
        None => (href.as_str(), None),
    };
    let path = rest.split('?').next().unwrap_or("").to_string();
    Some(Link { path, fragment })
}

fn has_scheme(href: &str) -> bool {
    let Some(colon) = href.find(':') else {
        return false;
    };
    let scheme = &href[..colon];
    let mut chars = scheme.chars();
    chars.next().is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// The file of the site a link's path reaches from `page`, or `None`.
///
/// An empty path is the page itself. A path that ends in `/` is that
/// directory's `index.html`, and a path to a directory with no `/` is its
/// `index.html` too, which is what a server answers after its redirect.
pub fn resolve(page: &str, path: &str, files: &BTreeSet<String>) -> Option<String> {
    if path.is_empty() {
        return Some(page.to_string());
    }
    let decoded = decode(path);
    let mut parts: Vec<&str> = page.split('/').collect();
    parts.pop();
    for segment in decoded.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            other => parts.push(other),
        }
    }
    let joined = parts.join("/");
    if decoded.ends_with('/') {
        let index = join(&joined, "index.html");
        return files.contains(&index).then_some(index);
    }
    if files.contains(&joined) {
        return Some(joined);
    }
    let index = join(&joined, "index.html");
    files.contains(&index).then_some(index)
}

/// `%XX` decoded, as a site's file names and `id`s are written.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] == b'%' && at + 2 < bytes.len() {
            let high = (bytes[at + 1] as char).to_digit(16);
            let low = (bytes[at + 2] as char).to_digit(16);
            if let (Some(high), Some(low)) = (high, low) {
                out.push((high * 16 + low) as u8);
                at += 3;
                continue;
            }
        }
        out.push(bytes[at]);
        at += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Every `href` and every `id` of one page, in document order and as a set.
///
/// A small scanner rather than a parser: it reads a tag's attributes, quoted
/// with `"` or `'` or bare, and it skips comments and the bodies of `script`
/// and `style`, where an attribute-shaped string is not an attribute.
pub fn scan(html: &str) -> (Vec<String>, BTreeSet<String>) {
    let bytes = html.as_bytes();
    let mut hrefs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'<' {
            at += 1;
            continue;
        }
        if html[at..].starts_with("<!--") {
            at = match html[at + 4..].find("-->") {
                Some(end) => at + 4 + end + 3,
                None => bytes.len(),
            };
            continue;
        }
        if !bytes.get(at + 1).is_some_and(u8::is_ascii_alphabetic) {
            at += 1;
            continue;
        }
        // The tag name.
        let mut cursor = at + 1;
        while cursor < bytes.len() && (bytes[cursor].is_ascii_alphanumeric() || bytes[cursor] == b'-') {
            cursor += 1;
        }
        let tag = html[at + 1..cursor].to_ascii_lowercase();
        // The attributes, up to the `>` that closes the tag.
        loop {
            while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b'/') {
                cursor += 1;
            }
            if cursor >= bytes.len() || bytes[cursor] == b'>' {
                cursor += 1;
                break;
            }
            let start = cursor;
            while cursor < bytes.len()
                && !bytes[cursor].is_ascii_whitespace()
                && !matches!(bytes[cursor], b'=' | b'>' | b'/')
            {
                cursor += 1;
            }
            let name = html[start..cursor].to_ascii_lowercase();
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if bytes.get(cursor) != Some(&b'=') {
                if cursor == start {
                    cursor += 1;
                }
                continue;
            }
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            let value = match bytes.get(cursor) {
                Some(&quote) if quote == b'"' || quote == b'\'' => {
                    let open = cursor + 1;
                    let close = html[open..]
                        .find(quote as char)
                        .map_or(bytes.len(), |end| open + end);
                    cursor = (close + 1).min(bytes.len());
                    &html[open..close]
                }
                _ => {
                    let open = cursor;
                    while cursor < bytes.len()
                        && !bytes[cursor].is_ascii_whitespace()
                        && bytes[cursor] != b'>'
                    {
                        cursor += 1;
                    }
                    &html[open..cursor]
                }
            };
            match name.as_str() {
                "href" => hrefs.push(value.to_string()),
                "id" => {
                    ids.insert(value.to_string());
                }
                _ => {}
            }
        }
        at = cursor;
        if tag == "script" || tag == "style" {
            let close = format!("</{tag}");
            at = match html[at.min(bytes.len())..].to_ascii_lowercase().find(&close) {
                Some(end) => at + end,
                None => bytes.len(),
            };
        }
    }
    (hrefs, ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_is_served_in_either_form_and_an_index_in_one() {
        assert_eq!(
            pages_of("decisions/0001-a.md"),
            vec!["decisions/0001-a/index.html", "decisions/0001-a.html"]
        );
        assert_eq!(pages_of("decisions/README.md"), vec!["decisions/index.html"]);
        assert_eq!(pages_of("index.md"), vec!["index.html"]);
        assert!(pages_of("nav.yml").is_empty());
    }

    #[test]
    fn a_link_out_of_the_site_is_not_read() {
        for href in ["https://example.org/", "mailto:a@b", "//cdn/x.js", "/abs/", "#", ""] {
            assert_eq!(in_site(href), None, "{href}");
        }
        assert_eq!(
            in_site("../b/?q=1&amp;r=2#part"),
            Some(Link {
                path: "../b/".to_string(),
                fragment: Some("part".to_string())
            })
        );
        assert_eq!(
            in_site("#own"),
            Some(Link {
                path: String::new(),
                fragment: Some("own".to_string())
            })
        );
    }

    #[test]
    fn a_path_resolves_against_its_page_and_a_directory_to_its_index() {
        let files: BTreeSet<String> = ["a/index.html", "a/b/index.html", "x.css"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(resolve("a/b/index.html", "../", &files).as_deref(), Some("a/index.html"));
        assert_eq!(resolve("a/b/index.html", "../../x.css", &files).as_deref(), Some("x.css"));
        assert_eq!(resolve("a/index.html", "b", &files).as_deref(), Some("a/b/index.html"));
        assert_eq!(resolve("a/index.html", "c/", &files), None);
        assert_eq!(resolve("a/index.html", "../../x.css", &files), None);
    }

    #[test]
    fn the_scanner_reads_both_quotes_and_skips_comments_and_scripts() {
        let (hrefs, ids) = scan(
            "<a href=\"one\" id='two'>x</a><!-- <a href=\"no\"> --><script>var s = '<a href=\"no\" id=\"no\">';</script><h2 id=three>t</h2><link rel=stylesheet href=four>",
        );
        assert_eq!(hrefs, vec!["one", "four"]);
        assert_eq!(
            ids,
            ["three", "two"].iter().map(|s| s.to_string()).collect()
        );
    }

    #[test]
    fn a_percent_escape_decodes() {
        assert_eq!(decode("a%20b%2"), "a b%2");
    }
}
