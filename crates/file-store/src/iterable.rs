
/**
 * @file iterable.rs
 * @author Krisna Pranav
 * @brief iterable
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

use crate::{const_serializable::{ConstSerializable}, peekable::Peekable, Result};
use ownedbytes::OwnedBytes;
use std::{
    cmp::Reverse,
    io::{self, Write},
    ops::Range,
    path::Path,
};

struct IterableHeader {
    num_upcoming_bytes: u64,
}

impl IterableHeader {
    #[inline]
    const fn serialized_size() -> usize {
        std::mem::size_of::<u64>()
    }

    fn serialize<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        writer.write_all(&self.num_upcoming_bytes.to_le_bytes())
    }

    fn deserialize(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != Self::serialized_size() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid number of bytes for IterableHeader",
            ));
        }

        Ok(IterableHeader {
            num_upcoming_bytes: u64::from_le_bytes(bytes[..8].try_into().unwrap()),
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WrittenOffset {
    pub start: u64,
    pub num_bytes: u64,
}

impl WrittenOffset {
    pub fn range(&self) -> Range<u64> {
        self.start..self.start + self.num_bytes
    }
}

pub struct IterableStoreWriter<T, W>
where
    W: io::Write,
{
    writer: io::BufWriter<W>,
    next_start: u64,
    _marker: std::marker::PhantomData<T>,
}
impl<T, W> IterableStoreWriter<T, W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer: io::BufWriter::new(writer),
            _marker: std::marker::PhantomData,
            next_start: 0,
        }
    }
}

impl<T, W> IterableStoreWriter<T, W>
where
    T: bincode::Encode,
    W: io::Write,
{
    pub fn write(&mut self, item: &T) -> Result<WrittenOffset> {
        let serialized = bincode::encode_to_vec(item, commons::bincode_config())?;
        let header = IterableHeader {
            num_upcoming_bytes: serialized.len() as u64,
        };
        header.serialize(&mut self.writer)?;
        self.writer.write_all(&serialized)?;

        let start = self.next_start;
        let bytes_written = IterableHeader::serialized_size() as u64 + serialized.len() as u64;

        self.next_start += bytes_written;

        Ok(WrittenOffset {
            start,
            num_bytes: bytes_written,
        })
    }

    pub fn finalize(mut self) -> Result<W> {
        self.writer.flush()?;

        self.writer.into_inner().map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub struct IterableStoreReader<T> {
    data: OwnedBytes,
    offset: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> IterableStoreReader<T> {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = OwnedBytes::mmap_from_path(path)?;

        Ok(Self {
            data,
            offset: 0,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data: OwnedBytes::new(data),
            offset: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn slice(&self, range: Range<usize>) -> IterableStoreReader<T> {
        IterableStoreReader {
            data: self.data.slice(range),
            offset: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Iterator for IterableStoreReader<T>
where
    T: bincode::Decode,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + IterableHeader::serialized_size() > self.data.len() {
            return None;
        }

        let header_bytes = &self.data[self.offset..self.offset + IterableHeader::serialized_size()];

        let header = match IterableHeader::deserialize(header_bytes) {
            Ok(header) => header,
            Err(_err) => return None,
        };

        self.offset += IterableHeader::serialized_size();
        let serialized = &self.data[self.offset..self.offset + header.num_upcoming_bytes as usize];

        self.offset += header.num_upcoming_bytes as usize;
        let (item, _) = bincode::decode_from_slice(serialized, commons::bincode_config()).unwrap();

        Some(item)
    }
}

impl<T> io::Seek for IterableStoreReader<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(offset) => {
                self.offset = offset as usize;
            }
            io::SeekFrom::End(offset) => {
                self.offset = self.data.len() - offset as usize;
            }
            io::SeekFrom::Current(offset) => {
                self.offset += offset as usize;
            }
        }

        Ok(self.offset as u64)
    }
}

type MinHeap<T> = std::collections::BinaryHeap<Reverse<T>>;

pub struct SortedIterableStoreReader<T>
where
    T: bincode::Decode,
{
    readers: MinHeap<Peekable<IterableStoreReader<T>>>,
}

impl<T> SortedIterableStoreReader<T>
where
    T: Ord + bincode::Decode,
{
    pub fn new(readers: Vec<IterableStoreReader<T>>) -> Self {
        let readers = readers
            .into_iter()
            .map(Peekable::new)
            .map(Reverse)
            .collect();

        Self { readers }
    }
}

impl<T> Iterator for SortedIterableStoreReader<T>
where
    T: Ord + bincode::Decode,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.readers.peek_mut().and_then(|mut item| item.0.next())
    }
}

pub struct ConstIterableStoreWriter<T, W>
where
    W: io::Write,
{
    writer: io::BufWriter<W>,
    buf: Vec<u8>,
    next_start: u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T, W> ConstIterableStoreWriter<T, W>
where
    T: ConstSerializable,
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer: io::BufWriter::new(writer),
            _marker: std::marker::PhantomData,
            buf: Vec::new(),
            next_start: 0,
        }
    }

    pub fn write(&mut self, item: &T) -> Result<WrittenOffset> {
        self.buf.resize(T::BYTES, 0);
        item.serialize(&mut self.buf);

        assert_eq!(self.buf.len(), T::BYTES);

        self.writer.write_all(&self.buf)?;

        let start = self.next_start;
        let bytes_written = T::BYTES as u64;

        self.next_start += bytes_written;

        Ok(WrittenOffset {
            start,
            num_bytes: bytes_written,
        })
    }

    pub fn finalize(mut self) -> Result<W> {
        self.writer.flush()?;

        self.writer.into_inner().map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub struct ConstIterableStoreReader<T> {
    data: OwnedBytes,
    offset: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ConstIterableStoreReader<T> {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let data = OwnedBytes::mmap_from_path(path)?;

        Ok(Self {
            data,
            offset: 0,
            _marker: std::marker::PhantomData,
        })
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data: OwnedBytes::new(data),
            offset: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn slice(&self, range: Range<usize>) -> ConstIterableStoreReader<T> {
        ConstIterableStoreReader {
            data: self.data.slice(range),
            offset: 0,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> ConstIterableStoreReader<T>
where
    T: ConstSerializable,
{
    pub fn len(&self) -> usize {
        self.data.len() / T::BYTES
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Iterator for ConstIterableStoreReader<T>
where
    T: ConstSerializable,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset + T::BYTES > self.data.len() {
            return None;
        }

        let item_bytes = &self.data[self.offset..self.offset + T::BYTES];

        self.offset += T::BYTES;

        Some(T::deserialize(item_bytes))
    }
}

impl<T> io::Seek for ConstIterableStoreReader<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(offset) => {
                self.offset = offset as usize;
            }
            io::SeekFrom::End(offset) => {
                self.offset = self.data.len() - offset as usize;
            }
            io::SeekFrom::Current(offset) => {
                self.offset += offset as usize;
            }
        }

        Ok(self.offset as u64)
    }
}
