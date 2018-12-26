use std::default;
use std::fmt;

bitflags! {
    /// Display RAM data address.
    pub struct DisplayDataAddress: u8 {
        /// Row 0
        const ROW_0 = 0;
        /// Row 1
        const ROW_1 = 1;
        /// Row 2
        const ROW_2 = 2;
        /// Row 3
        const ROW_3 = 3;
        /// Row 4
        const ROW_4 = 4;
        /// Row 5
        const ROW_5 = 5;
        /// Row 6
        const ROW_6 = 6;
        /// Row 7
        const ROW_7 = 7;
        /// Row 8
        const ROW_8 = 8;
        /// Row 9
        const ROW_9 = 9;
        /// Row 10
        const ROW_10 = 10;
        /// Row 11
        const ROW_11 = 11;
        /// Row 12
        const ROW_12 = 12;
        /// Row 13
        const ROW_13 = 13;
        /// Row 14
        const ROW_14 = 14;
        /// Row 15
        const ROW_15 = 15;
    }
}

impl default::Default for DisplayDataAddress {
    fn default() -> DisplayDataAddress {
        DisplayDataAddress::ROW_0
    }
}

impl fmt::Display for DisplayDataAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DisplayDataAddress::ROW_0 => write!(f, "DisplayDataAddress::ROW_0"),
            DisplayDataAddress::ROW_1 => write!(f, "DisplayDataAddress::ROW_1"),
            DisplayDataAddress::ROW_2 => write!(f, "DisplayDataAddress::ROW_2"),
            DisplayDataAddress::ROW_3 => write!(f, "DisplayDataAddress::ROW_3"),
            DisplayDataAddress::ROW_4 => write!(f, "DisplayDataAddress::ROW_4"),
            DisplayDataAddress::ROW_5 => write!(f, "DisplayDataAddress::ROW_5"),
            DisplayDataAddress::ROW_6 => write!(f, "DisplayDataAddress::ROW_6"),
            DisplayDataAddress::ROW_7 => write!(f, "DisplayDataAddress::ROW_7"),
            DisplayDataAddress::ROW_8 => write!(f, "DisplayDataAddress::ROW_8"),
            DisplayDataAddress::ROW_9 => write!(f, "DisplayDataAddress::ROW_9"),
            DisplayDataAddress::ROW_10 => write!(f, "DisplayDataAddress::ROW_10"),
            DisplayDataAddress::ROW_11 => write!(f, "DisplayDataAddress::ROW_11"),
            DisplayDataAddress::ROW_12 => write!(f, "DisplayDataAddress::ROW_12"),
            DisplayDataAddress::ROW_13 => write!(f, "DisplayDataAddress::ROW_13"),
            DisplayDataAddress::ROW_14 => write!(f, "DisplayDataAddress::ROW_14"),
            DisplayDataAddress::ROW_15 => write!(f, "DisplayDataAddress::ROW_15"),
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
