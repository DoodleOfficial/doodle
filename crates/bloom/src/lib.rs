/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief bloom
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use bitvec::vec::BitVec;

pub fn combine_u64s(nums: [u64; 2]) -> u128 {
    ((nums[0] as u128) << 64) | (nums[1] as u128)
} // pub fn combine_u64s(nums: [u64; 2]) -> u128

pub fn split_u128(num: u128) -> [u64; 2] {
    [(num >> 64) as u64, num as u64]
} // pub fn split_u128(num: u128) -> [u64; 2]

const XXH3_SECRET: &[u8] = &xxhash_rust::const_xxh3::const_custom_default_secret(42);

pub fn fast_stable_hash_64(t: &[u8]) -> u64 {
    xxhash_rust::xxh3::xxh3_64_with_secret(t, XXH3_SECRET)
} // pub fn fast_stable_hash_64(t: &[u8]) -> u64

pub fn fast_stable_hash_128(t: &[u8]) -> u128 {
    xxhash_rust::xxh3::xxh3_128_with_secret(t, XXH3_SECRET)
} // pub fn fast_stable_hash_128(t: &[u8]) -> u128

const LARGE_PRIME: u64 = 11400714819323198549;

#[inline]
fn num_bits(estimated_items: u64, fp: f64) -> u64 {
    ((estimated_items as f64) * fp.ln() / (-8.0 * 2.0_f64.ln().powi(2))).ceil() as u64
} // fn num_bits(estimated_items: u64, fp: f64) -> u64

#[inline]
fn num_hashes(num_bits: u64, estimated_items: u64) -> u64 {
    (((num_bits as f64) / estimated_items as f64 * 2.0_f64.ln()).ceil() as u64).max(1)
} // fn num_hashes(num_bits: u64, estimated_items: u64) -> u64

#[derive(
    Clone,
    bincode::Encode,
    bincode::Decode,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    PartialEq,
    Eq,
)]
pub struct U64BloomFilter {
    #[bincode(with_serde)]
    bit_vec: BitVec,
} // pub struct U64BloomFilter
