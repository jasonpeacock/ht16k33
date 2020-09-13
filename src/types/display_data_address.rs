use num_enum::{IntoPrimitive, TryFromPrimitive};
use core::fmt;

use DisplayDataAddress::*;

/// Display RAM data address.
#[allow(missing_docs, non_camel_case_types)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, 
    IntoPrimitive, TryFromPrimitive
)]
#[repr(u8)]
pub enum DisplayDataAddress {
    ROW_0,
    ROW_1,
    ROW_2,
    ROW_3,
    ROW_4,
    ROW_5,
    ROW_6,
    ROW_7,
    ROW_8,
    ROW_9,
    ROW_10,
    ROW_11,
    ROW_12,
    ROW_13,
    ROW_14,
    ROW_15,
}

impl Default for DisplayDataAddress {
    fn default() -> DisplayDataAddress {
        ROW_0
    }
}

impl fmt::Display for DisplayDataAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DisplayDataAddress::{:?}", *self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        assert_eq!(format!("{}", ROW_0), "DisplayDataAddress::ROW_0");
        assert_eq!(format!("{:?}", ROW_1), "ROW_1");
    }
}
