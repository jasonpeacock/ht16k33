use bitflags::bitflags;
use core::fmt;

bitflags! {
    /// RAM data for LED display.
    ///
    /// The LED for the corresponding bitflag will be enabled if the flag is `1`.
    pub struct DisplayData: u16 {
        /// No LEDs enabled.
        const ROW_NONE = 0b0000_0000_0000_0000;
        /// Led on row 0 enabled.
        const ROW_0 = 0b0000_0000_0000_0001;
        /// Led on row 1 enabled.
        const ROW_1 = 0b0000_0000_0000_0010;
        /// Led on row 2 enabled.
        const ROW_2 = 0b0000_0000_0000_0100;
        /// Led on row 3 enabled.
        const ROW_3 = 0b0000_0000_0000_1000;
        /// Led on row 4 enabled.
        const ROW_4 = 0b0000_0000_0001_0000;
        /// Led on row 5 enabled.
        const ROW_5 = 0b0000_0000_0010_0000;
        /// Led on row 6 enabled.
        const ROW_6 = 0b0000_0000_0100_0000;
        /// Led on row 7 enabled.
        const ROW_7 = 0b0000_0000_1000_0000;
        /// Led on row 8 enabled.
        const ROW_8 = 0b0000_0001_0000_0000;
        /// Led on row 9 enabled.
        const ROW_9 = 0b0000_0010_0000_0000;
        /// Led on row 10 enabled.
        const ROW_10 = 0b0000_0100_0000_0000;
        /// Led on row 11 enabled.
        const ROW_11 = 0b0000_1000_0000_0000;
        /// Led on row 12 enabled.
        const ROW_12 = 0b0001_0000_0000_0000;
        /// Led on row 13 enabled.
        const ROW_13 = 0b0010_0000_0000_0000;
        /// Led on row 14 enabled.
        const ROW_14 = 0b0100_0000_0000_0000;
        /// Led on row 15 enabled.
        const ROW_15 = 0b1000_0000_0000_0000;
    }
}

impl Default for DisplayData {
    fn default() -> DisplayData {
        DisplayData::ROW_NONE
    }
}

impl fmt::Display for DisplayData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DisplayData::ROW_NONE => write!(f, "DisplayData::ROW_NONE"),
            DisplayData::ROW_0 => write!(f, "DisplayData::ROW_0"),
            DisplayData::ROW_1 => write!(f, "DisplayData::ROW_1"),
            DisplayData::ROW_2 => write!(f, "DisplayData::ROW_2"),
            DisplayData::ROW_3 => write!(f, "DisplayData::ROW_3"),
            DisplayData::ROW_4 => write!(f, "DisplayData::ROW_4"),
            DisplayData::ROW_5 => write!(f, "DisplayData::ROW_5"),
            DisplayData::ROW_6 => write!(f, "DisplayData::ROW_6"),
            DisplayData::ROW_7 => write!(f, "DisplayData::ROW_7"),
            DisplayData::ROW_8 => write!(f, "DisplayData::ROW_8"),
            DisplayData::ROW_9 => write!(f, "DisplayData::ROW_9"),
            DisplayData::ROW_10 => write!(f, "DisplayData::ROW_10"),
            DisplayData::ROW_11 => write!(f, "DisplayData::ROW_11"),
            DisplayData::ROW_12 => write!(f, "DisplayData::ROW_12"),
            DisplayData::ROW_13 => write!(f, "DisplayData::ROW_13"),
            DisplayData::ROW_14 => write!(f, "DisplayData::ROW_14"),
            DisplayData::ROW_15 => write!(f, "DisplayData::ROW_15"),
            _ => write!(f, "DisplayData::{:#10b}", self.bits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(
            DisplayData::COMMON_NONE,
            DisplayData::default(),
            "DisplayData default COMMON_NONE"
        );
    }

    #[test]
    fn all_on() {
        let data = DisplayData::COMMON_0
            | DisplayData::COMMON_1
            | DisplayData::COMMON_2
            | DisplayData::COMMON_3
            | DisplayData::COMMON_4
            | DisplayData::COMMON_5
            | DisplayData::COMMON_6
            | DisplayData::COMMON_7;

        assert_eq!(data, DisplayData::all(), "DisplayData is all enabled");
    }
}
