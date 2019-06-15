use bitflags::bitflags;
use core::fmt;

bitflags! {
    /// RAM data for LED display.
    ///
    /// The LED for the corresponding bitflag will be enabled if the flag is `1`.
    pub struct DisplayData: u8 {
        /// No LEDs enabled.
        const COMMON_NONE = 0b0000_0000;
        /// Led on common 0 enabled.
        const COMMON_0 = 0b0000_0001;
        /// Led on common 1 enabled.
        const COMMON_1 = 0b0000_0010;
        /// Led on common 2 enabled.
        const COMMON_2 = 0b0000_0100;
        /// Led on common 3 enabled.
        const COMMON_3 = 0b0000_1000;
        /// Led on common 4 enabled.
        const COMMON_4 = 0b0001_0000;
        /// Led on common 5 enabled.
        const COMMON_5 = 0b0010_0000;
        /// Led on common 6 enabled.
        const COMMON_6 = 0b0100_0000;
        /// Led on common 7 enabled.
        const COMMON_7 = 0b1000_0000;
    }
}

impl Default for DisplayData {
    fn default() -> DisplayData {
        DisplayData::COMMON_NONE
    }
}

impl fmt::Display for DisplayData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DisplayData::COMMON_NONE => write!(f, "DisplayData::COMMON_NONE"),
            DisplayData::COMMON_0 => write!(f, "DisplayData::COMMON_0"),
            DisplayData::COMMON_1 => write!(f, "DisplayData::COMMON_1"),
            DisplayData::COMMON_2 => write!(f, "DisplayData::COMMON_2"),
            DisplayData::COMMON_3 => write!(f, "DisplayData::COMMON_3"),
            DisplayData::COMMON_4 => write!(f, "DisplayData::COMMON_4"),
            DisplayData::COMMON_5 => write!(f, "DisplayData::COMMON_5"),
            DisplayData::COMMON_6 => write!(f, "DisplayData::COMMON_6"),
            DisplayData::COMMON_7 => write!(f, "DisplayData::COMMON_7"),
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
