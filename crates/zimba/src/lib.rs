/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief zimba
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
pub mod wiki;

pub use wiki::{Article, ArticleIterator, Image, ImageIterator};

use nom::{
    bytes::complete::{take, take_while},
    combinator::map,
    IResult,
};

use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unexpected end of bytes")]
    UnexpectedEndOfBytes,

    #[error("Invalid magic number")]
    InvalidMagicNumber,

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Invalid compression type")]
    InvalidCompressionType,

    #[error("LZMA error: {0}")]
    Lzma(#[from] lzma::Error),
}

fn read_zero_terminated(bytes: &[u8]) -> IResult<&[u8], String> {
    let (remaining, string) = map(take_while(|b| b != 0), |bytes: &[u8]| {
        String::from_utf8_lossy(bytes).into_owned()
    })(bytes)?;

    let (remaining, zero) = take(1usize)(remaining)?;
    if zero != [0] {
        return Err(nom::Err::Error(nom::error::Error::new(
            remaining,
            nom::error::ErrorKind::Tag,
        )));
    }

    Ok((remaining, string))
}

trait NomParseNumber: Sized {
    fn nom_parse_le(bytes: &[u8]) -> IResult<&[u8], Self>;
}

macro_rules! read_u {
    ($type:ty) => {
        impl NomParseNumber for $type {
            fn nom_parse_le(bytes: &[u8]) -> IResult<&[u8], Self> {
                let (remaining, bytes) = take(std::mem::size_of::<$type>())(bytes)?;

                if bytes.len() < std::mem::size_of::<$type>() {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        remaining,
                        nom::error::ErrorKind::Eof,
                    )));
                }

                Ok((remaining, Self::from_le_bytes(bytes.try_into().unwrap())))
            }
        }
    };
}

read_u!(u16);
read_u!(u32);
read_u!(u64);
read_u!(u128);

impl NomParseNumber for char {
    fn nom_parse_le(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, byte) = take(1usize)(bytes)?;
        Ok((remaining, byte[0] as char))
    }
}

impl NomParseNumber for u8 {
    fn nom_parse_le(bytes: &[u8]) -> IResult<&[u8], Self> {
        let (remaining, byte) = take(1usize)(bytes)?;
        Ok((remaining, byte[0]))
    }
}

#[derive(Debug)]
#[allow(unused)]
struct Header {
    magic: u32,

    major_version: u16,

    minor_version: u16,

    uuid: u128,

    entry_count: u32,

    cluster_count: u32,

    url_ptr_pos: u64,

    title_ptr_pos: u64,

    cluster_ptr_pos: u64,

    mime_list_pos: u64,

    main_page: u32,

    layout_page: u32,

    checksum_pos: u64,
}

impl Header {
    fn from_bytes(bytes: &[u8]) -> Result<Header, Error> {
        if bytes.len() < 80 {
            return Err(Error::UnexpectedEndOfBytes);
        }

        let (remaining, magic) =
            u32::nom_parse_le(bytes).map_err(|_| Error::UnexpectedEndOfBytes)?;

        if magic != 72_173_914 {
            return Err(Error::InvalidMagicNumber);
        }

        let (remaining, major_version) =
            u16::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, minor_version) =
            u16::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        let (remaining, uuid) =
            u128::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        let (remaining, entry_count) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, cluster_count) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        let (remaining, url_ptr_pos) =
            u64::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, title_ptr_pos) =
            u64::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, cluster_ptr_pos) =
            u64::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, mime_list_pos) =
            u64::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        let (remaining, main_page) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, layout_page) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        let (_remaining, checksum_pos) =
            u64::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        Ok(Header {
            magic,
            major_version,
            minor_version,
            uuid,
            entry_count,
            cluster_count,
            url_ptr_pos,
            title_ptr_pos,
            cluster_ptr_pos,
            mime_list_pos,
            main_page,
            layout_page,
            checksum_pos,
        })
    }
}

#[derive(Debug)]
pub struct MimeTypes(Vec<String>);

impl MimeTypes {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let mut mime_types = Vec::new();

        let mut bytes = bytes;
        while !bytes.is_empty() {
            let (remaining, mime_type) =
                read_zero_terminated(bytes).map_err(|_| Error::UnexpectedEndOfBytes)?;

            if mime_type.is_empty() {
                break;
            }

            mime_types.push(mime_type);

            bytes = remaining;
        }

        Ok(Self(mime_types))
    }
}

impl std::ops::Index<u16> for MimeTypes {
    type Output = String;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

#[derive(Debug)]
pub struct UrlPointer(pub u64);

#[derive(Debug)]
pub struct UrlPointerList(Vec<UrlPointer>);

impl std::ops::Index<u32> for UrlPointerList {
    type Output = UrlPointer;

    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl UrlPointerList {
    fn from_bytes(bytes: &[u8], num_urls: u32) -> Result<Self, Error> {
        let mut url_pointers = Vec::new();

        let mut bytes = bytes;
        for _ in 0..num_urls {
            let (remaining, url_pointer) =
                u64::nom_parse_le(bytes).map_err(|_| Error::UnexpectedEndOfBytes)?;

            let url_pointer = UrlPointer(url_pointer);
            url_pointers.push(url_pointer);

            bytes = remaining;
        }

        Ok(Self(url_pointers))
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct TitlePointer(u32);

#[derive(Debug)]
pub struct TitlePointerList(Vec<TitlePointer>);

impl std::ops::Index<u32> for TitlePointerList {
    type Output = TitlePointer;

    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl TitlePointerList {
    fn from_bytes(bytes: &[u8], num_titles: u32) -> Result<Self, Error> {
        let mut title_pointers = Vec::new();

        let mut bytes = bytes;
        for _ in 0..num_titles {
            let (remaining, title_pointer) =
                u32::nom_parse_le(bytes).map_err(|_| Error::UnexpectedEndOfBytes)?;

            let title_pointer = TitlePointer(title_pointer);

            title_pointers.push(title_pointer);

            bytes = remaining;
        }

        Ok(Self(title_pointers))
    }
}

#[derive(Debug)]
struct ClusterPointer(u64);

#[derive(Debug)]
struct ClusterPointerList(Vec<ClusterPointer>);

impl std::ops::Index<u32> for ClusterPointerList {
    type Output = ClusterPointer;

    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl ClusterPointerList {
    fn from_bytes(bytes: &[u8], num_clusters: u32) -> Result<Self, Error> {
        let mut cluster_pointers = Vec::new();

        let mut bytes = bytes;
        for _ in 0..num_clusters {
            let (remaining, cluster_pointer) =
                u64::nom_parse_le(bytes).map_err(|_| Error::UnexpectedEndOfBytes)?;

            let cluster_pointer = ClusterPointer(cluster_pointer);

            cluster_pointers.push(cluster_pointer);

            bytes = remaining;
        }

        Ok(Self(cluster_pointers))
    }
}

#[derive(Debug)]
pub enum DirEntry {
    Content {
        mime_type: u16,
        parameter_len: u8,
        namespace: char,
        revision: u32,
        cluster_number: u32,
        blob_number: u32,
        url: String,
        title: String,
    },
    Redirect {
        mime_type: u16,
        parameter_len: u8,
        namespace: char,
        revision: u32,
        redirect_index: u32,
        url: String,
        title: String,
    },
}

fn u8_reader_parser(bytes: &mut impl Iterator<Item = Result<u8, io::Error>>) -> Result<u8, Error> {
    Ok(bytes.next().ok_or(Error::UnexpectedEndOfBytes)??)
}

fn u32_reader_parser(
    bytes: &mut impl Iterator<Item = Result<u8, io::Error>>,
) -> Result<u32, Error> {
    Ok(u32::from_le_bytes([
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
    ]))
}

fn u64_reader_parser(
    bytes: &mut impl Iterator<Item = Result<u8, io::Error>>,
) -> Result<u64, Error> {
    Ok(u64::from_le_bytes([
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
        bytes.next().ok_or(Error::UnexpectedEndOfBytes)??,
    ]))
}

impl DirEntry {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let (remaining, mime_type) =
            u16::nom_parse_le(bytes).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, parameter_len) =
            u8::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, namespace) =
            char::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, revision) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        if mime_type == 0xffff {
            let (remaining, redirect_index) =
                u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
            let (remaining, url) =
                read_zero_terminated(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
            let (_, title) =
                read_zero_terminated(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
            return Ok(Self::Redirect {
                mime_type,
                parameter_len,
                namespace,
                revision,
                redirect_index,
                url,
                title,
            });
        }

        let (remaining, cluster_number) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (remaining, blob_number) =
            u32::nom_parse_le(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        let (remaining, url) =
            read_zero_terminated(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;
        let (_, title) =
            read_zero_terminated(remaining).map_err(|_| Error::UnexpectedEndOfBytes)?;

        Ok(Self::Content {
            mime_type,
            parameter_len,
            namespace,
            revision,
            cluster_number,
            blob_number,
            url,
            title,
        })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OffsetSize {
    U32,
    U64,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum CompressionType {
    Uncompressed,
    Lzma,
    Zstd,
}

enum CompressedReader<'a> {
    Uncompressed(BufReader<std::io::Cursor<&'a [u8]>>),
    Lzma(Box<BufReader<lzma::Reader<BufReader<&'a [u8]>>>>),
    Zstd(BufReader<zstd::Decoder<'a, BufReader<&'a [u8]>>>),
}

impl std::io::Read for CompressedReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            CompressedReader::Uncompressed(reader) => reader.read(buf),
            CompressedReader::Lzma(reader) => reader.read(buf),
            CompressedReader::Zstd(reader) => reader.read(buf),
        }
    }
}

#[derive(Debug)]
struct ClusterOffset {
    offset: u64,
}

#[derive(Debug)]
pub struct Cluster {
    blob_offsets: Vec<ClusterOffset>,
    blobs: Vec<u8>,
}

impl Cluster {
    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let cluster_info = bytes[0];
        let comp_info = cluster_info & 0x0F;
        let extended = cluster_info & 0x10;

        let size = if extended == 0 {
            OffsetSize::U32
        } else {
            OffsetSize::U64
        };

        let compression_type = match comp_info {
            0 => CompressionType::Uncompressed,
            1 => CompressionType::Uncompressed,
            4 => CompressionType::Lzma,
            5 => CompressionType::Zstd,
            _ => return Err(Error::InvalidCompressionType),
        };

        let mut reader = match compression_type {
            CompressionType::Uncompressed => {
                CompressedReader::Uncompressed(BufReader::new(std::io::Cursor::new(&bytes[1..])))
            }
            CompressionType::Lzma => {
                let decoder = lzma::Reader::from(BufReader::new(&bytes[1..]))?;
                CompressedReader::Lzma(Box::new(BufReader::new(decoder)))
            }
            CompressionType::Zstd => {
                let decoder = zstd::Decoder::new(&bytes[1..])?;
                CompressedReader::Zstd(BufReader::new(decoder))
            }
        }
        .bytes();

        let mut blob_offsets = Vec::new();

        match size {
            OffsetSize::U32 => {
                blob_offsets.push(ClusterOffset {
                    offset: u64::from(u32_reader_parser(&mut reader)?),
                });
            }
            OffsetSize::U64 => {
                blob_offsets.push(ClusterOffset {
                    offset: u64_reader_parser(&mut reader)?,
                });
            }
        }

        let num_offsets = match size {
            OffsetSize::U32 => blob_offsets[0].offset as u32 / 4,
            OffsetSize::U64 => blob_offsets[0].offset as u32 / 8,
        };

        for _ in 1..num_offsets {
            match size {
                OffsetSize::U32 => {
                    blob_offsets.push(ClusterOffset {
                        offset: u64::from(u32_reader_parser(&mut reader)?),
                    });
                }
                OffsetSize::U64 => {
                    blob_offsets.push(ClusterOffset {
                        offset: u64_reader_parser(&mut reader)?,
                    });
                }
            }
        }

        let bytes_read = blob_offsets.len()
            * match size {
                OffsetSize::U32 => 4,
                OffsetSize::U64 => 8,
            };

        let missing_bytes = blob_offsets.last().unwrap().offset as usize - bytes_read;

        let mut blobs = Vec::new();

        for _ in 0..missing_bytes {
            blobs.push(u8_reader_parser(&mut reader)?);
        }

        Ok(Self {
            blob_offsets,
            blobs,
        })
    }

    #[must_use]
    pub fn get_blob(&self, blob_number: usize) -> Option<&[u8]> {
        if self.blob_offsets.is_empty() {
            return None;
        }

        if blob_number >= self.blob_offsets.len() - 1 {
            return None;
        }

        let offset =
            self.blob_offsets[blob_number].offset as usize - self.blob_offsets[0].offset as usize;
        let next_offset = self.blob_offsets[blob_number + 1].offset as usize
            - self.blob_offsets[0].offset as usize;

        Some(&self.blobs[offset..next_offset])
    }
}

pub struct ZimFile {
    header: Header,
    mime_types: MimeTypes,
    url_pointers: UrlPointerList,
    title_pointers: TitlePointerList,
    cluster_pointers: ClusterPointerList,
    mmap: memmap2::Mmap,
}

impl ZimFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<ZimFile, Error> {
        let file = File::open(path)?;
        let mmap = unsafe { memmap2::MmapOptions::new().map(&file)? };

        let header = Header::from_bytes(&mmap)?;

        if header.magic != 72_173_914 {
            return Err(Error::InvalidMagicNumber);
        }

        let mime_types = MimeTypes::from_bytes(&mmap[header.mime_list_pos as usize..])?;
        let url_pointers =
            UrlPointerList::from_bytes(&mmap[header.url_ptr_pos as usize..], header.entry_count)?;

        let title_pointers = TitlePointerList::from_bytes(
            &mmap[header.title_ptr_pos as usize..],
            header.entry_count,
        )?;

        let cluster_pointers = ClusterPointerList::from_bytes(
            &mmap[header.cluster_ptr_pos as usize..],
            header.cluster_count,
        )?;

        if cluster_pointers.0.len() != header.cluster_count as usize {
            return Err(Error::UnexpectedEndOfBytes);
        }

        Ok(Self {
            header,
            mime_types,
            url_pointers,
            title_pointers,
            cluster_pointers,
            mmap,
        })
    }

    pub fn get_dir_entry(&self, index: usize) -> Result<Option<DirEntry>, Error> {
        if index >= self.header.entry_count as usize {
            return Ok(None);
        }

        let pointer = self.url_pointers.0[index].0 as usize;
        Ok(Some(DirEntry::from_bytes(&self.mmap[pointer..])?))
    }

    pub fn get_cluster(&self, index: u32) -> Result<Option<Cluster>, Error> {
        if index >= self.header.cluster_count {
            return Ok(None);
        }

        let pointer = self.cluster_pointers[index].0 as usize;
        Ok(Some(Cluster::from_bytes(&self.mmap[pointer..])?))
    }

    #[must_use]
    pub fn mime_types(&self) -> &MimeTypes {
        &self.mime_types
    }

    #[must_use]
    pub fn url_pointers(&self) -> &UrlPointerList {
        &self.url_pointers
    }

    #[must_use]
    pub fn title_pointers(&self) -> &TitlePointerList {
        &self.title_pointers
    }

    #[must_use]
    pub fn dir_entries(&self) -> DirEntryIterator<'_> {
        DirEntryIterator::new(&self.mmap, &self.url_pointers)
    }

    pub fn articles(&self) -> Result<ArticleIterator<'_>, Error> {
        ArticleIterator::new(self)
    }

    pub fn images(&self) -> Result<ImageIterator<'_>, Error> {
        ImageIterator::new(self)
    }
}

pub struct DirEntryIterator<'a> {
    mmap: &'a memmap2::Mmap,
    url_pointers: &'a UrlPointerList,
    counter: usize,
}

impl<'a> DirEntryIterator<'a> {
    fn new(mmap: &'a memmap2::Mmap, url_pointers: &'a UrlPointerList) -> Self {
        Self {
            mmap,
            url_pointers,
            counter: 0,
        }
    }
}

impl Iterator for DirEntryIterator<'_> {
    type Item = Result<DirEntry, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter >= self.url_pointers.0.len() {
            return None;
        }

        let pointer = self.url_pointers.0[self.counter].0 as usize;
        self.counter += 1;

        Some(DirEntry::from_bytes(&self.mmap[pointer..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data_path = Path::new("../../data/test.zim");
        if !data_path.exists() {
            return;
        }

        let zim = ZimFile::open(data_path).unwrap();

        assert_eq!(zim.header.magic, 72_173_914);
        assert_eq!(zim.header.major_version, 5);
        assert_eq!(zim.header.minor_version, 0);

        let first_article = zim
            .dir_entries()
            .find(|x| match x {
                Ok(DirEntry::Content { namespace, .. }) => *namespace == 'A',
                _ => false,
            })
            .unwrap()
            .unwrap();

        let url = match first_article {
            DirEntry::Content { url, .. } => url,
            _ => panic!(),
        };

        assert_eq!(url, "African_Americans");
        assert_eq!(zim.dir_entries().count(), 8477);
    }
}
