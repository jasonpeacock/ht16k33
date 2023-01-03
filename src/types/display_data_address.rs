use bitflags::bitflags;
use core::fmt;

bitflags! {
    /// Display RAM data address in terms of `u16`, split into bytes when
    /// written to the 16 entries by 8 bit RAM of the HT16K33.
    pub struct DisplayDataAddress: u8 {
        /// Common 0 Address
        const COMMON_0 = 0;
        /// Common 1 Address
        const COMMON_1 = 1;
        /// Common 2 Address
        const COMMON_2 = 2;
        /// Common 3 Address
        const COMMON_3 = 3;
        /// Common 4 Address
        const COMMON_4 = 4;
        /// Common 5 Address
        const COMMON_5 = 5;
        /// Common 6 Address
        const COMMON_6 = 6;
        /// Common 7 Address
        const COMMON_7 = 7;
    }
}

impl Default for DisplayDataAddress {
    fn default() -> DisplayDataAddress {
        DisplayDataAddress::COMMON_0
    }
}

impl fmt::Display for DisplayDataAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DisplayDataAddress::COMMON_0 => write!(f, "DisplayDataAddress::COMMON_0"),
            DisplayDataAddress::COMMON_1 => write!(f, "DisplayDataAddress::COMMON_1"),
            DisplayDataAddress::COMMON_2 => write!(f, "DisplayDataAddress::COMMON_2"),
            DisplayDataAddress::COMMON_3 => write!(f, "DisplayDataAddress::COMMON_3"),
            DisplayDataAddress::COMMON_4 => write!(f, "DisplayDataAddress::COMMON_4"),
            DisplayDataAddress::COMMON_5 => write!(f, "DisplayDataAddress::COMMON_5"),
            DisplayDataAddress::COMMON_6 => write!(f, "DisplayDataAddress::COMMON_6"),
            DisplayDataAddress::COMMON_7 => write!(f, "DisplayDataAddress::COMMON_7"),
            _ => write!(f, "DisplayDataAddress::{:#10b}", self.bits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(
            DisplayDataAddress::ROW_0,
            DisplayDataAddress::default(),
            "DisplayDataAddress default is row 0"
        );
    }
}
