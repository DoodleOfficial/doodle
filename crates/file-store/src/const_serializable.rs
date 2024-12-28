/**
 * @file const_serializable.rs
 * @author Krisna Pranav
 * @brief const serializable
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use std::ops::Range;

pub trait ConstSerializable {
    const BYTES: usize;

    fn serialize(&self, buf: &mut [u8]);
    fn deserialize(buf: &[u8]) -> Self;

    fn serialize_to_vec(&self) -> Vec<u8> {
        let mut buf = vec![0; Self::BYTES];
        self.serialize(&mut buf);
        buf
    }
}

impl ConstSerializable for Range<u64> {
    const BYTES: usize = std::mem::size_of::<u64>() * 2;

    fn serialize(&self, buf: &mut [u8]) {
        self.start.serialize(&mut buf[..std::mem::size_of::<u64>()]);
        self.end.serialize(&mut buf[std::mem::size_of::<u64>()..]);
    }

    fn deserialize(buf: &[u8]) -> Self {
        let start = u64::deserialize(&buf[..std::mem::size_of::<u64>()]);
        let end = u64::deserialize(&buf[std::mem::size_of::<u64>()..]);
        start..end
    }
}

macro_rules! impl_const_serializable_num {
    ($t:ty, $n:expr) => {
        impl ConstSerializable for $t {
            const BYTES: usize = $n;

            fn serialize(&self, buf: &mut [u8]) {
                buf[..Self::BYTES].copy_from_slice(&self.to_le_bytes());
            }

            fn deserialize(buf: &[u8]) -> Self {
                let mut bytes = [0; Self::BYTES];
                bytes.copy_from_slice(&buf[..Self::BYTES]);
                <$t>::from_le_bytes(bytes)
            }
        }
    };
}

impl_const_serializable_num!(u8, 1);
impl_const_serializable_num!(u16, 2);
impl_const_serializable_num!(u32, 4);
impl_const_serializable_num!(u64, 8);
impl_const_serializable_num!(u128, 16);

impl_const_serializable_num!(i8, 1);
impl_const_serializable_num!(i16, 2);
impl_const_serializable_num!(i32, 4);
impl_const_serializable_num!(i64, 8);
impl_const_serializable_num!(i128, 16);

impl_const_serializable_num!(f32, 4);
impl_const_serializable_num!(f64, 8);
