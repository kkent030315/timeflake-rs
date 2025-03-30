use core::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The provided bytes resulted in an invalid Timeflake value.
    ///
    /// This happens when the integer value is out of the valid range.
    InvalidFlake,

    /// Failed to parse the provided string into a Timeflake.
    ParseError {
        /// The string that failed to parse.
        input: String,
        /// The detailed reason for the parse failure.
        reason: String,
    },

    /// The timestamp component is invalid (exceeds MAX_TIMESTAMP).
    InvalidTimestamp(u64),

    /// The random component is invalid (exceeds MAX_RANDOM).
    InvalidRandom,

    /// An error occurred during conversion to or from UUID.
    UuidError(String),

    /// General conversion error.
    ConversionError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidFlake => write!(f, "Invalid Timeflake: value out of valid range"),
            Error::ParseError { input, reason } => {
                write!(f, "Failed to parse '{}' as Timeflake: {}", input, reason)
            }
            Error::InvalidTimestamp(ts) => {
                write!(f, "Invalid timestamp: {} exceeds maximum allowed value", ts)
            }
            Error::InvalidRandom => {
                write!(f, "Invalid random component: exceeds maximum allowed value")
            }
            Error::UuidError(msg) => write!(f, "UUID error: {}", msg),
            Error::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
