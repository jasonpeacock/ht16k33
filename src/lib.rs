//! # HT16K33
//!
//! `ht16k33` is a driver for the [Holtek HT16K33 RAM Mapping 16\*8 LED Controller Driver with keyscan](http://www.holtek.com/productdetail/-/vg/HT16K33) chip.
//!
//! Currently, only the 28-pin SOP package type is supported.
//!
//! # Features
//!
//! - [x] Uses the [`embedded-hal`](https://crates.io/crates/embedded-hal) hardware abstraction.
//! - [x] Supports `no_std` for embedded devices.
//! - [ ] Supports all 20/24/28-pin SOP package types.
//! - [x] Displays all 128 LEDs.
//! - [ ] Reads keyscan.
//! - [ ] Manages interrupts.
//! - [ ] Manages slave devices.
//!
//! # Usage
//!
//! ## Linux-based platforms
//!
//! Using the recommended [`linux-embedded-hal`](https://crates.io/crates/linux-embedded-hal)
//! crate which implements the `embedded-hal` traits for Linux devices, including I2C.
//!
//! ```!ignore
//! # use failure::Error;
//! use linux_embedded_hal::I2cdev;
//! use ht16k33::HT16K33;
//! # fn main() -> Result<(), Error>{
//!
//! // The I2C device address.
//! let address = 112u8;
//!
//! // Create an I2C device.
//! let mut i2c = I2cdev::new("/path/to/i2c/device")?;
//! i2c.set_slave_address(address as u16)?;
//!
//! let mut ht16k33 = HT16K33::new(i2c, address);
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Embedded platforms
//!
//! Using this crate without default features in your `Cargo.toml` like so:
//! ```toml
//! [dependencies]
//! htk16k33 = { version = "*", default-features = false }
//! ```
//!
//! ## All platforms, using I2C simulation
//!
//! Not all platforms have I2C support. The provided `ht16k33::i2c_mock` implements the
//! `embedded-hal` traits for I2C.
//!
//! ```
//! # use failure::Error;
//! use ht16k33::i2c_mock::I2cMock;
//! use ht16k33::HT16K33;
//! # fn main() {
//!
//! // The I2C device address.
//! let address = 112u8;
//!
//! // Create a mock I2C device.
//! let mut i2c = I2cMock::new();
//!
//! let mut ht16k33 = HT16K33::new(i2c, address);
//!
//! # }
//! ```
#![cfg_attr(not(feature = "std"), no_std)]
#![doc(html_root_url = "https://docs.rs/ht16k33/0.4.0")]
#![deny(missing_docs)]
use embedded_hal as hal;

#[cfg(feature = "serde")]
use serde;

mod constants;
mod errors;
mod types;

pub mod i2c_mock;

pub use errors::ValidationError;
pub use types::{Dimming, Display, DisplayData, DisplayDataAddress, LedLocation, Oscillator};

pub use constants::{COMMONS_SIZE, ROWS_SIZE};
use hal::blocking::i2c::{Write, WriteRead};

/// The HT16K33 state and configuration.
pub struct HT16K33<I2C> {
    i2c: I2C,

    // Device I2C address.
    address: u8,

    // Represents the desired values of the device, may not match
    // the current values if it has not been written recently.
    buffer: [DisplayData; ROWS_SIZE],

    // The following values are write-only registers and cannot
    // be queried from the device. We need to track their state
    // here and synchronize them with the device.
    oscillator_state: Oscillator,
    display_state: Display,
    dimming_state: Dimming,
}

impl<I2C, E> HT16K33<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Create an HT16K33 driver.
    ///
    /// # Arguments
    ///
    /// * `i2c` - The I2C device to communicate with the HT16K33 chip.
    ///
    /// # Examples
    ///
    /// ```
    /// use ht16k33::i2c_mock::I2cMock;
    /// use ht16k33::HT16K33;
    /// # fn main() {
    ///
    /// // Create an I2C device.
    /// let mut i2c = I2cMock::new();
    ///
    /// // The I2C device address.
    /// let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    ///
    /// # }
    /// ```
    pub fn new(i2c: I2C, address: u8) -> Self {
        // Configure the initial values to match the power-on defaults.
        HT16K33 {
            address,
            i2c,
            buffer: [DisplayData::empty(); ROWS_SIZE],
            oscillator_state: Oscillator::OFF,
            display_state: Display::OFF,
            dimming_state: Dimming::BRIGHTNESS_MAX,
        }
    }

    /// Initialize the HT16K33.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() -> Result<(), Error> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.initialize()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn initialize(&mut self) -> Result<(), E> {
        // Enable the oscillator so we can use the device.
        self.set_oscillator(Oscillator::ON)?;

        // Set all values to match their defaults.
        self.set_display(Display::OFF)?;
        self.set_dimming(Dimming::BRIGHTNESS_MAX)?;

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
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
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
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// let &buffer = ht16k33.display_buffer();
    ///
    /// # }
    /// ```
    pub fn display_buffer(&self) -> &[DisplayData; ROWS_SIZE] {
        &self.buffer
    }

    /// Return the current oscillator state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// let oscillator = ht16k33.oscillator();
    ///
    /// # }
    /// ```
    pub fn oscillator(&self) -> &Oscillator {
        &self.oscillator_state
    }

    /// Return the current display state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// let display = ht16k33.display();
    ///
    /// # }
    /// ```
    pub fn display(&self) -> &Display {
        &self.display_state
    }

    /// Return the current dimming state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// let dimming = ht16k33.dimming();
    ///
    /// # }
    /// ```
    pub fn dimming(&self) -> &Dimming {
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
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::ValidationError;
    /// use ht16k33::LedLocation;
    /// # fn main() -> Result<(), ValidationError> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    ///
    /// let led_location = LedLocation::new(0, 0)?;
    /// ht16k33.update_display_buffer(led_location, true);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_display_buffer(&mut self, location: LedLocation, enabled: bool) {
        // TODO Validate `address` parameter.

        // Turn on/off the specified LED.
        self.buffer[location.common_as_index()].set(location.row, enabled);
    }

    /// Clear contents of the display buffer.
    ///
    /// The buffer must be written using [write_display_buffer()](struct.HT16K33.html#method.write_display_buffer)
    /// for the change to be displayed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.clear_display_buffer();
    ///
    /// # }
    /// ```
    pub fn clear_display_buffer(&mut self) {
        // TODO is there any advantage to iteration vs just assigning
        // a new, empty `[0; COMMON_SIZE]` array?
        for row in self.buffer.iter_mut() {
            *row = DisplayData::ROW_NONE;
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
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::Oscillator;
    /// # fn main() -> Result<(), Error> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.set_oscillator(Oscillator::ON)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_oscillator(&mut self, oscillator: Oscillator) -> Result<(), E> {
        self.oscillator_state = oscillator;

        self.i2c.write(
            self.address,
            &[(Oscillator::COMMAND | self.oscillator_state).bits()],
        )?;

        Ok(())
    }

    /// Control the display.
    ///
    /// # Arguments
    ///
    /// * `display` - Set the display On/Off.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::Display;
    /// # fn main() -> Result<(), Error> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.set_display(Display::HALF_HZ)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_display(&mut self, display: Display) -> Result<(), E> {
        self.display_state = display;

        self.i2c.write(
            self.address,
            &[(Display::COMMAND | self.display_state).bits()],
        )?;

        Ok(())
    }

    /// Control the display dimming.
    ///
    /// # Arguments
    ///
    /// * `dimming` - Set the dimming brightness.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::Dimming;
    /// # fn main() -> Result<(), Error> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.set_dimming(Dimming::from_u8(4)?)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_dimming(&mut self, dimming: Dimming) -> Result<(), E> {
        self.dimming_state = dimming;

        self.i2c.write(
            self.address,
            &[(Dimming::COMMAND | self.dimming_state).bits()],
        )?;

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
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// use ht16k33::LedLocation;
    /// # fn main() -> Result<(), Error> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    ///
    /// let led_location = LedLocation::new(0, 0)?;
    /// ht16k33.set_led(led_location, true)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_led(&mut self, location: LedLocation, enabled: bool) -> Result<(), E> {
        // TODO Validate `address` parameter.
        self.update_display_buffer(location, enabled);

        let buf_addr = location.common_as_index();
        let chip_addr = location.common_as_index_on_chip();

        let new_mask = if chip_addr % 2 != 0 {
            u16::to_le_bytes(self.buffer[buf_addr].bits())[1]
        } else {
            u16::to_le_bytes(self.buffer[buf_addr].bits())[0]
        };

        self.i2c.write(
            self.address,
            &[
                chip_addr,
                new_mask
            ],
        )?;

        Ok(())
    }

    /// Write the display buffer to the HT16K33 chip.
    ///
    /// # Examples
    ///
    /// ```
    /// # use failure::Error;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # fn main() -> Result<(), Box<Error>> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.write_display_buffer();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_display_buffer(&mut self) -> Result<(), E> {
        let mut write_buffer = [0u8; ROWS_SIZE + 1];
        write_buffer[0] = DisplayDataAddress::COMMON_0.bits();

        for (write_idx, ddata) in Iterator::zip((1..17).step_by(2), self.buffer) {
            let bytes = u16::to_le_bytes(ddata.bits());
            write_buffer[write_idx] = bytes[0];
            write_buffer[write_idx + 1] = bytes[1];
        }

        self.i2c.write(self.address, &write_buffer)?;

        Ok(())
    }

    /// Read the display buffer from the HT16K33 chip.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use ht16k33::HT16K33;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<Error>> {
    /// # let mut i2c = I2cMock::new();
    /// # let address = 0u8;
    ///
    /// let mut ht16k33 = HT16K33::new(i2c, address);
    /// ht16k33.read_display_buffer();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_display_buffer(&mut self) -> Result<(), E> {
        let mut read_buffer = [0u8; ROWS_SIZE];

        self.i2c.write_read(
            self.address,
            &[DisplayDataAddress::COMMON_0.bits()],
            &mut read_buffer,
        )?;

        let mut bytes = [0; 2];
        for (index, value) in read_buffer.iter().enumerate() {
            if index % 2 != 0 {
                bytes[1] = *value;
                self.buffer[index/2] = DisplayData::from_bits_truncate(u16::from_le_bytes(bytes));
            } else {
                bytes[0] = *value;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use embedded_hal_mock as hal;

    use self::hal::i2c::{Mock as I2cMock, Transaction as I2cTransaction};
    use super::*;

    use std::vec;

    const ADDRESS: u8 = 0;

    #[test]
    fn new() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn initialize() {
        let mut write_buffer = vec![super::DisplayDataAddress::ROW_0.bits()];
        write_buffer.extend([0; super::ROWS_SIZE].iter().cloned());

        let expectations = [
            I2cTransaction::write(
                ADDRESS,
                vec![(super::Oscillator::COMMAND | super::Oscillator::ON).bits()],
            ),
            I2cTransaction::write(
                ADDRESS,
                vec![(super::Display::COMMAND | super::Display::OFF).bits()],
            ),
            I2cTransaction::write(
                ADDRESS,
                vec![(super::Dimming::COMMAND | Dimming::BRIGHTNESS_MAX).bits()],
            ),
            I2cTransaction::write(ADDRESS, write_buffer),
        ];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33.initialize().unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn display_buffer() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS);

        let &buffer = ht16k33.display_buffer();

        // Ensure it's the expected size.
        assert_eq!(buffer.len(), ROWS_SIZE);

        for row in buffer.iter() {
            // And because we just initialized this buffer, it should be all zeros.
            assert_eq!(row.bits(), 0u8);
        }

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn oscillator() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS);

        let &oscillator = ht16k33.oscillator();

        assert_eq!(oscillator, Oscillator::OFF);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn display() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS);

        let &display = ht16k33.display();

        assert_eq!(display, Display::OFF);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn dimming() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let ht16k33 = HT16K33::new(i2c, ADDRESS);

        let &dimming = ht16k33.dimming();

        assert_eq!(dimming, Dimming::BRIGHTNESS_MAX);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn update_display_buffer() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        let first_led = LedLocation::new(1, 4).unwrap();
        let second_led = LedLocation::new(1, 5).unwrap();

        // Turn on the LED.
        ht16k33.update_display_buffer(first_led, true);
        assert_eq!(ht16k33.display_buffer()[1].bits(), 0b0001_0000);

        // Turn on another LED.
        ht16k33.update_display_buffer(second_led, true);
        assert_eq!(ht16k33.display_buffer()[1].bits(), 0b0011_0000);

        // Turn off the first LED.
        ht16k33.update_display_buffer(first_led, false);
        assert_eq!(ht16k33.display_buffer()[1].bits(), 0b0010_0000);

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn clear_display_buffer() {
        let expectations = [];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        let first_led = LedLocation::new(1, 4).unwrap();
        let second_led = LedLocation::new(1, 5).unwrap();

        // Turn on the LEDs.
        ht16k33.update_display_buffer(first_led, true);
        ht16k33.update_display_buffer(second_led, true);

        // Clear the display buffer.
        ht16k33.clear_display_buffer();

        let &buffer = ht16k33.display_buffer();

        // Ensure it's still the expected size.
        assert_eq!(buffer.len(), ROWS_SIZE);

        for row in buffer.iter() {
            // We just cleared this buffer, it should be all zeros.
            assert_eq!(row.bits(), 0u8);
        }

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_oscillator() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![(super::Oscillator::COMMAND | super::Oscillator::OFF).bits()],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33.set_oscillator(super::Oscillator::OFF).unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_display() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![(super::Display::COMMAND | super::Display::OFF).bits()],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33.set_display(super::Display::OFF).unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_dimming() {
        let expectations = [I2cTransaction::write(
            ADDRESS,
            vec![(super::Dimming::COMMAND | Dimming::BRIGHTNESS_MAX).bits()],
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33.set_dimming(Dimming::BRIGHTNESS_MAX).unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn set_led() {
        let expectations = [I2cTransaction::write(ADDRESS, vec![1u8, 0b1000_0000])];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33
            .set_led(LedLocation::new(1, 7).unwrap(), true)
            .unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn write_display_buffer() {
        let mut write_buffer = vec![super::DisplayDataAddress::ROW_0.bits()];
        write_buffer.extend([0; super::ROWS_SIZE].iter().cloned());

        let expectations = [I2cTransaction::write(ADDRESS, write_buffer)];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33.write_display_buffer().unwrap();

        i2c = ht16k33.destroy();
        i2c.done();
    }

    #[test]
    fn read_display_buffer() {
        let mut read_buffer = vec![0; super::ROWS_SIZE];
        read_buffer[1] = 0b0000_0010;
        read_buffer[15] = 0b0000_0010;

        let expectations = [I2cTransaction::write_read(
            ADDRESS,
            vec![super::DisplayDataAddress::ROW_0.bits()],
            read_buffer,
        )];

        let mut i2c = I2cMock::new(&expectations);
        let mut ht16k33 = HT16K33::new(i2c, ADDRESS);

        ht16k33.read_display_buffer().unwrap();

        let &buffer = ht16k33.display_buffer();

        for value in 0..buffer.len() {
            match value {
                1 | 15 => assert_eq!(buffer[value].bits(), 0b0000_0010),
                _ => assert_eq!(buffer[value].bits(), 0),
            }
        }

        i2c = ht16k33.destroy();
        i2c.done();
    }
}
