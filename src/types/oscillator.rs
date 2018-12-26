use std::default;
use std::fmt;

bitflags! {
    /// System oscillator setup and control.
    pub struct Oscillator: u8 {
        /// Command to set system setup.
        const COMMAND = 0b0010_0000;
        /// Normal operation mode.
        const ON = 0b0000_0001;
        /// Standby mode.
        ///
        /// *This is the Power-on Reset default.*
        const OFF = 0b0000_0000;
    }
}

impl default::Default for Oscillator {
    fn default() -> Oscillator {
        Oscillator::OFF
    }
}

impl fmt::Display for Oscillator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Oscillator::COMMAND => write!(f, "Oscillator::COMMAND"),
            Oscillator::ON => write!(f, "Oscillator::ON"),
            Oscillator::OFF => write!(f, "Oscillator::OFF"),
            _ => write!(f, "Oscillator::{:#10b}", self.bits()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        assert_eq!(
            Oscillator::OFF,
            Oscillator::default(),
            "Oscillator default is OFF"
        );
    }
}
