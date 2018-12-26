//! # HT16K33
//!
//! A driver for the [Holtek HT16K33](http://www.holtek.com/productdetail/-/vg/HT16K33).
//!
//! The set_*/read_*/write_* methods interact directly with the device. All other methods operate
//! on local state only.
extern crate embedded_hal as hal;

/// Re-export slog
///
/// Users of this library can, but don't have to, use slog to build their own loggers.
#[macro_use]
pub extern crate slog;
extern crate slog_stdlog;

pub mod i2c_mock;

use hal::blocking::i2c::{Write, WriteRead};

use slog::Drain;

const ROWS_SIZE: usize = 16;
const COMMONS_SIZE: usize = 8;

const DATA_ADDRESS: u8 = 0b0000_0000;
const DIMMING_SET: u8 = 0b1110_0000;
const DISPLAY_SET: u8 = 0b1000_0000;
const SYSTEM_SET: u8 = 0b0010_0000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Oscillator {
    /// Normal operation mode.
    On,
    /// Standby mode. Power-on default.
    Off,
}

impl Oscillator {
    fn value(self) -> u8 {
        match self {
            Oscillator::On => 0b0000_0001,
            Oscillator::Off => 0b0000_0000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Display {
    On,
    /// Power-on default.
    Off,
}

impl Display {
    fn value(self) -> u8 {
        match self {
            Display::On => 0b0000_0001,
            Display::Off => 0b0000_0000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Blink {
    /// Power-on default.
    Off,
    HalfHz,
    OneHz,
    TwoHz,
}

impl Blink {
    fn value(self) -> u8 {
        match self {
            Blink::Off => 0b0000_0000,
            Blink::HalfHz => 0b0000_0110,
            Blink::OneHz => 0b0000_0100,
            Blink::TwoHz => 0b0000_0010,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dimming {
    Brightness(u8),
    MinBrightness,
    /// Power-on default.
    MaxBrightness,
}

impl Dimming {
    // Can we implement From<u8> instead?
    // https://ricardomartins.cc/2016/08/03/convenient_and_idiomatic_conversions_in_rust
    pub fn from_u8(value: u8) -> Result<Self, Error> {
        if value > Dimming::MaxBrightness.value() {
            return Err(Error::OutOfRange(format!(
                "Dimming value [{}] is greater than max brightness [{}]",
                value,
                Dimming::MaxBrightness.value()
            )));
        }

        Ok(Dimming::Brightness(value))
    }

    pub fn value(self) -> u8 {
        match self {
            Dimming::Brightness(value) => value,
            Dimming::MinBrightness => 0b0000_0000,
            Dimming::MaxBrightness => 0b0000_1111,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Error {
    OutOfRange(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LedLocation {
    row: u8,
    common: u8,
}

impl LedLocation {
    #![allow(clippy::new_ret_no_self)]
    pub fn new(row: u8, common: u8) -> Result<Self, Error> {
        if row >= ROWS_SIZE as u8 {
            return Err(Error::OutOfRange(format!(
                "Row value [{}] is greater than rows size [{}]",
                row, ROWS_SIZE
            )));
        }

        if common >= COMMONS_SIZE as u8 {
            return Err(Error::OutOfRange(format!(
                "Common value [{}] is greater than commons size [{}]",
                common, COMMONS_SIZE
            )));
        }

        Ok(LedLocation { row, common })
    }
}

pub struct HT16K33<I2C> {
    i2c: I2C,

    // Device I2C address.
    address: u8,

    // Represents the desired values of the device, may not match
    // the current values if it has not been written recently.
    buffer: [u8; ROWS_SIZE],

    // The following values are write-only registers and cannot
    // be queried from the device. We need to track their state
    // here and synchronize them with the device.
    oscillator_state: Oscillator,
    display_state: Display,
    blink_state: Blink,
    dimming_state: Dimming,

    logger: slog::Logger,
}

impl<I2C, E> HT16K33<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Create an HT16K33 driver.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C device to communicate with the HT16K33 driver.
    /// * `logger` - A logging instance.
    ///
    /// # Notes
    ///
    /// `logger = None`, will log to the `slog-stdlog` drain. This makes the library
    /// effectively work the same as if it was just using `log` instead of `slog`.
    ///
    /// `Into` trick allows passing `Logger` directly, without the `Some` part.
    /// See http://xion.io/post/code/rust-optional-args.html
    ///
    /// # Examples
    ///
    /// ```
    /// // NOTE: `None` is used for the Logger in these examples for convenience,
    /// // in practice using an actual logger is preferred.
    ///
    /// extern crate ht16k33;
    /// use ht16k33::i2c_mock::I2cMock;
    /// use ht16k33::HT16K33;
    ///
    /// # fn main() {
    ///
    /// // Create an I2C device.
    /// let mut i2c = I2cMock::new(None);
    ///
    /// // The I2C device address.
    /// let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// # }
    /// ```
    pub fn new<L>(i2c: I2C, address: u8, logger: L) -> Self
    where
        L: Into<Option<slog::Logger>>,
    {
        let logger = logger
            .into()
            .unwrap_or_else(|| slog::Logger::root(slog_stdlog::StdLog.fuse(), o!()));

        trace!(logger, "Constructing HT16K33");

        // Configure the initial values to match the power-on defaults.
        HT16K33 {
            address,
            i2c,
            buffer: [0; ROWS_SIZE],
            oscillator_state: Oscillator::Off,
            display_state: Display::Off,
            blink_state: Blink::Off,
            dimming_state: Dimming::MaxBrightness,
            logger,
        }
    }

    /// Initialize the HT16K33.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    /// ht16k33.initialize().unwrap();
    ///
    /// # }
    /// ```
    pub fn initialize(&mut self) -> Result<(), E> {
        // Enable the oscillator so we can use the device.
        self.set_oscillator(Oscillator::On)?;

        // Set all values to match their defaults.
        self.set_display(Display::Off, Blink::Off)?;
        self.set_dimming(Dimming::MaxBrightness)?;

        // And clear the display.
        self.clear_display_buffer();
        self.write_display_buffer()?;

        Ok(())
    }

    /// Return the given I2C device, making this device unusable.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    /// i2c = ht16k33.destroy();
    ///
    /// # }
    /// ```
    pub fn destroy(self) -> I2C {
        // TODO Improve the naming? And somehow mark the state as destroyed so that
        // all other I2C accesses fail nicely? Should we continue to allow non-I2C access?
        self.i2c
    }

    /// Return the current display buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    /// let buffer = ht16k33.get_display_buffer();
    ///
    /// # }
    /// ```
    pub fn get_display_buffer(&self) -> &[u8; ROWS_SIZE] {
        trace!(self.logger, "get_display_buffer");

        &self.buffer
    }

    /// Return the current oscillator state.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    /// let oscillator = ht16k33.get_oscillator();
    ///
    /// # }
    /// ```
    pub fn get_oscillator(&self) -> &Oscillator {
        trace!(self.logger, "get_oscillator");

        &self.oscillator_state
    }

    /// Return the current display & blink state.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    /// let (display, blink) = ht16k33.get_display();
    ///
    /// # }
    /// ```
    pub fn get_display(&self) -> (&Display, &Blink) {
        trace!(self.logger, "get_display");

        (&self.display_state, &self.blink_state)
    }

    /// Return the current dimming state.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    /// let dimming = ht16k33.get_dimming();
    ///
    /// # }
    /// ```
    pub fn get_dimming(&self) -> &Dimming {
        trace!(self.logger, "get_dimming");

        &self.dimming_state
    }

    /// Enable/disable an LED address in the display buffer.
    ///
    /// The buffer must be written using [write_display_buffer()](struct.HT16K33.html#method.write_display_buffer)
    /// for the change to be displayed.
    ///
    /// # Arguments
    ///
    /// * `location` - The LED location to update.
    /// * `enabled` - Set the LED on (true) or off (false).
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::LedLocation;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// let led_location = LedLocation::new(0, 0).unwrap();
    /// ht16k33.update_display_buffer(led_location, true).unwrap();
    ///
    /// # }
    /// ```
    pub fn update_display_buffer(&mut self, location: LedLocation, enabled: bool) -> Result<(), E> {
        // TODO Validate `address` parameter.
        trace!(self.logger, "update_display_buffer"; "location" => format!("{:?}", location), "enabled" => enabled);

        if enabled {
            // Turn on the specified LED (set bit to one).
            self.buffer[location.row as usize] |= 1 << location.common;
        } else {
            // Turn off the specified LED (set bit to zero).
            self.buffer[location.row as usize] &= !(1 << location.common);
        }

        Ok(())
    }

    /// Clear contents of the display buffer.
    ///
    /// The buffer must be written using [write_display_buffer()](struct.HT16K33.html#method.write_display_buffer)
    /// for the change to be displayed.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// ht16k33.clear_display_buffer();
    ///
    /// # }
    /// ```
    pub fn clear_display_buffer(&mut self) {
        trace!(self.logger, "clear_display_buffer");

        // TODO is there any advantage to iteration vs just assigning
        // an empty [0; ROWS_SIZE] array?
        for row in self.buffer.iter_mut() {
            *row = 0;
        }
    }

    /// Control the oscillator.
    ///
    /// # Arguments
    ///
    /// * `oscillator` - Set the oscillator On/Off.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::Oscillator;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// ht16k33.set_oscillator(Oscillator::On).unwrap();
    ///
    /// # }
    /// ```
    pub fn set_oscillator(&mut self, oscillator: Oscillator) -> Result<(), E> {
        trace!(self.logger, "set_oscillator"; "oscillator" => format!("{:?}", oscillator));

        self.oscillator_state = oscillator;

        self.i2c
            .write(self.address, &[SYSTEM_SET | self.oscillator_state.value()])?;

        Ok(())
    }

    /// Control the display.
    ///
    /// # Arguments
    ///
    /// * `display` - Set the display On/Off.
    /// * `blink` - Set the blink On/Off/etc.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::Display;
    /// use ht16k33::Blink;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// ht16k33.set_display(Display::On, Blink::HalfHz).unwrap();
    ///
    /// # }
    /// ```
    pub fn set_display(&mut self, display: Display, blink: Blink) -> Result<(), E> {
        trace!(self.logger, "set_display";
        "display" => format!("{:?}", display),
        "blink" => format!("{:?}", blink),
        );

        self.display_state = display;
        self.blink_state = blink;

        self.i2c.write(
            self.address,
            &[DISPLAY_SET | self.display_state.value() | self.blink_state.value()],
        )?;

        Ok(())
    }

    /// Control the display dimming.
    ///
    /// # Arguments
    ///
    /// * `dimming` - A value from `0` (lowest) to `15` (highest).
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::Dimming;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// ht16k33.set_dimming(Dimming::from_u8(4).unwrap()).unwrap();
    ///
    /// # }
    /// ```
    pub fn set_dimming(&mut self, dimming: Dimming) -> Result<(), E> {
        trace!(self.logger, "set_dimming"; "dimming" => format!("{:?}", dimming));

        self.dimming_state = dimming;

        self.i2c
            .write(self.address, &[DIMMING_SET | self.dimming_state.value()])?;

        Ok(())
    }

    /// Control an LED.
    ///
    /// # Arguments
    ///
    /// * `location` - The LED location to update.
    /// * `enabled` - Set the LED on (true) or off (false).
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::LedLocation;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// let led_location = LedLocation::new(0, 0).unwrap();
    /// ht16k33.set_led(led_location, true).unwrap();
    ///
    /// # }
    /// ```
    pub fn set_led(&mut self, location: LedLocation, enabled: bool) -> Result<(), E> {
        // TODO Validate `address` parameter.
        trace!(self.logger, "set_led"; "location" => format!("{:?}", location), "enabled" => enabled);

        self.update_display_buffer(location, enabled)?;

        self.i2c.write(
            self.address,
            &[
                DATA_ADDRESS | location.row,
                self.buffer[location.row as usize],
            ],
        )?;

        Ok(())
    }

    /// Write the display buffer to the HT16K33 driver.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// ht16k33.write_display_buffer().unwrap();
    ///
    /// # }
    /// ```
    pub fn write_display_buffer(&mut self) -> Result<(), E> {
        trace!(self.logger, "write_display_buffer"; "buffer" => format!("{:?}", self.buffer));

        let mut write_buffer = [0u8; ROWS_SIZE + 1];
        write_buffer[0] = DATA_ADDRESS;

        for value in 0..self.buffer.len() {
            write_buffer[value as usize + 1] = self.buffer[value as usize];
        }

        self.i2c.write(self.address, &write_buffer)?;

        Ok(())
    }

    /// Read the display buffer from the HT16K33 driver.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address, None);
    ///
    /// ht16k33.read_display_buffer().unwrap();
    ///
    /// # }
    /// ```
    pub fn read_display_buffer(&mut self) -> Result<(), E> {
        trace!(self.logger, "read_display_buffer"; "buffer" => format!("{:?}", self.buffer));

        self.i2c
            .write_read(self.address, &[DATA_ADDRESS], &mut self.buffer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate embedded_hal_mock as hal;

    use self::hal::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use super::*;

    const ADDRESS: u8 = 0;

    #[test]
    fn new() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn initialize() {
        let mut write_buffer = vec![super::DATA_ADDRESS];
        write_buffer.extend([0; super::ROWS_SIZE].iter().cloned());

        let expectations = [
            I2cTransaction::write(
                ADDRESS,
                vec![super::SYSTEM_SET | super::Oscillator::On.value()],
            ),
            I2cTransaction::write(
                ADDRESS,
                vec![super::DISPLAY_SET | super::Display::Off.value() | super::Blink::Off.value()],
            ),
            I2cTransaction::write(
                ADDRESS,
                vec![super::DIMMING_SET | Dimming::MaxBrightness.value()],
            ),
            I2cTransaction::write(ADDRESS, write_buffer),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33.initialize().unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn get_display_buffer() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        let &buffer = ht16k33.get_display_buffer();

        // Ensure it's the expected size.
        assert_eq!(buffer.len(), ROWS_SIZE);

        for row in buffer.iter() {
            // And because we just initialized this buffer, it should be all zeros.
            assert_eq!(*row, 0u8);
        }

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn get_oscillator() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        let &oscillator = ht16k33.get_oscillator();

        assert_eq!(oscillator, Oscillator::Off);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn get_display() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        let (&display, &blink) = ht16k33.get_display();

        assert_eq!(display, Display::Off);
        assert_eq!(blink, Blink::Off);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn get_dimming() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        let &dimming = ht16k33.get_dimming();

        assert_eq!(dimming, Dimming::MaxBrightness);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn update_display_buffer() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        let first_led = LedLocation::new(1, 4).unwrap();
        let second_led = LedLocation::new(1, 5).unwrap();

        // Turn on the LED.
        ht16k33.update_display_buffer(first_led, true).unwrap();
        assert_eq!(ht16k33.get_display_buffer()[1], 0b0001_0000);

        // Turn on another LED.
        ht16k33.update_display_buffer(second_led, true).unwrap();
        assert_eq!(ht16k33.get_display_buffer()[1], 0b0011_0000);

        // Turn off the first LED.
        ht16k33.update_display_buffer(first_led, false).unwrap();
        assert_eq!(ht16k33.get_display_buffer()[1], 0b0010_0000);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn clear_display_buffer() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        // Clear the display buffer.
        ht16k33.clear_display_buffer();

        let &buffer = ht16k33.get_display_buffer();

        // Ensure it's still the expected size.
        assert_eq!(buffer.len(), ROWS_SIZE);

        for row in buffer.iter() {
            // We just cleared this buffer, it should be all zeros.
            assert_eq!(*row, 0u8);
        }

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_oscillator() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![super::SYSTEM_SET | super::Oscillator::Off.value()],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33.set_oscillator(super::Oscillator::Off).unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_display() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![super::DISPLAY_SET | super::Display::Off.value() | super::Blink::Off.value()],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33
            .set_display(super::Display::Off, super::Blink::Off)
            .unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_dimming() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![super::DIMMING_SET | Dimming::MaxBrightness.value()],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33.set_dimming(Dimming::MaxBrightness).unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_led() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![super::DATA_ADDRESS | 1u8, 0b1000_0000],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33
            .set_led(LedLocation { row: 1, common: 7 }, true)
            .unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn write_display_buffer() {
        let mut write_buffer = vec![super::DATA_ADDRESS];
        write_buffer.extend([0; super::ROWS_SIZE].iter().cloned());

        let expectations = [I2cTransaction::write(ADDRESS, write_buffer)];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33.write_display_buffer().unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn read_display_buffer() {
        let mut read_buffer = vec![0; super::ROWS_SIZE];
        read_buffer[1] = 1;
        read_buffer[15] = 1;

        let expectations = [I2cTransaction::write_read(
            ADDRESS,
            vec![super::DATA_ADDRESS],
            read_buffer,
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS, None);

        ht16k33.read_display_buffer().unwrap();

        let &buffer = ht16k33.get_display_buffer();

        for value in 0..buffer.len() {
            match value {
                1 | 15 => assert_eq!(buffer[value], 1),
                _ => assert_eq!(buffer[value], 0),
            }
        }

        i2c = ht16k33.destroy();
        i2c.done();
    }
}
