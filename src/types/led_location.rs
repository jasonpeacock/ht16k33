use crate::constants::ROWS_SIZE;
use crate::errors::ValidationError;
use crate::types::DisplayData;
use crate::types::DisplayDataAddress;

use core::convert::TryFrom;
use core::fmt;

/// Represents the LED location.
///
/// The LED location is a ([`DisplayDataAddress`], [`DisplayData`]) pair, created from a validated
/// (`row`, `common`) pair of `u8` values.
///
/// # Example
///
/// ```
/// use ht16k33::LedLocation;
/// use ht16k33::DisplayData;
/// use ht16k33::DisplayDataAddress;
/// use ht16k33::ValidationError;
/// # fn main() -> Result<(), ValidationError>{
///
/// let row = 1u8;
/// let common = 2u8;
///
/// let location = LedLocation::new(row, common)?;
///
/// assert_eq!(ht16k33::DisplayDataAddress::ROW_1, location.row);
/// assert_eq!(ht16k33::DisplayData::COMMON_2, location.common);
///
/// # Ok(())
/// # }
/// ```
///
/// [`DisplayDataAddress`]: struct.DisplayDataAddress.html
/// [`DisplayData`]: struct.DisplayData.html
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LedLocation {
    /// The Display RAM `row` address.
    pub row: DisplayDataAddress,
    /// The Display RAM `common` data.
    pub common: DisplayData,
}

impl fmt::Display for LedLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LedLocation(row: {}, common: {})", self.row, self.common)
    }
}

impl LedLocation {
    /// Create an `LedLocation` with the given `row` and `common` values.
    ///
    /// # Errors
    ///
    /// The `row` and `common` values are validated to be within their respective [`ROWS_SIZE`] and
    /// [`COMMONS_SIZE`] ranges of the device. If validation fails then [`ht16k33::ValidationError::ValueTooLarge`] is
    /// returned.
    ///
    /// [`ROWS_SIZE`]: constant.ROWS_SIZE.html
    /// [`COMMONS_SIZE`]: constant.COMMONS_SIZE.html
    /// [`ht16k33::ValidationError::ValueTooLarge`]: enum.ValidationError.html#variant.ValueTooLarge
    ///
    /// ```should_panic
    /// use ht16k33::LedLocation;
    /// use ht16k33::ValidationError;
    /// # use ht16k33::ROWS_SIZE;
    /// # fn main() {
    /// # let row = ROWS_SIZE as u8;
    /// # let common = 2u8;
    ///
    /// let location = match LedLocation::new(row, common) {
    ///     Ok(location) => location,
    ///     Err(ValidationError) => panic!(),
    /// };
    ///
    /// # }
    /// ```
    #[allow(clippy::new_ret_no_self)]
    pub fn new(row: u8, common: u8) -> Result<Self, ValidationError> {
        let row = DisplayDataAddress::try_from(row)
            .map_err(|_| ValidationError::ValueTooLarge {
                name: "row",
                value: row,
                limit: ROWS_SIZE as u8,
                inclusive: false,
            })?;

        let common = DisplayData::try_from_common(common)?;

        Ok(LedLocation { row, common })
    }

    /// Return the `row` value.
    pub fn row_as_index(self) -> usize {
        self.row as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let location = LedLocation::default();

        assert!(
            DisplayDataAddress::ROW_0 == location.row
                && DisplayData::COMMON_NONE == location.common,
            "LedLocation default is (0, None)"
        );
    }

    #[test]
    fn new() {
        let location = LedLocation::new(0, 0).unwrap();

        assert!(
            DisplayDataAddress::ROW_0 == location.row && DisplayData::COMMON_0 == location.common,
            "LedLocation is (0, 0)"
        );

        let location = LedLocation::new(15, 7).unwrap();

        assert!(
            DisplayDataAddress::ROW_15 == location.row && DisplayData::COMMON_7 == location.common,
            "LedLocation is (15, 7)"
        );
    }

    #[test]
    #[should_panic]
    fn row_too_large() {
        let _ = LedLocation::new(16, 0).unwrap();
    }

    #[test]
    #[should_panic]
    fn common_too_large() {
        let _ = LedLocation::new(0, 8).unwrap();
    }

    #[test]
    fn row_as_index() {
        let location = LedLocation::new(2, 2).unwrap();
        assert_eq!(2usize, location.row_as_index());
    }
}
