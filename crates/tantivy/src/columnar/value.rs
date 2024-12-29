use crate::common::DateTime;

use super::InvalidData;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum NumericalValue {
    I64(i64),
    U64(u64),
    F64(f64),
    U128(u128),
}

impl NumericalValue {
    pub fn numerical_type(&self) -> NumericalType {
        match self {
            NumericalValue::I64(_) => NumericalType::I64,
            NumericalValue::U64(_) => NumericalType::U64,
            NumericalValue::F64(_) => NumericalType::F64,
            NumericalValue::U128(_) => NumericalType::U128,
        }
    }
}

impl From<u64> for NumericalValue {
    fn from(val: u64) -> NumericalValue {
        NumericalValue::U64(val)
    }
}

impl From<i64> for NumericalValue {
    fn from(val: i64) -> Self {
        NumericalValue::I64(val)
    }
}

impl From<f64> for NumericalValue {
    fn from(val: f64) -> Self {
        NumericalValue::F64(val)
    }
}

impl From<u128> for NumericalValue {
    fn from(val: u128) -> Self {
        NumericalValue::U128(val)
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum NumericalType {
    #[default]
    I64 = 0,
    U64 = 1,
    F64 = 2,
    U128 = 3,
}

impl NumericalType {
    pub fn to_code(self) -> u8 {
        self as u8
    }

    pub fn try_from_code(code: u8) -> Result<NumericalType, InvalidData> {
        match code {
            0 => Ok(NumericalType::I64),
            1 => Ok(NumericalType::U64),
            2 => Ok(NumericalType::F64),
            3 => Ok(NumericalType::U128),
            _ => Err(InvalidData),
        }
    }
}

/// We voluntarily avoid using `Into` here to keep this
/// implementation quirk as private as possible.
///
/// # Panics
/// This coercion trait actually panics if it is used
/// to convert a loose types to a stricter type.
///
/// The level is strictness is somewhat arbitrary.
/// - i64
/// - u64
/// - f64.
pub(crate) trait Coerce {
    fn coerce(numerical_value: NumericalValue) -> Self;
}

impl Coerce for i64 {
    fn coerce(value: NumericalValue) -> Self {
        match value {
            NumericalValue::I64(val) => val,
            NumericalValue::U64(val) => val as i64,
            NumericalValue::U128(val) => val as i64,
            NumericalValue::F64(_) => unreachable!(),
        }
    }
}

impl Coerce for u64 {
    fn coerce(value: NumericalValue) -> Self {
        match value {
            NumericalValue::I64(val) => val as u64,
            NumericalValue::U64(val) => val,
            NumericalValue::F64(_) => unreachable!(),
            NumericalValue::U128(val) => val as u64,
        }
    }
}

impl Coerce for f64 {
    fn coerce(value: NumericalValue) -> Self {
        match value {
            NumericalValue::I64(val) => val as f64,
            NumericalValue::U64(val) => val as f64,
            NumericalValue::F64(val) => val,
            NumericalValue::U128(val) => val as f64,
        }
    }
}

impl Coerce for DateTime {
    fn coerce(value: NumericalValue) -> Self {
        let timestamp_micros = i64::coerce(value);
        DateTime::from_timestamp_nanos(timestamp_micros)
    }
}

impl Coerce for u128 {
    fn coerce(value: NumericalValue) -> Self {
        match value {
            NumericalValue::U128(val) => val,
            NumericalValue::I64(val) => val as u128,
            NumericalValue::U64(val) => val as u128,
            NumericalValue::F64(_) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NumericalType;

    #[test]
    fn test_numerical_type_code() {
        let mut num_numerical_type = 0;
        for code in u8::MIN..=u8::MAX {
            if let Ok(numerical_type) = NumericalType::try_from_code(code) {
                assert_eq!(numerical_type.to_code(), code);
                num_numerical_type += 1;
            }
        }
        assert_eq!(num_numerical_type, 4);
    }
}