/// Errors encountered during validation.
#[derive(Debug, Fail)]
pub enum ValidationError {
    /// The value is too large.
    #[fail(
        display = "'{}' value [{}] must be less than (or equal: {}) [{}])",
        name, value, inclusive, limit
    )]
    ValueTooLarge {
        /// Name of the value.
        name: String,
        /// Value that failed validation.
        value: u8,
        /// Limit that the value exceeded.
        limit: u8,
        /// Whether the limit is inclusive or not.
        inclusive: bool,
    },
}
