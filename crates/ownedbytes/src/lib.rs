/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief ownedbytes
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
pub use stable_deref_trait::StableDeref;
use std::{
    fmt, io,
    ops::{Deref, Range},
    path::Path,
    sync::Arc,
};

pub struct OwnedBytes {
    data: &'static [u8],
    box_stable_deref: Arc<dyn Deref<Target = [u8]> + Sync + Send>,
} // pub struct OwnedBytes

impl OwnedBytes {
    pub fn mmap_from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let mmap = unsafe { memmap2::Mmap::map(&std::fs::File::open(path)?)? };

        let box_stable_deref = Arc::new(mmap);
        let bytes: &[u8] = box_stable_deref.deref();
        let data = unsafe { &*(bytes as *const [u8]) };
        Ok(Self {
            data,
            box_stable_deref,
        })
    }

    pub fn empty() -> Self {
        Self::new(&[][..])
    }

    pub fn new<T: StableDeref + Deref<Target = [u8]> + 'static + Send + Sync>(
        data_holder: T,
    ) -> Self {
        let box_stable_deref = Arc::new(data_holder);
        let bytes: &[u8] = box_stable_deref.deref();
        let data = unsafe { &*(bytes as *const [u8]) };
        Self {
            data,
            box_stable_deref,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.data
    }

    #[must_use]
    #[inline]
    pub fn slice(&self, range: Range<usize>) -> Self {
        Self {
            data: &self.data[range],
            box_stable_deref: self.box_stable_deref.clone(),
        }
    }

    #[inline]
    #[must_use]
    pub fn split(self, split_len: usize) -> (Self, Self) {
        let (left_data, right_data) = self.data.split_at(split_len);
        let right_box_stable_deref = self.box_stable_deref.clone();
        let left = Self {
            data: left_data,
            box_stable_deref: self.box_stable_deref,
        };
        let right = Self {
            data: right_data,
            box_stable_deref: right_box_stable_deref,
        };
        (left, right)
    }

    #[inline]
    #[must_use]
    pub fn rsplit(self, split_len: usize) -> (Self, Self) {
        let data_len = self.data.len();
        self.split(data_len - split_len)
    }

    pub fn split_off(&mut self, split_len: usize) -> Self {
        let (left, right) = self.data.split_at(split_len);
        let right_box_stable_deref = self.box_stable_deref.clone();
        let right_piece = Self {
            data: right,
            box_stable_deref: right_box_stable_deref,
        };
        self.data = left;
        right_piece
    }

    #[inline]
    pub fn advance(&mut self, advance_len: usize) -> &[u8] {
        let (data, rest) = self.data.split_at(advance_len);
        self.data = rest;
        data
    }

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        self.advance(1)[0]
    }

    #[inline]
    fn read_n<const N: usize>(&mut self) -> [u8; N] {
        self.advance(N).try_into().unwrap()
    }

    #[inline]
    pub fn read_u32_le(&mut self) -> u32 {
        u32::from_le_bytes(self.read_n())
    }

    #[inline]
    pub fn read_u64_le(&mut self) -> u64 {
        u64::from_le_bytes(self.read_n())
    }
} // impl OwnedBytes
