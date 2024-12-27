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

impl Deref for OwnedBytes {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.data
    }
} // impl Deref for OwnedBytes

impl AsRef<[u8]> for OwnedBytes {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.data
    }
} // impl AsRef<[u8]> for OwnedBytes

impl fmt::Debug for OwnedBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes_truncated: &[u8] = if self.len() > 8 {
            &self.as_slice()[..8]
        } else {
            self.as_slice()
        };

        write!(f, "OwnedBytes({bytes_truncated:?}, len={})", self.len())
    }
} // impl fmt::Debug for OwnedBytes

impl Clone for OwnedBytes {
    fn clone(&self) -> Self {
        OwnedBytes {
            data: self.data,
            box_stable_deref: self.box_stable_deref.clone(),
        }
    }
} // impl Clone for OwnedBytes

impl io::Read for OwnedBytes {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let data_len = self.data.len();
        let buf_len = buf.len();
        if data_len >= buf_len {
            let data = self.advance(buf_len);
            buf.copy_from_slice(data);
            Ok(buf_len)
        } else {
            buf[..data_len].copy_from_slice(self.data);
            self.data = &[];
            Ok(data_len)
        }
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        buf.extend(self.data);
        let read_len = self.data.len();
        self.data = &[];
        Ok(read_len)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let read_len = self.read(buf)?;
        if read_len != buf.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "failed to fill whole buffer",
            ));
        }
        Ok(())
    }
} // impl io::Read for OwnedBytes

impl From<Vec<u8>> for OwnedBytes {
    fn from(vec: Vec<u8>) -> Self {
        Self::new(vec)
    }
} // impl From<Vec<u8>> for OwnedBytes

impl PartialEq for OwnedBytes {
    fn eq(&self, other: &OwnedBytes) -> bool {
        self.as_slice() == other.as_slice()
    }
} // impl PartialEq for OwnedBytes

impl Eq for OwnedBytes {}

impl PartialEq<[u8]> for OwnedBytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_slice() == other
    }
} // impl PartialEq<[u8]> for OwnedBytes

impl PartialEq<str> for OwnedBytes {
    fn eq(&self, other: &str) -> bool {
        self.as_slice() == other.as_bytes()
    }
} // impl PartialEq<str> for OwnedBytes

impl<'a, T: ?Sized> PartialEq<&'a T> for OwnedBytes
where
    OwnedBytes: PartialEq<T>,
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
} // impl<'a, T: ?Sized> PartialEq<&'a T> for OwnedBytes where OwnedBytes: PartialEq<T>
