use core::fmt;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use DisplayDataAddress::*;

/// Display RAM data address.
#[allow(non_camel_case_types)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, 
    IntoPrimitive, TryFromPrimitive
)]
#[repr(u8)]
pub enum DisplayDataAddress {
    /// Row 0
    ROW_0 = 0,
    /// Row 1
    ROW_1 = 1,
    /// Row 2
    ROW_2 = 2,
    /// Row 3
    ROW_3 = 3,
    /// Row 4
    ROW_4 = 4,
    /// Row 5
    ROW_5 = 5,
    /// Row 6
    ROW_6 = 6,
    /// Row 7
    ROW_7 = 7,
    /// Row 8
    ROW_8 = 8,
    /// Row 9
    ROW_9 = 9,
    /// Row 10
    ROW_10 = 10,
    /// Row 11
    ROW_11 = 11,
    /// Row 12
    ROW_12 = 12,
    /// Row 13
    ROW_13 = 13,
    /// Row 14
    ROW_14 = 14,
    /// Row 15
    ROW_15 = 15,
}

impl DisplayDataAddress {
    /// Returns the bitvalue
    #[deprecated(since = "0.1.2", note = "Use `as u8` instead")]
    pub fn bits(&self) -> u8 {
        *self as u8
    }
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
