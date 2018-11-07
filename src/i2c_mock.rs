//! # i2c_mock
//!
//! A mock I2C library to support using the [HT16K33](../struct.HT16K33.html) driver on non-Linux systems that do
//! not have I2CDevice support.
//!
//! Original implementation: https://github.com/rust-embedded/rust-i2cdev/blob/master/src/mock.rs
use std::error;
use std::fmt;
use std::result;

use i2cdev::core::I2CDevice;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

pub type I2CResult<T> = result::Result<T, MockI2CDeviceError>;

pub struct I2CRegisterMap {
    registers: [u8; 0xFF],
    offset: usize,
    logger: Logger,
}

impl I2CRegisterMap {
    /// Create an I2CRegisterMap.
    ///
    /// # Arguments
    ///
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
    /// // Create an I2CRegisterMap.
    /// use ht16k33::i2c_mock::I2CRegisterMap;
    /// let mut i2c_register_map = I2CRegisterMap::new(None);
    /// ```
    pub fn new<L>(logger: L) -> I2CRegisterMap
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        trace!(logger, "Constructing I2CRegisterMap");

        I2CRegisterMap {
            registers: [0x00; 0xFF],
            offset: 0,
            logger: logger,
        }
    }

    /// Read data from the registers to fill the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `data` - Buffer to receive data from the registers.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create an I2CRegisterMap.
    /// use ht16k33::i2c_mock::I2CRegisterMap;
    /// let mut i2c_register_map = I2CRegisterMap::new(None);
    ///
    /// // Read data.
    /// let mut buffer = [0u8; 5];
    /// i2c_register_map.read(&mut buffer);
    /// ```
    pub fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        for i in 0..data.len() {
            data[i] = self.registers[self.offset];
            self.offset += 1;
        }
        trace!(self.logger, "READ";
               "register" => format!("0x{:X}", self.offset - data.len()),
               "data" => format!("{:?}", data));
        Ok(())
    }

    /// Write the provided buffer to the registers.
    ///
    /// # Arguments
    ///
    /// * `data` - Buffer to write data to the registers.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create an I2CRegisterMap.
    /// use ht16k33::i2c_mock::I2CRegisterMap;
    /// let mut i2c_register_map = I2CRegisterMap::new(None);
    ///
    /// // Write data.
    /// let buffer = [0u8; 5];
    /// i2c_register_map.write(&buffer);
    /// ```
    pub fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        // TODO validate assumptions
        // ASSUMPTION: first byte sets the offset
        // ASSUMPTION: write has length of at least 1 (will panic)
        let offset = data[0] as usize;
        let remdata = &data[1..];
        self.write_registers(offset, remdata);
        self.offset = offset + remdata.len();
        Ok(())
    }

    /// Write data to registers.
    ///
    /// # Arguments
    ///
    /// * `offset` - ???
    /// * `data` - ???
    ///
    /// # Examples
    ///
    /// ```
    /// // Create an I2CRegisterMap.
    /// use ht16k33::i2c_mock::I2CRegisterMap;
    /// let mut i2c_register_map = I2CRegisterMap::new(None);
    ///
    /// // Write registers.
    /// let offset = 0usize;
    /// let data = [0u8; 5];
    /// i2c_register_map.write_registers(offset, &data);
    /// ```
    pub fn write_registers(&mut self, offset: usize, data: &[u8]) {
        trace!(self.logger, "WRITE";
               "register" => format!("0x{:X}", offset),
               "data" => format!("{:?}", data));
        for i in 0..data.len() {
            self.registers[offset + i] = data[i];
        }
    }
}

pub struct MockI2CDevice {
    /// Backing datastore for mock I2C device; public access is provided for testing.
    pub regmap: I2CRegisterMap,
    logger: Logger,
}

#[derive(Debug)]
pub struct MockI2CDeviceError;

impl fmt::Display for MockI2CDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MockI2CDeviceError!")
    }
}

impl error::Error for MockI2CDeviceError {
    fn description(&self) -> &str {
        "MockI2CDeviceError!"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl MockI2CDevice {
    /// Create a MockI2CDevice.
    ///
    /// # Arguments
    ///
    /// * `logger` - A logging instance.
    ///
    /// # Notes
    ///
    /// `logger = None`, will log to the `slog-stdlog` drain. This makes the library
    /// effectively work the same as if it was just using `log` intead of `slog`.
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
    /// // Create a MockI2CDevice.
    /// use ht16k33::i2c_mock::MockI2CDevice;
    /// let mut i2c_device = MockI2CDevice::new(None);
    /// ```
    pub fn new<L>(logger: L) -> MockI2CDevice
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing MockI2CDevice");

        let regmap_logger = logger.new(o!("mod" => "HT16K33::i2c_mock::I2CRegisterMap"));

        MockI2CDevice {
            regmap: I2CRegisterMap::new(regmap_logger),
            logger: logger,
        }
    }
}

impl I2CDevice for MockI2CDevice {
    type Error = MockI2CDeviceError;

    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        debug!(self.logger, "read");
        self.regmap.read(data)
    }

    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        debug!(self.logger, "write";
               "data" => format!("{:?}", data));
        self.regmap.write(data)
    }

    fn smbus_write_quick(&mut self, _bit: bool) -> I2CResult<()> {
        debug!(self.logger, "smbus_write_quick";
               "bit" => _bit);
        Ok(())
    }

    fn smbus_read_block_data(&mut self, _register: u8) -> I2CResult<Vec<u8>> {
        debug!(self.logger, "smbus_read_block_data";
               "register" => format!("0x{:X}", _register));
        Ok(Vec::new())
    }

    fn smbus_read_i2c_block_data(&mut self, _register: u8, _len: u8) -> I2CResult<Vec<u8>> {
        debug!(self.logger, "smbus_read_i2c_block_data";
               "register" => format!("0x{:X}", _register),
               "length" => _len);
        Ok(Vec::new())
    }

    fn smbus_write_block_data(&mut self, _register: u8, _values: &[u8]) -> I2CResult<()> {
        debug!(self.logger, "smbus_write_block_data";
               "register" => format!("0x{:X}", _register),
               "values" => format!("{:?}", _values));
        Ok(())
    }

    fn smbus_write_i2c_block_data(&mut self, _register: u8, _values: &[u8]) -> I2CResult<()> {
        debug!(self.logger, "smbus_write_i2c_block_data";
               "register" => format!("0x{:X}", _register),
               "values" => format!("{:?}", _values));
        Ok(())
    }

    fn smbus_process_block(&mut self, _register: u8, _values: &[u8]) -> I2CResult<Vec<u8>> {
        debug!(self.logger, "smbus_process_block";
               "register" => format!("0x{:X}", _register),
               "values" => format!("{:?}", _values));
        Ok(Vec::new())
    }
}
