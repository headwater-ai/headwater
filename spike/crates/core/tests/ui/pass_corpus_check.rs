//! Positive control, and the important one.
//!
//! Reading siblings is not forbidden. It is *declared*. A check that needs the
//! whole corpus implements `CorpusCheck`, receives a `CorpusView`, and registers
//! in the corpus bucket where the runner keys its cache on the whole corpus.
//! The declaration and the access are one fact, so they cannot disagree.

use headwater_core::check::{CheckMeta, CorpusCheck, Registry};
use headwater_core::finding::{Finding, Severity};
use headwater_core::view::CorpusView;

struct CountKinds;

impl CheckMeta for CountKinds {
    fn id(&self) -> &'static str {
        "count_kinds"
    }
    fn obligation(&self) -> &'static str {
        "OB-CORPUS-9"
    }
    fn severity(&self) -> Severity {
        Severity::Info
    }
}

impl CorpusCheck for CountKinds {
    fn evaluate(&self, view: &CorpusView<'_>) -> Vec<Finding> {
        // Sibling comparison, legitimately.
        let decisions = view
            .documents()
            .filter(|d| d.kind() == headwater_core::model::Kind::Decision)
            .count();
        let _ = decisions;
        Vec::new()
    }
}

fn main() {
    let r = Registry::new().with_corpus(CountKinds);
    assert_eq!(r.count(), 1);
}
