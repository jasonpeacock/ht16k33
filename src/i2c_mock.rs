//! # i2c_mock
//!
//! A mock I2C library to support using the [HT16K33](../struct.HT16K33.html) driver on non-Linux systems that do
//! not have I2C support.
extern crate embedded_hal as hal;

use std::fmt;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

use constants::ROWS_SIZE;
use types::DisplayDataAddress;

/// Mock error to satisfy the I2C trait.
#[derive(Debug)]
pub struct I2cMockError;

impl fmt::Display for I2cMockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "I2c MockError")
    }
}

/// The mock I2C state.
///
/// # Example
///
/// ```
/// // NOTE: `None` is used for the Logger in these examples for convenience,
/// // in practice using an actual logger is preferred.
///
/// extern crate ht16k33;
/// use ht16k33::i2c_mock::I2cMock;
/// # fn main() {
///
/// // Create an I2cMock.
/// let i2c_mock = I2cMock::new(None);
///
/// # }
/// ```
pub struct I2cMock {
    /// Display RAM state.
    pub data_values: [u8; ROWS_SIZE],
    logger: Logger,
}

impl I2cMock {
    /// Create an I2cMock.
    ///
    /// # Arguments
    ///
    /// * `logger` - A logging instance.
    ///
    /// # Notes
    ///
    /// `logger = None`, will log to the `slog-stdlog` drain. This makes the library
    /// effectively work the same as if it was just using `log` instead of `slog`.
    pub fn new<L>(logger: L) -> Self
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger
            .into()
            .unwrap_or_else(|| Logger::root(StdLog.fuse(), o!()));

        trace!(logger, "Constructing I2cMock");

        I2cMock {
            data_values: [0; ROWS_SIZE],
            logger,
        }
    }
}

impl hal::blocking::i2c::WriteRead for I2cMock {
    type Error = I2cMockError;

    /// `write_read` implementation.
    ///
    /// # Arguments
    ///
    /// * `_address` - The slave address. Ignored.
    /// * `bytes` - The command/address instructions to be written.
    /// * `buffer` - The read results.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate embedded_hal;
    /// # extern crate ht16k33;
    /// # use embedded_hal::blocking::i2c::WriteRead;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # fn main() {
    /// let mut i2c_mock = I2cMock::new(None);
    ///
    /// let mut read_buffer = [0u8; 16];
    /// i2c_mock.write_read(0, &[ht16k33::DisplayDataAddress::ROW_0.bits()], &mut read_buffer);
    ///
    /// # }
    /// ```
    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        trace!(self.logger, "write_read"; "address" => address, "bytes" => format!("{:?}", bytes), "buffer" => format!("{:?}", buffer));

        // The `bytes` have the `data_address` command + index to start reading from,
        // need to clear the command to extract the starting index.
        let mut data_offset = (bytes[0] ^ DisplayDataAddress::ROW_0.bits()) as usize;

        for value in buffer.iter_mut() {
            *value = self.data_values[data_offset];

            // The HT16K33 supports auto-increment and wrap-around, emulate that.
            data_offset = (data_offset + 1) % self.data_values.len();
        }

        Ok(())
    }
}

impl hal::blocking::i2c::Write for I2cMock {
    type Error = I2cMockError;

    /// `write` implementation.
    ///
    /// # Arguments
    ///
    /// * `_address` - The slave address. Ignored.
    /// * `bytes` - The command/address instructions to be written.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate embedded_hal;
    /// # extern crate ht16k33;
    /// # use embedded_hal::blocking::i2c::Write;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # fn main() {
    /// let mut i2c_mock = I2cMock::new(None);
    ///
    /// // First value is the data address, remaining values are to be written
    /// // starting at the data address which auto-increments and then wraps.
    /// let write_buffer = [ht16k33::DisplayDataAddress::ROW_0.bits(), 0u8, 0u8];
    ///
    /// i2c_mock.write(0, &write_buffer);
    ///
    /// # }
    /// ```
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        trace!(self.logger, "write"; "address" => address, "bytes" => format!("{:?}", bytes));

        // "Command-only" writes are length 1 and write-only, and cannot be read back,
        // discard them for simplicity.
        if bytes.len() == 1 {
            return Ok(());
        }

        // Other writes have data, store them.
        let mut data_offset = (bytes[0] ^ DisplayDataAddress::ROW_0.bits()) as usize;
        let data = &bytes[1..];

        for value in data.iter() {
            self.data_values[data_offset] = *value;

            // The HT16K33 supports auto-increment and wrap-around, emulate that.
            data_offset = (data_offset + 1) % self.data_values.len();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hal::blocking::i2c::{Write, WriteRead};

    const ADDRESS: u8 = 0;

    #[test]
    fn new() {
        let _i2c_mock = I2cMock::new(None);
    }

    #[test]
    fn write() {
        let mut i2c_mock = I2cMock::new(None);

        let write_buffer = [super::DisplayDataAddress::ROW_0.bits(), 1u8, 1u8];
        i2c_mock.write(ADDRESS, &write_buffer).unwrap();

        for value in 0..i2c_mock.data_values.len() {
            match value {
                0 | 1 => assert_eq!(
                    i2c_mock.data_values[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
                _ => assert_eq!(
                    i2c_mock.data_values[value], 0,
                    "index [{}] should be 0, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
            }
        }
    }

    #[test]
    fn write_with_offset() {
        let mut i2c_mock = I2cMock::new(None);

        let offset = 4u8;
        let write_buffer = [super::DisplayDataAddress::ROW_0.bits() | offset, 1u8, 1u8];
        i2c_mock.write(ADDRESS, &write_buffer).unwrap();

        for value in 0..i2c_mock.data_values.len() {
            match value {
                4 | 5 => assert_eq!(
                    i2c_mock.data_values[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
                _ => assert_eq!(
                    i2c_mock.data_values[value], 0,
                    "index [{}] should be 0, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
            }
        }
    }

    #[test]
    fn write_with_wraparound() {
        let mut i2c_mock = I2cMock::new(None);

        // Match the data values size, +2 to wrap around, +1 for the data command.
        let mut write_buffer = [1u8; super::ROWS_SIZE + 3];
        write_buffer[0] = super::DisplayDataAddress::ROW_0.bits();

        // These values should wrap and end up at indexes 0 & 1.
        write_buffer[write_buffer.len() - 1] = 2;
        write_buffer[write_buffer.len() - 2] = 2;

        i2c_mock.write(ADDRESS, &write_buffer).unwrap();

        for value in 0..i2c_mock.data_values.len() {
            match value {
                0 | 1 => assert_eq!(
                    i2c_mock.data_values[value], 2,
                    "index [{}] should be 2, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
                _ => assert_eq!(
                    i2c_mock.data_values[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
            }
        }
    }

    #[test]
    fn write_with_wraparound_and_offset() {
        let mut i2c_mock = I2cMock::new(None);

        // Match the data values size, +2 to wrap around, +1 for the data command.
        let mut write_buffer = [1u8; super::ROWS_SIZE + 3];

        let offset = 4u8;
        write_buffer[0] = super::DisplayDataAddress::ROW_0.bits() | offset;

        // These values should wrap and end up at indexes 4 & 5.
        write_buffer[write_buffer.len() - 1] = 2;
        write_buffer[write_buffer.len() - 2] = 2;

        i2c_mock.write(ADDRESS, &write_buffer).unwrap();

        for value in 0..i2c_mock.data_values.len() {
            match value {
                4 | 5 => assert_eq!(
                    i2c_mock.data_values[value], 2,
                    "index [{}] should be 2, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
                _ => assert_eq!(
                    i2c_mock.data_values[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, i2c_mock.data_values[value]
                ),
            }
        }
    }

    #[test]
    fn write_read() {
        let mut i2c_mock = I2cMock::new(None);

        i2c_mock.data_values[0] = 1;
        i2c_mock.data_values[1] = 1;

        let mut read_buffer = [0u8; super::ROWS_SIZE];
        i2c_mock
            .write_read(
                ADDRESS,
                &[super::DisplayDataAddress::ROW_0.bits()],
                &mut read_buffer,
            )
            .unwrap();

        for value in 0..read_buffer.len() {
            match value {
                0 | 1 => assert_eq!(
                    read_buffer[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, read_buffer[value]
                ),
                _ => assert_eq!(
                    read_buffer[value], 0,
                    "index [{}] should be 0, found [{}]",
                    value, read_buffer[value]
                ),
            }
        }
    }

    #[test]
    fn write_read_offset() {
        let mut i2c_mock = I2cMock::new(None);

        i2c_mock.data_values[2] = 1;
        i2c_mock.data_values[3] = 1;

        let mut read_buffer = [0u8; 4];

        let offset = 2u8;
        i2c_mock
            .write_read(
                ADDRESS,
                &[super::DisplayDataAddress::ROW_0.bits() | offset],
                &mut read_buffer,
            )
            .unwrap();

        for value in 0..read_buffer.len() {
            match value {
                0 | 1 => assert_eq!(
                    read_buffer[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, read_buffer[value]
                ),
                _ => assert_eq!(
                    read_buffer[value], 0,
                    "index [{}] should be 0, found [{}]",
                    value, read_buffer[value]
                ),
            }
        }
    }

    #[test]
    fn write_read_wraparound() {
        let mut i2c_mock = I2cMock::new(None);

        i2c_mock.data_values[2] = 1;
        i2c_mock.data_values[3] = 1;

        let mut read_buffer = [0u8; super::ROWS_SIZE + 4];

        i2c_mock
            .write_read(
                ADDRESS,
                &[super::DisplayDataAddress::ROW_0.bits()],
                &mut read_buffer,
            )
            .unwrap();

        for value in 0..read_buffer.len() {
            match value {
                2 | 3 | 18 | 19 => assert_eq!(
                    read_buffer[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, read_buffer[value]
                ),
                _ => assert_eq!(
                    read_buffer[value], 0,
                    "index [{}] should be 0, found [{}]",
                    value, read_buffer[value]
                ),
            }
        }
    }

    #[test]
    fn write_read_wraparound_and_offset() {
        let mut i2c_mock = I2cMock::new(None);

        i2c_mock.data_values[0] = 1;
        i2c_mock.data_values[1] = 1;

        let mut read_buffer = [0u8; super::ROWS_SIZE];

        let offset = 4u8;
        i2c_mock
            .write_read(
                ADDRESS,
                &[super::DisplayDataAddress::ROW_0.bits() | offset],
                &mut read_buffer,
            )
            .unwrap();

        for value in 0..read_buffer.len() {
            match value {
                // The indexes will be 12/13 b/c the data values are at 1/2, but the read is offset
                // by 4, so the read buffer will wraparound to load those values.
                12 | 13 => assert_eq!(
                    read_buffer[value], 1,
                    "index [{}] should be 1, found [{}]",
                    value, read_buffer[value]
                ),
                _ => assert_eq!(
                    read_buffer[value], 0,
                    "index [{}] should be 0, found [{}]",
                    value, read_buffer[value]
                ),
            }
        }
    }
}
