// SPDX-License-Identifier: Apache-2.0
//! Content hashing for cache keys.
//!
//! FNV-1a, 128-bit. This is a **placeholder**: a real engine wants a
//! collision-resistant hash here, because spec 12 calls an incomplete cache key
//! a correctness bug rather than a performance bug. It is adequate for
//! measuring latency, which is all item 4 asks of it.

const OFFSET: u128 = 0x6c62272e07bb014262b821756295c58d;
const PRIME: u128 = 0x0000000001000000000000000000013b;

pub fn content(bytes: &[u8]) -> u128 {
    let mut h = OFFSET;
    for b in bytes {
        h ^= *b as u128;
        h = h.wrapping_mul(PRIME);
    }
    h
}

pub fn combine(parts: &[u128]) -> u128 {
    let mut h = OFFSET;
    for p in parts {
        h ^= *p;
        h = h.wrapping_mul(PRIME);
    }
    h
}
