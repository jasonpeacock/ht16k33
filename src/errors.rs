use core::fmt;

/// Errors encountered during validation.
#[derive(Debug)]
pub enum ValidationError {
    /// The value is too large.
    ValueTooLarge {
        /// Name of the value.
        name: &'static str,
        /// Value that failed validation.
        value: u8,
        /// Limit that the value exceeded.
        limit: u8,
        /// Whether the limit is inclusive or not.
        inclusive: bool,
    },
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl std::error::Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::ValueTooLarge {
                name,
                value,
                limit,
                inclusive,
            } => write!(
                f,
                "'{}' value [{}] must be less than (or equal: {}) [{}])",
                name, value, limit, inclusive
            ),
        }
    }
}
