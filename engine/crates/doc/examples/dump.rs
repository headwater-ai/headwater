fn main() {
    let path = std::env::args().nth(1).expect("path");
    let source = std::fs::read_to_string(&path).expect("read");
    let doc = headwater_doc::parse(&source).expect("parse");
    for s in doc.body.sentences() {
        if s.words > 25 || std::env::var("ALL").is_ok() {
            println!("{}:{} [{}] {}", s.span.start.line, s.span.start.col, s.words, &s.text.chars().take(140).collect::<String>());
        }
    }
}
