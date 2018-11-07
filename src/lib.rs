//! # HT16K33
//!
//! A driver for the [Holtek HT16K33](http://www.holtek.com/productdetail/-/vg/HT16K33) memory mapping and multi-function LED controller driver, as used by the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).
//!
//! The implementation was inspired by [Adafruit's HT16K33.py Python
//! implementation](https://github.com/adafruit/Adafruit_Python_LED_Backpack/blob/master/Adafruit_LED_Backpack/HT16K33.py).

extern crate i2cdev;
extern crate num_integer;

#[macro_use]
extern crate slog;
extern crate slog_stdlog;

use std::error;
use std::fmt;

use i2cdev::core::I2CDevice;

use num_integer::Integer;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

pub mod i2c_mock;

pub enum HT16K33Error<D>
where
    D: I2CDevice,
{
    /// Error from the `I2C` interface.
    Device(D::Error),
    /// Error from `HT16K33`.
    Error,
}

impl<D> fmt::Debug for HT16K33Error<D>
where
    D: I2CDevice,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HT16K33Error: {:?}", self)
    }
}

impl<D> fmt::Display for HT16K33Error<D>
where
    D: I2CDevice,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HT16K33Error::Device(ref err) => write!(f, "I2CDevice Error: {}", err),
            HT16K33Error::Error => write!(f, "HT16K33 Error"),
        }
    }
}

impl<D> error::Error for HT16K33Error<D>
where
    D: I2CDevice,
{
    fn description(&self) -> &str {
        match *self {
            HT16K33Error::Device(_) => "I2CDevice Error",
            HT16K33Error::Error => "HT16K33 Error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HT16K33Error::Device(ref err) => Some(err),
            HT16K33Error::Error => None,
        }
    }
}

pub struct HT16K33<D>
where
    D: I2CDevice,
{
    buffer: [u8; 16],
    i2c_device: D,
    is_ready: bool,
    logger: Logger,
    steps: u8,
}

// System initialization values.
const SYSTEM_SETUP: u8 = 0x20;
const OSCILLATOR: u8 = 0x01;

// Display brightness.
const BRIGHTNESS_CMD: u8 = 0xE0;

// Blink values.
const BLINK_CMD: u8 = 0x80;
const BLINK_DISPLAYON: u8 = 0x01;

// TODO use an enum for these values.
/// Disable blinking of the display.
pub const BLINK_OFF: u8 = 0x00;
/// Blink the display at 2Hz.
pub const BLINK_2HZ: u8 = 0x02;
/// Blink the display at 1Hz.
pub const BLINK_1HZ: u8 = 0x04;
/// Blink the display at 0.5Hz.
pub const BLINK_HALFHZ: u8 = 0x06;

// A bitmask value where the first bit is Green, and the second bit is
// Red. If both bits are set then the color is Yellow (Red + Green light).
// TODO use an enum for these values.
/// Turn off both the Red & Green LEDs.
pub const COLOR_OFF: u8 = 0;
/// Turn on only the Green LED.
pub const COLOR_GREEN: u8 = 1;
/// Turn on only the Red LED.
pub const COLOR_RED: u8 = 2;
/// Turn on both the Red  & Green LEDs.
pub const COLOR_YELLOW: u8 = 3;

impl<D> HT16K33<D>
where
    D: I2CDevice,
{
    /// Create an HT16K33 driver.
    ///
    /// # Arguments
    ///
    /// * `i2c_device` - The I2C device to communicate with the HT16K33 driver.
    /// * `steps` - The resolution of the display.
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
    /// // Create a mock I2C device.
    /// use ht16k33::i2c_mock::MockI2CDevice;
    /// let i2c_device = MockI2CDevice::new(None);
    ///
    /// // Create a connected display with a resolution of 24 steps.
    /// use ht16k33::HT16K33;
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ```
    pub fn new<L>(i2c_device: D, steps: u8, logger: L) -> Result<HT16K33<D>, HT16K33Error<D>>
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing HT16K33 driver"; "steps" => steps);

        let ht16k33 = HT16K33 {
            buffer: [0; 16],
            i2c_device: i2c_device,
            is_ready: false,
            logger: logger,
            steps: steps,
        };

        Ok(ht16k33)
    }

    /// Initialize the HT16K33 chip.
    ///
    /// Sets the initial state:
    ///
    /// * System setup.
    /// * Enable clock oscillator.
    /// * Disable blinking.
    /// * Set maximum brightness.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    ///
    /// // Initialize the HT16K33.
    /// ht16k33.initialize();
    /// ```
    pub fn initialize(&mut self) -> Result<(), HT16K33Error<D>> {
        debug!(self.logger, "Initializing HT16K33");

        debug!(
            self.logger,
            "Setting up the system & enabling the oscillator"
        );
        self.i2c_device
            .smbus_write_block_data(SYSTEM_SETUP | OSCILLATOR, &[0; 0])
            .map_err(HT16K33Error::Device)?;

        // All initializations finished, ready to use.
        self.is_ready = true;

        Ok(())
    }

    /// Get the resolution (number of steps) of the display.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// let steps = ht16k33.get_resolution();
    /// ```
    pub fn get_resolution(&mut self) -> u8 {
        self.steps
    }

    /// Check if the HT16K33 driver is ready to be used.
    ///
    /// The HT16K33 driver must be initialized to be ready to be used.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    ///
    /// // Not ready to use yet.
    /// assert_eq!(false, ht16k33.is_ready());
    ///
    /// // Initialize the HT16K33.
    /// ht16k33.initialize();
    ///
    /// // Ready to use.
    /// assert_eq!(true, ht16k33.is_ready());
    /// ```
    pub fn is_ready(&mut self) -> bool {
        self.is_ready
    }

    /// Enable/disabling blinking the display.
    ///
    /// # Arguments
    ///
    /// * `frequency` - A valid frequency value.
    ///
    /// # Notes
    ///
    /// The frequency must be one of the following values:
    ///
    /// * [BLINK_OFF](constant.BLINK_OFF.html)
    /// * [BLINK_HALFHZ](constant.BLINK_HALFHZ.html)
    /// * [BLINK_1HZ](constant.BLINK_1HZ.html)
    /// * [BLINK_2HZ](constant.BLINK_2Hz.html)
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Blink the display quickly.
    /// ht16k33.set_blink(ht16k33::BLINK_2HZ);
    /// ```
    pub fn set_blink(&mut self, frequency: u8) -> Result<(), HT16K33Error<D>> {
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        // TODO Validate `frequency` parameter.
        self.i2c_device
            .smbus_write_block_data(BLINK_CMD | BLINK_DISPLAYON | frequency, &[0; 0])
            .map_err(HT16K33Error::Device)?;

        Ok(())
    }

    /// Set the display brightness.
    ///
    /// # Arguments
    ///
    /// * `brightness` - A value from `0` (lowest) to `15` (highest).
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Set the display to maximum brightness.
    /// ht16k33.set_brightness(15u8);
    /// ```
    pub fn set_brightness(&mut self, brightness: u8) -> Result<(), HT16K33Error<D>> {
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        // TODO Validate `brightness` parameter.
        self.i2c_device
            .smbus_write_block_data(BRIGHTNESS_CMD | brightness, &[0; 0])
            .map_err(HT16K33Error::Device)?;

        Ok(())
    }

    /// Write the display buffer to the HT16K33 driver.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Write the current buffer contents to the HT16K33 driver.
    /// ht16k33.write_display();
    /// ```
    pub fn write_display(&mut self) -> Result<(), HT16K33Error<D>> {
        // TODO rename to be "update_display" or "write_buffer"
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        for value in 0..self.buffer.len() {
            self.i2c_device
                .smbus_write_byte_data(value as u8, self.buffer[value])
                .map_err(HT16K33Error::Device)?;
        }

        Ok(())
    }

    /// Read the display buffer from the HT16K33 driver.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Write the current buffer contents to the HT16K33 driver.
    /// let values = ht16k33.read_display();
    /// ```
    pub fn read_display(&mut self) -> Result<(), HT16K33Error<D>> {
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        for value in 0..self.buffer.len() {
            self.i2c_device
                .smbus_write_byte_data(value as u8, self.buffer[value])
                .map_err(HT16K33Error::Device)?;
        }

        Ok(())
    }

    /// Enable/disable an LED in the display buffer.
    ///
    /// The buffer must be written using [write_display()](struct.HT16K33.html#method.write_display)
    /// for the change to be displayed.
    ///
    /// # Arguments
    ///
    /// * `led` - An LED address from `0` to `127`.
    /// * `enabled` - Turn the LED on (true) or off (false).
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Turn on the LED at address 0.
    /// ht16k33.set_led(0u8, true);
    ///
    /// // Write the current buffer contents to the HT16K33 driver.
    /// ht16k33.write_display();
    /// ```
    pub fn set_led(&mut self, led: u8, enabled: bool) -> Result<(), HT16K33Error<D>> {
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        // TODO Validate `led` parameter.

        // Calculate position in byte buffer and get offset of desired LED.
        let (pos, offset) = led.div_mod_floor(&8);

        if enabled {
            // Turn on the specified LED (set bit to one).
            self.buffer[pos as usize] |= 1 << offset;
        } else {
            // Turn off the specified LED (set bit to zero).
            self.buffer[pos as usize] &= !(1 << offset);
        }

        Ok(())
    }

    /// Clear contents of display buffer.
    ///
    /// The buffer must be written using [write_display()](struct.HT16K33.html#method.write_display)
    /// for the change to be displayed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Clear the display buffer.
    /// ht16k33.clear();
    ///
    /// // Write the current buffer contents to the HT16K33 driver.
    /// ht16k33.write_display();
    /// ```
    pub fn clear(&mut self) -> Result<(), HT16K33Error<D>> {
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        self.buffer = [0; 16];

        Ok(())
    }

    /// Set bar to desired color. Bar should be a value of 0 to 23, and color should be
    /// OFF, GREEN, RED, or YELLOW.
    ///
    /// The buffer must be written using [write_display()](struct.HT16K33.html#method.write_display)
    /// for the change to be displayed.
    ///
    /// # Arguments
    ///
    /// * `bar` - A value from `0` to `23`.
    /// * `color` - A valid color value.
    ///
    /// # Notes
    ///
    /// The color must be one of the following values:
    ///
    /// * [COLOR_OFF](constant.COLOR_OFF.html)
    /// * [COLOR_GREEN](constant.COLOR_GREEN.html)
    /// * [COLOR_RED](constant.COLOR_RED.html)
    /// * [COLOR_YELLOW](constant.COLOR_YELLOW.html)
    ///
    /// # Examples
    ///
    /// ```
    /// # use ht16k33::HT16K33;
    /// # use ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// #
    /// // Create an HT16K33 driver.
    /// let mut ht16k33 = HT16K33::new(i2c_device, 24, None).unwrap();
    /// ht16k33.initialize();
    ///
    /// // Set the first bar to be Yellow.
    /// ht16k33.set_bar(0u8, ht16k33::COLOR_YELLOW);
    ///
    /// // Write the current buffer contents to the HT16K33 driver.
    /// ht16k33.write_display();
    /// ```
    pub fn set_bar(&mut self, bar: u8, color: u8) -> Result<(), HT16K33Error<D>> {
        // TODO use Option to return only errors for these void functions
        if !self.is_ready() {
            return Err(HT16K33Error::Error);
        }

        // TODO Validate `bar` parameter.
        // TODO Validate `color` parameter.
        // Compute cathode and anode values.
        let (c, mut a) = (if bar < 12 { bar } else { bar - 12 }).div_mod_floor(&4);
        if bar >= 12 {
            a += 4;
        }

        // Set green LED based on 1st bit in color.
        self.set_led(
            c * 16 + a + 8,
            if color & COLOR_GREEN > 0 { true } else { false },
        )?;

        // Set red LED based on 2nd bit in color.
        self.set_led(c * 16 + a, if color & COLOR_RED > 0 { true } else { false })?;

        Ok(())
    }
}
