use std::default;
use std::fmt;

bitflags! {
    /// The LED display state.
    ///
    /// The LEDs can be all off (default), all on, or all blinking at 1/2Hz, 1Hz, or 2Hz.
    pub struct Display: u8 {
        /// Command to set the display.
        const COMMAND = 0b1000_0000;
        /// Display on; blinking off.
        const ON = 0b0000_0001;
        /// Display off.
        ///
        /// *This is the Power-on Reset default.*
        const OFF = 0b0000_0000;
        /// Display on; blinking @ 0.5Hz.
        const HALF_HZ = 0b0000_0110 | Self::ON.bits;
        /// Display on; blinking @ 1Hz.
        const ONE_HZ = 0b0000_0100 | Self::ON.bits;
        /// Display on; blinking @ 2Hz.
        const TWO_HZ = 0b0000_0010 | Self::ON.bits;
    }
}

impl default::Default for Display {
    fn default() -> Display {
        Display::OFF
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Display::COMMAND => write!(f, "Display::COMMAND"),
            Display::ON => write!(f, "Display::ON"),
            Display::OFF => write!(f, "Display::OFF"),
            Display::HALF_HZ => write!(f, "Display::HALF_HZ"),
            Display::ONE_HZ => write!(f, "Display::ONE_HZ"),
            Display::TWO_HZ => write!(f, "Display::TWO_HZ"),
            _ => write!(f, "Display::{:#10b}", self.bits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(Display::OFF, Display::default(), "Display default is OFF");
    }

    #[test]
    fn blink_includes_on() {
        assert!(
            Display::HALF_HZ.contains(Display::ON),
            "HALF_HZ includes ON"
        );
        assert!(Display::ONE_HZ.contains(Display::ON), "ONE_HZ includes ON");
        assert!(Display::TWO_HZ.contains(Display::ON), "TWO_HZ includes ON");
    }
}
