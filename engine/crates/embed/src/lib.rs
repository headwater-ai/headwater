// SPDX-License-Identifier: Apache-2.0
//! The offline embedding path, and nothing an agent reads.
//!
//! [HW-DR-0064](../../../../docs/decisions/0064-q64-whether-intent-time-routing-gains-an-offline-embedding-path-in-shadow-mode.md)
//! rules on four things this crate holds, and each one has a type:
//!
//! - **The model is pinned and fetched, never committed.** [`Pin`] reads
//!   `.headwater/embedding.yml`, which names every file by URL and digest. The
//!   caller fetches the bytes into an ignored directory, as `taxonomy vendor`
//!   does, and [`Model::load`] refuses a file whose digest is not the pinned
//!   one. No crate here opens a socket.
//! - **Inference is pure Rust.** `tract-onnx` runs the graph, and
//!   [`WordPiece`] is the tokenizer.
//! - **The vectors are a cache.** [`Cache`] lives under `.headwater/cache/`,
//!   keyed by the model digest and the digest of each summary. Quantized
//!   arithmetic differs between instruction sets, so a vector is a fact about
//!   this machine and never about the tree, and no emitter writes one.
//! - **The model digest names the population.** [`Pin::digest`] covers every
//!   pinned file, so a second model, or a second vocabulary, is a second digest.

mod wordpiece;

pub use wordpiece::WordPiece;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tract_onnx::prelude::*;

/// Where the pin is committed, relative to the repository root.
pub const PIN: &str = ".headwater/embedding.yml";

/// Where a caller fetches the model files to by default. Ignored by git.
pub const MODELS: &str = ".headwater/models";

/// Where the vectors are kept. Inside the ignored check cache directory.
pub const CACHE: &str = ".headwater/cache/embeddings";

/// The two files every pin names, because inference reads both.
const GRAPH: &str = "model.onnx";
const VOCABULARY: &str = "vocab.txt";

/// One pinned file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pinned {
    pub name: String,
    pub url: String,
    pub digest: String,
}

/// The committed pin: which model, under which license, and the digest of
/// every byte inference reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pin {
    pub model: String,
    pub revision: String,
    pub license: String,
    pub max_tokens: usize,
    pub files: Vec<Pinned>,
}

impl Pin {
    pub fn read(root: &Path) -> Result<Pin, String> {
        let path = root.join(PIN);
        let source = std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read `{PIN}`: {error}"))?;
        Pin::parse(&source)
    }

    pub fn parse(source: &str) -> Result<Pin, String> {
        let document = headwater_yaml::load(source)
            .map_err(|_| format!("`{PIN}` is not a YAML document this engine can read"))?;
        let map = document
            .value
            .as_map()
            .ok_or_else(|| format!("`{PIN}` is not a mapping"))?;
        let text = |key: &str| -> Result<String, String> {
            map.get(key)
                .and_then(|value| value.value.as_scalar())
                .map(|scalar| scalar.text.clone())
                .filter(|text| !text.is_empty())
                .ok_or_else(|| format!("`{PIN}` states no `{key}`"))
        };
        let max_tokens = text("max_tokens")?
            .parse::<usize>()
            .ok()
            .filter(|limit| *limit > 1)
            .ok_or_else(|| format!("`max_tokens` in `{PIN}` is not a whole number above one"))?;

        let files = map
            .get("files")
            .and_then(|value| value.value.as_map())
            .ok_or_else(|| format!("`{PIN}` states no `files` mapping"))?;
        let mut pinned = Vec::new();
        for entry in files {
            let name = entry.key.value.clone();
            let fields = entry
                .value
                .value
                .as_map()
                .ok_or_else(|| format!("`files.{name}` in `{PIN}` is not a mapping"))?;
            let field = |key: &str| -> Result<String, String> {
                fields
                    .get(key)
                    .and_then(|value| value.value.as_scalar())
                    .map(|scalar| scalar.text.clone())
                    .filter(|text| !text.is_empty())
                    .ok_or_else(|| format!("`files.{name}` in `{PIN}` states no `{key}`"))
            };
            let digest = field("digest")?;
            if !is_digest(&digest) {
                return Err(format!(
                    "`files.{name}.digest` in `{PIN}` is not a `sha256:` digest"
                ));
            }
            pinned.push(Pinned {
                url: field("url")?,
                digest,
                name,
            });
        }
        for required in [GRAPH, VOCABULARY] {
            if !pinned.iter().any(|file| file.name == required) {
                return Err(format!("`{PIN}` pins no `{required}`"));
            }
        }

        Ok(Pin {
            model: text("model")?,
            revision: text("revision")?,
            license: text("license")?,
            max_tokens,
            files: pinned,
        })
    }

    /// The digest of the pinned set: each file name and its digest, in name
    /// order. It moves when any byte inference reads moves, and never when a
    /// URL does.
    pub fn digest(&self) -> String {
        let mut named: Vec<(&str, &str)> = self
            .files
            .iter()
            .map(|file| (file.name.as_str(), file.digest.as_str()))
            .collect();
        named.sort_unstable();
        let mut listing = String::new();
        for (name, digest) in named {
            listing.push_str(name);
            listing.push('\t');
            listing.push_str(digest);
            listing.push('\n');
        }
        headwater_hash::digest(listing.as_bytes())
    }

    fn file(&self, name: &str) -> Option<&Pinned> {
        self.files.iter().find(|file| file.name == name)
    }
}

fn is_digest(text: &str) -> bool {
    text.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

/// A loaded model whose every file matched its pin.
pub struct Model {
    plan: Arc<TypedSimplePlan>,
    tokenizer: WordPiece,
    max_tokens: usize,
    digest: String,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model")
            .field("digest", &self.digest)
            .finish_non_exhaustive()
    }
}

impl Model {
    /// Load the pinned files from `dir`, refusing any whose digest differs.
    pub fn load(pin: &Pin, dir: &Path) -> Result<Model, String> {
        let graph = verified(pin, dir, GRAPH)?;
        let vocabulary = verified(pin, dir, VOCABULARY)?;
        let vocabulary =
            String::from_utf8(vocabulary).map_err(|_| format!("`{VOCABULARY}` is not UTF-8"))?;
        let tokenizer = WordPiece::parse(&vocabulary)?;
        let plan = tract_onnx::onnx()
            .model_for_read(&mut graph.as_slice())
            .and_then(|model| model.into_optimized())
            .and_then(|model| model.into_runnable())
            .map_err(|error| format!("`{GRAPH}` did not load: {error}"))?;
        Ok(Model {
            plan,
            tokenizer,
            max_tokens: pin.max_tokens,
            digest: pin.digest(),
        })
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    /// The unit-length, mean-pooled vector of a text.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let ids = self.tokenizer.encode(text, self.max_tokens);
        let count = ids.len();
        let failed = |error: TractError| format!("inference failed: {error}");
        let input_ids = tract_ndarray::Array2::from_shape_vec((1, count), ids)
            .map_err(|error| format!("inference failed: {error}"))?
            .into_tensor();
        let mask = tract_ndarray::Array2::from_elem((1, count), 1_i64).into_tensor();
        let types = tract_ndarray::Array2::from_elem((1, count), 0_i64).into_tensor();
        let outputs = self
            .plan
            .run(tvec!(input_ids.into(), mask.into(), types.into()))
            .map_err(failed)?;
        let hidden = outputs
            .first()
            .ok_or_else(|| "inference returned no output".to_string())?
            .to_plain_array_view::<f32>()
            .map_err(failed)?;
        let shape = hidden.shape();
        if shape.len() != 3 || shape[0] != 1 || shape[1] != count {
            return Err(format!("inference returned an output of shape {shape:?}"));
        }
        let mut pooled = vec![0_f32; shape[2]];
        for token in hidden.index_axis(tract_ndarray::Axis(0), 0).outer_iter() {
            for (sum, value) in pooled.iter_mut().zip(token.iter()) {
                *sum += *value;
            }
        }
        // Mean pooling divides by the token count, and the unit normalization
        // removes that factor again, so the sum is normalized directly.
        let norm = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for value in &mut pooled {
                *value /= norm;
            }
        }
        Ok(pooled)
    }
}

fn verified(pin: &Pin, dir: &Path, name: &str) -> Result<Vec<u8>, String> {
    let pinned = pin
        .file(name)
        .ok_or_else(|| format!("`{PIN}` pins no `{name}`"))?;
    let path = dir.join(name);
    let bytes = std::fs::read(&path).map_err(|error| {
        format!(
            "cannot read `{}`: {error}. Fetch the pinned files with `tools/embed/fetch-model.sh`",
            path.display()
        )
    })?;
    // Hashing the graph costs most of a warm run, and the hook runs this on
    // every prompt. So a verified file leaves a stamp of its digest, length and
    // modification time, and a file whose stamp still holds is not hashed
    // again. Any change to the length or the time hashes it, as git's index does.
    let stamp = dir.join(format!(".{name}.verified"));
    let witness = std::fs::metadata(&path).ok().and_then(|meta| {
        let modified = meta
            .modified()
            .ok()?
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?;
        Some(format!(
            "{}\t{}\t{}.{:09}\n",
            pinned.digest,
            meta.len(),
            modified.as_secs(),
            modified.subsec_nanos()
        ))
    });
    if let Some(witness) = &witness {
        if std::fs::read_to_string(&stamp).is_ok_and(|held| held == *witness) {
            return Ok(bytes);
        }
    }
    let found = headwater_hash::digest(&bytes);
    if found != pinned.digest {
        let _ = std::fs::remove_file(&stamp);
        return Err(format!(
            "`{}` is not the pinned bytes: the pin says {}, the file is {found}",
            path.display(),
            pinned.digest
        ));
    }
    if let Some(witness) = witness {
        let _ = std::fs::write(&stamp, witness);
    }
    Ok(bytes)
}

/// The dot product. Both vectors are unit length, so it is the cosine.
pub fn similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// The vector cache for one model digest.
///
/// Every entry is recomputable from the text it is keyed on, so a missing,
/// unreadable or malformed file is an empty cache and never an error. A write
/// keeps only the entries this run read, which keeps the file the size of the
/// corpus in front of it.
#[derive(Debug)]
pub struct Cache {
    path: PathBuf,
    entries: BTreeMap<String, Vec<f32>>,
    used: BTreeSet<String>,
    computed: usize,
}

const HEADER: &str = "headwater-embed-cache 1";

impl Cache {
    pub fn open(root: &Path, model: &str) -> Cache {
        let name = model.strip_prefix("sha256:").unwrap_or(model);
        let path = root.join(CACHE).join(name);
        let entries = std::fs::read_to_string(&path)
            .ok()
            .and_then(|text| parse_cache(&text, model))
            .unwrap_or_default();
        Cache {
            path,
            entries,
            used: BTreeSet::new(),
            computed: 0,
        }
    }

    /// The vector of a text, from the cache where it holds one.
    pub fn vector(&mut self, model: &Model, text: &str) -> Result<Vec<f32>, String> {
        let key = headwater_hash::digest(text.as_bytes());
        self.used.insert(key.clone());
        if let Some(vector) = self.entries.get(&key) {
            return Ok(vector.clone());
        }
        let vector = model.embed(text)?;
        self.computed += 1;
        self.entries.insert(key, vector.clone());
        Ok(vector)
    }

    /// How many vectors this run computed rather than read.
    pub fn computed(&self) -> usize {
        self.computed
    }

    /// Write the entries this run read, through a rename so a concurrent
    /// reader never sees half a file. A failure leaves the cache as it was.
    pub fn write(&self, model: &str) {
        if self.computed == 0 && self.used.len() == self.entries.len() {
            return;
        }
        let mut text = format!("{HEADER} {model}\n");
        for key in &self.used {
            if let Some(vector) = self.entries.get(key) {
                text.push_str(key);
                text.push('\t');
                for value in vector {
                    for byte in value.to_le_bytes() {
                        text.push_str(&format!("{byte:02x}"));
                    }
                }
                text.push('\n');
            }
        }
        let Some(dir) = self.path.parent() else {
            return;
        };
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
        let staged = self
            .path
            .with_extension(format!("tmp-{}", std::process::id()));
        if std::fs::write(&staged, text).is_ok() && std::fs::rename(&staged, &self.path).is_err() {
            let _ = std::fs::remove_file(&staged);
        }
    }
}

fn parse_cache(text: &str, model: &str) -> Option<BTreeMap<String, Vec<f32>>> {
    let mut lines = text.lines();
    if lines.next()? != format!("{HEADER} {model}") {
        return None;
    }
    let mut entries = BTreeMap::new();
    for line in lines {
        let (key, hex) = line.split_once('\t')?;
        if !is_digest(key) || hex.len() % 8 != 0 {
            return None;
        }
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok())
            .collect::<Option<_>>()?;
        // `hex.len() % 8 == 0` above leaves no remainder here.
        let (quads, _) = bytes.as_chunks::<4>();
        let vector = quads.iter().copied().map(f32::from_le_bytes).collect();
        entries.insert(key.to_string(), vector);
    }
    Some(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PINNED: &str = "model: sentence-transformers/all-MiniLM-L6-v2\n\
        revision: 1110a243fdf4706b3f48f1d95db1a4f5529b4d41\n\
        license: Apache-2.0\n\
        max_tokens: 256\n\
        files:\n  \
          model.onnx:\n    \
            url: https://example.invalid/model.onnx\n    \
            digest: sha256:6fd5d72fe4589f189f8ebc006442dbb529bb7ce38f8082112682524616046452\n  \
          vocab.txt:\n    \
            url: https://example.invalid/vocab.txt\n    \
            digest: sha256:07eced375cec144d27c900241f3e339478dec958f92fddbc551f295c992038a3\n";

    /// A directory under the temporary directory that is removed when this value
    /// is dropped, so a case that fails an assertion leaves nothing behind (#1158).
    struct Scratch(std::path::PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    impl std::ops::Deref for Scratch {
        type Target = std::path::Path;
        fn deref(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl AsRef<std::path::Path> for Scratch {
        fn as_ref(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl AsRef<std::ffi::OsStr> for Scratch {
        fn as_ref(&self) -> &std::ffi::OsStr {
            self.0.as_os_str()
        }
    }

    #[test]
    fn a_pin_reads_every_field() {
        let pin = Pin::parse(PINNED).expect("the fixture pin parses");
        assert_eq!(pin.model, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(pin.max_tokens, 256);
        assert_eq!(pin.files.len(), 2);
    }

    #[test]
    fn the_model_digest_moves_with_a_file_digest_and_not_with_a_url() {
        let pin = Pin::parse(PINNED).expect("the fixture pin parses");
        let moved_url = Pin::parse(&PINNED.replace("example.invalid", "mirror.invalid"))
            .expect("the moved pin parses");
        let moved_bytes =
            Pin::parse(&PINNED.replace("6fd5d72f", "6fd5d72e")).expect("the moved pin parses");
        assert_eq!(pin.digest(), moved_url.digest());
        assert_ne!(pin.digest(), moved_bytes.digest());
    }

    #[test]
    fn a_pin_without_the_vocabulary_is_refused() {
        let cut = PINNED
            .split("  vocab.txt:")
            .next()
            .expect("the fixture names the vocabulary");
        assert!(Pin::parse(cut).is_err());
    }

    #[test]
    fn a_digest_that_is_not_sha256_is_refused() {
        assert!(Pin::parse(&PINNED.replace("sha256:6fd5", "md5:6fd5")).is_err());
    }

    #[test]
    fn a_file_that_is_not_the_pinned_bytes_is_refused_before_anything_loads() {
        let dir = Scratch(
            std::env::temp_dir().join(format!("headwater-embed-refuse-{}", std::process::id())),
        );
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        std::fs::write(dir.join(GRAPH), b"not a model").expect("a planted file");
        let pin = Pin::parse(PINNED).expect("the fixture pin parses");
        let refused = Model::load(&pin, &dir).expect_err("the planted bytes do not match");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(refused.contains("is not the pinned bytes"), "{refused}");
    }

    #[test]
    fn a_stamp_that_no_longer_matches_the_file_does_not_skip_the_digest() {
        let dir = Scratch(
            std::env::temp_dir().join(format!("headwater-embed-stamp-{}", std::process::id())),
        );
        std::fs::create_dir_all(&dir).expect("a scratch directory");
        std::fs::write(dir.join(GRAPH), b"not a model").expect("a planted file");
        let pin = Pin::parse(PINNED).expect("the fixture pin parses");
        let digest = &pin.file(GRAPH).expect("the pin names the graph").digest;
        std::fs::write(
            dir.join(format!(".{GRAPH}.verified")),
            format!("{digest}\t90405214\t1.000000000\n"),
        )
        .expect("a planted stamp");
        let refused = Model::load(&pin, &dir).expect_err("the stamp describes other bytes");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(refused.contains("is not the pinned bytes"), "{refused}");
    }

    /// The pinned model reproduces the similarity sentence-transformers
    /// publishes for its own example pair. The files are 90MB and fetched, so a
    /// run with no `HEADWATER_MODEL_DIR` says it skipped rather than passing.
    #[test]
    fn the_pinned_model_ranks_a_paraphrase_above_an_unrelated_sentence() {
        let Some(dir) = std::env::var_os("HEADWATER_MODEL_DIR") else {
            eprintln!("skipped: set HEADWATER_MODEL_DIR to the fetched model files");
            return;
        };
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let pin = Pin::read(&root).expect("the committed pin parses");
        let model = Model::load(&pin, Path::new(&dir)).expect("the fetched files match the pin");
        let eating = model
            .embed("A man is eating food.")
            .expect("inference runs");
        let bread = model
            .embed("A man is eating a piece of bread.")
            .expect("inference runs");
        let baby = model
            .embed("The girl is carrying a baby.")
            .expect("inference runs");
        let close = similarity(&eating, &bread);
        let far = similarity(&eating, &baby);
        assert!(
            (close - 0.7553).abs() < 0.01,
            "paraphrase similarity {close}"
        );
        assert!(far < 0.1, "unrelated similarity {far}");
    }

    #[test]
    fn a_cache_file_round_trips_and_a_foreign_model_reads_as_empty() {
        let vector = vec![0.25_f32, -1.0, 0.5, 3.0e-7];
        let key = headwater_hash::digest(b"a summary");
        let mut hex = String::new();
        for value in &vector {
            for byte in value.to_le_bytes() {
                hex.push_str(&format!("{byte:02x}"));
            }
        }
        let text = format!("{HEADER} sha256:aa\n{key}\t{hex}\n");
        let read = parse_cache(&text, "sha256:aa").expect("the cache parses");
        assert_eq!(read.get(&key), Some(&vector));
        assert!(parse_cache(&text, "sha256:bb").is_none());
    }
}
