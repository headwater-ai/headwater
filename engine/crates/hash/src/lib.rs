// SPDX-License-Identifier: Apache-2.0
//! SHA-256, as FIPS 180-4 defines it.
//!
//! # Why this is here rather than in a dependency
//!
//! The engine has six third-party crates, and every one of them is there because
//! writing it would have been a project. This is not one: SHA-256 is a fixed
//! function of about eighty lines with published test vectors, and the vetted
//! Rust implementation of it arrives with six transitive crates behind it.
//!
//! The other half of the argument is what the digest is for. Most digests this
//! engine writes are an integrity mark inside a repository that already holds
//! the bytes they were computed from. None of them is a signature.
//!
//! One is not like the others. The package digest that
//! [spec 7](../../../../docs/spec/distribution-and-federation.md#publishing)
//! puts on a release is a check on an artifact fetched from elsewhere, and
//! `headwater_resolve::release` computes it with this function.
//! [Q22](../../../../docs/spec/decisions.md#q22--the-integrity-posture-of-a-published-package)
//! decided that rather than inheriting it, and the argument is what the two
//! options buy. A vetted implementation buys a correct SHA-256 and no
//! authentication at all: the check is strong because a person committed the
//! pin, not because of who wrote the compression function. So the risk a
//! dependency answers is an implementation defect.
//!
//! That risk is answered by evidence here. The test vectors below are the
//! published ones, so a defect fails a test rather than producing a lock that
//! nobody can reproduce. `tests/oracle.rs` holds sixteen further subjects
//! against `sha256sum`, which is an implementation nobody here wrote, and
//! continuous integration makes its absence a failure rather than a skip.
//!
//! What no implementation of this function buys is a statement about who
//! published an artifact. That gap is
//! [HW-OBL-0115](../../../../docs/obligations/0115-a-pinned-digest-authenticates-the-pin-and-never-the-publisher.md).
//!
//! # Why it is a crate of its own
//!
//! Three components hash, and they must agree. The lock hashes the canonical
//! text of a resolved taxonomy. The census hashes the bytes of each document it
//! reads. The check cache hashes a key built over both
//! ([spec 12](../../../../docs/spec/check-layer.md#determinism-concretely)).
//! A cache key that disagreed with a lock digest about what the digest of one
//! byte string is would be the correctness bug that spec 12 names, so there is
//! one implementation of the function and no crate carries a second.

const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// The digest of a byte string, named by the function that produced it.
///
/// This is the form every artifact of this engine writes: a lock's own digest
/// and the digest of each of its sources, the digest a census row carries, and
/// the digest that keys a cache entry. The prefix is what lets a later engine
/// change the function and say which one a recorded digest came from.
pub fn digest(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(bytes))
}

/// The digest of a byte string, as sixty-four lower-case hexadecimal digits.
///
/// This is [`Hasher`] fed the whole string in one call, so a digest computed
/// in pieces and a digest computed at once go through one compression
/// function and cannot disagree.
pub fn hex(bytes: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(bytes);
    hasher.finish_hex()
}

/// The initial hash value of FIPS 180-4, section 5.3.3.
const INITIAL: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// SHA-256 over a byte string that arrives in pieces.
///
/// The digest of the pieces is the digest of their concatenation, wherever
/// the splits fall. It exists for a caller that hashes many strings with a
/// common beginning: the check cache keys every instance over a text whose
/// first lines name only the rule and its scope, so it hashes those lines
/// once, keeps the state, and clones it for each instance (#1968). A clone
/// is a copy of 112 bytes, so the common lines cost nothing after the first.
///
/// It also writes through [`std::fmt::Write`], so a caller can `write!` a
/// line into it without building the line as a `String` first.
#[derive(Clone, Debug)]
pub struct Hasher {
    state: [u32; 8],
    /// The bytes of a block that is not yet full. The first `filled` are
    /// meaningful.
    block: [u8; 64],
    filled: usize,
    /// Every byte fed so far, across every call, which the padding writes as
    /// the message length.
    length: u64,
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher {
    pub const fn new() -> Self {
        Hasher {
            state: INITIAL,
            block: [0; 64],
            filled: 0,
            length: 0,
        }
    }

    /// Feed the next bytes of the message.
    pub fn update(&mut self, bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        let mut bytes = bytes;
        // Fill the partial block first. It is compressed only when it is full,
        // and a block that one call leaves full is compressed in that call, so
        // `filled` is always below 64 between calls.
        if self.filled > 0 {
            let take = (64 - self.filled).min(bytes.len());
            self.block[self.filled..self.filled + take].copy_from_slice(&bytes[..take]);
            self.filled += take;
            bytes = &bytes[take..];
            if self.filled < 64 {
                return;
            }
            compress(&mut self.state, &self.block);
            self.filled = 0;
        }
        // `as_chunks` rather than `chunks_exact`: the chunk size is a constant
        // here, so the compiler carries the length in the type and no bounds
        // check reaches the inner loop. It is stable from 1.88 and the
        // workspace floor is 1.91, so clippy's `chunks_exact_to_as_chunks`
        // fires on the older form and `-D warnings` in continuous integration
        // turns that into a build failure.
        let (blocks, rest) = bytes.as_chunks::<64>();
        for block in blocks {
            compress(&mut self.state, block);
        }
        self.block[..rest.len()].copy_from_slice(rest);
        self.filled = rest.len();
    }

    /// The digest of everything fed, as sixty-four lower-case hexadecimal
    /// digits.
    pub fn finish_hex(mut self) -> String {
        // The padding: a one bit, zeroes up to 56 bytes into a block, and the
        // bit length of the message as a 64-bit big-endian integer.
        let bits = self.length.wrapping_mul(8);
        let zeroes = if self.filled < 56 {
            55 - self.filled
        } else {
            119 - self.filled
        };
        let mut padding = [0u8; 64];
        padding[0] = 0x80;
        self.update(&padding[..1 + zeroes]);
        self.update(&bits.to_be_bytes());
        debug_assert_eq!(self.filled, 0, "the padding ends on a block boundary");

        use std::fmt::Write;
        self.state
            .iter()
            .fold(String::with_capacity(64), |mut out, word| {
                let _ = write!(out, "{word:08x}");
                out
            })
    }
}

impl std::fmt::Write for Hasher {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.update(text.as_bytes());
        Ok(())
    }
}

/// One block of the compression function, FIPS 180-4 section 6.2.2. This is
/// the only copy of it.
fn compress(state: &mut [u32; 8], block: &[u8; 64]) {
    let mut w = [0u32; 64];
    let (words, _) = block.as_chunks::<4>();
    for (index, word) in words.iter().enumerate() {
        w[index] = u32::from_be_bytes(*word);
    }
    for index in 16..64 {
        let s0 =
            w[index - 15].rotate_right(7) ^ w[index - 15].rotate_right(18) ^ (w[index - 15] >> 3);
        let s1 =
            w[index - 2].rotate_right(17) ^ w[index - 2].rotate_right(19) ^ (w[index - 2] >> 10);
        w[index] = w[index - 16]
            .wrapping_add(s0)
            .wrapping_add(w[index - 7])
            .wrapping_add(s1);
    }

    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;
    for index in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let choose = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(choose)
            .wrapping_add(K[index])
            .wrapping_add(w[index]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let majority = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(majority);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    for (slot, value) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
        *slot = slot.wrapping_add(value);
    }
}

#[cfg(test)]
mod tests {
    use super::{digest, hex, Hasher};

    /// The named form is the hexadecimal one with the function in front of it.
    #[test]
    fn a_digest_names_the_function_that_produced_it() {
        assert_eq!(digest(b"abc"), format!("sha256:{}", hex(b"abc")));
    }

    /// The published vectors. A defect here would produce a lock that nobody
    /// else can reproduce, and nothing else in this crate would notice.
    #[test]
    fn the_published_vectors() {
        assert_eq!(
            hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            hex(&b"a".repeat(1_000_000)),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    /// The block boundary, which is where a padding defect hides.
    #[test]
    fn every_length_around_a_block_boundary() {
        // Each of these is the digest of `a` repeated, at the lengths where the
        // padding either fits in the last block or forces another one.
        let cases = [
            (
                55,
                "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
            ),
            (
                56,
                "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
            ),
            (
                63,
                "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
            ),
            (
                64,
                "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
            ),
            (
                65,
                "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
            ),
        ];
        for (length, expected) in cases {
            assert_eq!(hex(&b"a".repeat(length)), expected, "at length {length}");
        }
    }

    fn pieces(parts: &[&[u8]]) -> String {
        let mut hasher = Hasher::new();
        for part in parts {
            hasher.update(part);
        }
        hasher.finish_hex()
    }

    /// A digest fed in pieces is the digest of the whole, wherever the pieces
    /// split: at 0 and at the full length, at each byte in between, and in
    /// three pieces whose middle one crosses a block boundary. The subjects
    /// are the published vectors and every length the case above brackets,
    /// with bytes that differ from one position to the next so that a piece
    /// fed out of place changes the digest.
    #[test]
    fn an_incremental_digest_agrees_with_one_call_at_every_split() {
        let mut subjects: Vec<Vec<u8>> = vec![
            b"".to_vec(),
            b"abc".to_vec(),
            b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".to_vec(),
        ];
        for length in 0..=130usize {
            subjects.push((0..length).map(|index| (index * 31 + 7) as u8).collect());
        }
        for bytes in &subjects {
            let whole = hex(bytes);
            let length = bytes.len();
            for split in 0..=length {
                let (left, right) = bytes.split_at(split);
                assert_eq!(
                    pieces(&[left, right]),
                    whole,
                    "length {length} split at {split}"
                );
            }
            for first in 0..=length.min(64) {
                for second in [63usize, 64, 65, 127, 128, 129] {
                    if second < first || second > length {
                        continue;
                    }
                    assert_eq!(
                        pieces(&[&bytes[..first], &bytes[first..second], &bytes[second..]]),
                        whole,
                        "length {length} split at {first} and {second}"
                    );
                }
            }
        }
        // The million-byte vector, in pieces of 1000, which is no multiple of
        // a block.
        let million = b"a".repeat(1_000_000);
        let mut hasher = Hasher::new();
        for chunk in million.chunks(1000) {
            hasher.update(chunk);
        }
        assert_eq!(
            hasher.finish_hex(),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    /// A clone carries the state of its original, and finishing the clone
    /// leaves the original able to go on: the property the check cache's
    /// memo of a key's first lines rests on.
    #[test]
    fn a_cloned_hasher_goes_on_from_where_its_original_stood() {
        let mut prefix = Hasher::new();
        prefix.update(&[b'p'; 70]);
        for suffix in [&b""[..], b"x", &[b'y'; 64], &[b'z'; 200]] {
            let mut clone = prefix.clone();
            clone.update(suffix);
            let mut whole = vec![b'p'; 70];
            whole.extend_from_slice(suffix);
            assert_eq!(clone.finish_hex(), hex(&whole));
        }
        assert_eq!(prefix.finish_hex(), hex(&[b'p'; 70]));
    }

    /// `write!` into a hasher is `update` with the formatted bytes.
    #[test]
    fn a_hasher_written_through_fmt_hashes_the_formatted_text() {
        use std::fmt::Write;
        let mut hasher = Hasher::new();
        let version = 2;
        writeln!(hasher, "rule r version {version}").expect("never fails");
        assert_eq!(hasher.finish_hex(), hex(b"rule r version 2\n"));
    }
}
