use errors::ValidationError;

use std::default;
use std::fmt;

bitflags! {
    /// Display dimming.
    ///
    /// The whole display is dimmed via PWM @ N/16 duty cycle; individual LEDs cannot be dimmed independently.
    ///
    /// The value should be in the inclusive range [`BRIGHTNESS_MIN`] to [`BRIGHTNESS_MAX`]. Use the [`from_u8`]
    /// helper to create a validated `Dimming` value.
    ///
    /// [`BRIGHTNESS_MIN`]: struct.Dimming.html#associatedconstant.BRIGHTNESS_MIN
    /// [`BRIGHTNESS_MAX`]: struct.Dimming.html#associatedconstant.BRIGHTNESS_MAX
    /// [`from_u8`]: struct.Dimming.html#method.from_u8
    pub struct Dimming: u8 {
        /// Command to set the digital dimming.
        const COMMAND = 0b1110_0000;
        /// Minimum brightness @ 1/16 PWM duty cycle. (Same as `BRIGHTNESS_1_16`)
        const BRIGHTNESS_MIN = Self::BRIGHTNESS_1_16.bits;
        /// Brightness @ 1/16 PWM duty cycle.
        const BRIGHTNESS_1_16 = 0;
        /// Brightness @ 2/16 PWM duty cycle.
        const BRIGHTNESS_2_16 = 1;
        /// Brightness @ 3/16 PWM duty cycle.
        const BRIGHTNESS_3_16 = 2;
        /// Brightness @ 4/16 PWM duty cycle.
        const BRIGHTNESS_4_16 = 3;
        /// Brightness @ 5/16 PWM duty cycle.
        const BRIGHTNESS_5_16 = 4;
        /// Brightness @ 6/16 PWM duty cycle.
        const BRIGHTNESS_6_16 = 5;
        /// Brightness @ 7/16 PWM duty cycle.
        const BRIGHTNESS_7_16 = 6;
        /// Brightness @ 8/16 PWM duty cycle.
        const BRIGHTNESS_8_16 = 7;
        /// Brightness @ 9/16 PWM duty cycle.
        const BRIGHTNESS_9_16 = 8;
        /// Brightness @ 10/16 PWM duty cycle.
        const BRIGHTNESS_10_16 = 9;
        /// Brightness @ 11/16 PWM duty cycle.
        const BRIGHTNESS_11_16 = 10;
        /// Brightness @ 12/16 PWM duty cycle.
        const BRIGHTNESS_12_16 = 11;
        /// Brightness @ 13/16 PWM duty cycle.
        const BRIGHTNESS_13_16 = 12;
        /// Brightness @ 14/16 PWM duty cycle.
        const BRIGHTNESS_14_16 = 13;
        /// Brightness @ 15/16 PWM duty cycle.
        const BRIGHTNESS_15_16 = 14;
        /// Brightness @ 16/16 PWM duty cycle.
        const BRIGHTNESS_16_16 = 15;
        /// Maximum brightness @ 16/16 PWM duty cycle. (Same as `BRIGHTNESS_16_16`)
        ///
        /// *This is the Power-on Reset default.*
        const BRIGHTNESS_MAX = Self::BRIGHTNESS_16_16.bits;
    }
}

impl default::Default for Dimming {
    fn default() -> Dimming {
        Dimming::BRIGHTNESS_MAX
    }
}

impl fmt::Display for Dimming {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Dimming::COMMAND => write!(f, "Dimming::COMMAND"),
            Dimming::BRIGHTNESS_MIN => write!(f, "Dimming::BRIGHTNESS_MIN"),
            Dimming::BRIGHTNESS_MAX => write!(f, "Dimming::BRIGHTNESS_MAX"),
            _ => write!(f, "Dimming::{:#10b}", self.bits()),
        }
    }
}

impl Dimming {
    /// Return a validated `Dimming` value from the given `u8`.
    ///
    /// *NOTE: The brightness values are 0-indexed, e.g. `0u8` is equivalent to `1/16`, and `15u8` is `16/16`.*
    ///
    /// # Errors
    ///
    /// The value is validated to be in the inclusive range [`BRIGHTNESS_MIN`] to [`BRIGHTNESS_MAX`]. If
    /// the given `u8` value is too large then [`ht16k33::ValidationError::ValueTooLarge`] is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate failure;
    /// # extern crate ht16k33;
    /// # use failure::Error;
    /// use ht16k33::Dimming;
    /// # fn main() -> Result<(), Error> {
    ///
    /// let brightness = Dimming::from_u8(1u8)?;
    ///
    /// assert_eq!(1u8, brightness.bits());
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Error Example
    ///
    /// ```should_panic
    /// # extern crate ht16k33;
    /// use ht16k33::Dimming;
    /// use ht16k33::ValidationError;
    /// # fn main() {
    ///
    /// // Greater than the `BRIGHTNESS_MAX` value of `15u8`.
    /// let value = 16u8;
    ///
    /// let brightness = match Dimming::from_u8(value) {
    ///     Ok(brightness) => brightness,
    ///     Err(ValidationError) => panic!(),
    /// };
    ///
    /// # }
    /// ```
    ///
    /// [`BRIGHTNESS_MIN`]: struct.Dimming.html#associatedconstant.BRIGHTNESS_MIN
    /// [`BRIGHTNESS_MAX`]: struct.Dimming.html#associatedconstant.BRIGHTNESS_MAX
    /// [`ht16k33::ValidationError::ValueTooLarge`]: enum.ValidationError.html#variant.ValueTooLarge
    // TODO Implement as TryFrom<u8> once it's available in `stable`.
    pub fn from_u8(value: u8) -> Result<Self, ValidationError> {
        if value > Dimming::BRIGHTNESS_MAX.bits() {
            return Err(ValidationError::ValueTooLarge {
                name: "Dimming".to_string(),
                value,
                limit: Dimming::BRIGHTNESS_MAX.bits(),
                inclusive: true,
            });
        }

        Ok(Dimming::from_bits_truncate(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_min() {
        assert_eq!(
            Dimming::BRIGHTNESS_1_16,
            Dimming::BRIGHTNESS_MIN,
            "Dimming MIN brightness matches 1/16 value"
        );
    }

    #[test]
    fn brightness_max() {
        assert_eq!(
            Dimming::BRIGHTNESS_16_16,
            Dimming::BRIGHTNESS_MAX,
            "Dimming MAX brightness matches 16/16 value"
        );
    }

    #[test]
    fn default() {
        assert_eq!(
            Dimming::BRIGHTNESS_MAX,
            Dimming::default(),
            "Dimming default is MAX brightness"
        );
    }

    #[test]
    fn from_u8() {
        for value in 0u8..16 {
            let dimming = Dimming::from_u8(value).unwrap();
            assert_eq!(value, dimming.bits());
        }
    }

    #[test]
    #[should_panic]
    fn from_u8_too_large() {
        let _ = Dimming::from_u8(16u8).unwrap();
    }
}
