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

impl U64BloomFilter {
    pub fn new(estimated_items: u64, fp: f64) -> Self {
        let num_bits = num_bits(estimated_items, fp);
        Self {
            bit_vec: BitVec::repeat(false, num_bits as usize),
        }
    } // pub fn new(estimated_items: u64, fp: f64) -> Self

    pub fn empty_from(other: &Self) -> Self {
        Self {
            bit_vec: BitVec::repeat(false, other.bit_vec.len()),
        }
    } // pub fn empty_from(other: &Self) -> Self

    pub fn fill(&mut self) {
        for i in 0..self.bit_vec.len() {
            self.bit_vec.set(i, true);
        }
    } // pub fn fill(&mut self)

    fn hash(item: u64) -> usize {
        item.wrapping_mul(LARGE_PRIME) as usize
    } // fn hash(item: u64) -> usize

    pub fn insert(&mut self, item: u64) {
        let h = Self::hash(item);
        let num_bits = self.bit_vec.len();
        self.bit_vec.set(h % num_bits, true);
    } // pub fn insert(&mut self, item: u64)

    pub fn insert_u128(&mut self, item: u128) {
        self.insert(item as u64)
    } // pub fn insert_u128(&mut self, item: u128)

    pub fn contains(&self, item: u64) -> bool {
        let h = Self::hash(item);
        self.bit_vec[h % self.bit_vec.len()]
    } // pub fn contains(&self, item: u64) -> bool

    pub fn contains_u128(&self, item: u128) -> bool {
        self.contains(item as u64)
    } // pub fn contains_u128(&self, item: u128) -> bool

    pub fn estimate_card(&self) -> u64 {
        let num_ones = self.bit_vec.count_ones() as u64;

        if num_ones == 0 || self.bit_vec.is_empty() {
            return 0;
        }

        if num_ones == self.bit_vec.len() as u64 {
            return u64::MAX;
        }

        (-(self.bit_vec.len() as i64)
            * (1.0 - (num_ones as f64) / (self.bit_vec.len() as f64)).ln() as i64)
            .try_into()
            .unwrap_or_default()
    } // pub fn estimate_card(&self) -> u64

    pub fn union(&mut self, other: Self) {
        debug_assert_eq!(self.bit_vec.len(), other.bit_vec.len());

        self.bit_vec |= other.bit_vec;
    } // pub fn union(&mut self, other: Self)
} // impl U64BloomFilter

#[derive(bincode::Encode, bincode::Decode)]
pub struct BytesBloomFilter<T> {
    #[bincode(with_serde)]
    bit_vec: BitVec,
    num_hashes: u64,
    _marker: std::marker::PhantomData<T>,
} // pub struct BytesBloomFilter<T>

impl<T> BytesBloomFilter<T> {
    pub fn new(estimated_items: u64, fp: f64) -> Self {
        let num_bits = num_bits(estimated_items, fp);
        let num_hashes = num_hashes(num_bits, estimated_items);
        Self {
            bit_vec: BitVec::repeat(false, num_bits as usize),
            num_hashes,
            _marker: std::marker::PhantomData,
        }
    } // pub fn new(estimated_items: u64, fp: f64) -> Self

    fn hash_raw(item: &[u8]) -> [u64; 2] {
        split_u128(fast_stable_hash_128(item))
    } // fn hash_raw(item: &[u8]) -> [u64; 2]

    pub fn contains_raw(&self, item: &[u8]) -> bool {
        let [a, b] = Self::hash_raw(item);

        for i in 0..self.num_hashes {
            let h = ((a.wrapping_mul(i).wrapping_add(b)) % LARGE_PRIME) % self.bit_vec.len() as u64;
            if !self.bit_vec[h as usize] {
                return false;
            }
        }

        true
    } // pub fn contains_raw(&self, item: &[u8]) -> bool

    pub fn insert_raw(&mut self, item: &[u8]) {
        let [a, b] = Self::hash_raw(item);

        for i in 0..self.num_hashes {
            let h = ((a.wrapping_mul(i).wrapping_add(b)) % LARGE_PRIME) % self.bit_vec.len() as u64;
            self.bit_vec.set(h as usize, true);
        }
    } // pub fn insert_raw(&mut self, item: &[u8])
} // impl<T> BytesBloomFilter<T>

impl<T> BytesBloomFilter<T>
where
    T: AsRef<[u8]>,
{
    pub fn insert(&mut self, item: &T) {
        self.insert_raw(item.as_ref())
    } // pub fn insert(&mut self, item:  &T)

    pub fn contains(&self, item: &T) -> bool {
        self.contains_raw(item.as_ref())
    } // pub fn contains(&self, item: &T) -> bool
} // impl<T> BytesBloomFilter<T> where T: AsRef<[u8]>
