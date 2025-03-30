//! Rust port of Timeflake, a 128-bit, roughly-ordered, URL-safe UUID.
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! timeflake = "0.1.0"
//! ```
//!
//! ```
//! use timeflake::Timeflake;
//!
//! fn main() {
//!     let mut rng = rand::rng();
//!     let flake = Timeflake::new_random(&mut rng);
//!     println!("{flake}");
//! }
//! ```

use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};
use utcnow::UtcTime;

use error::{Error, Result};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use rand::Rng;
#[cfg(feature = "uuid")]
use uuid::Uuid;

#[cfg(test)]
mod tests;

pub mod error;

/// The Base62 character set used for encoding and decoding [Timeflake]s.
///
/// This set consists of:
/// - Digits: `0-9`
/// - Uppercase letters: `A-Z`
/// - Lowercase letters: `a-z`
///
/// Base62 is a URL-safe encoding commonly used for compact representations of large numbers.
pub const BASE62: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
/// The hexadecimal character set used for encoding [Timeflake]s as hexadecimal strings.
///
/// This set consists of:
/// - Digits: `0-9`
/// - Lowercase letters: `a-f`
///
/// Hexadecimal encoding is often used for debugging and lower-level data representations.
pub const HEX: &str = "0123456789abcdef";
/// The maximum possible timestamp component in a [Timeflake].
///
/// This value is derived from the 48-bit space allocated for the timestamp
/// and corresponds to approximately 8910 years from the Unix epoch.
pub const MAX_TIMESTAMP: u64 = 281474976710655;
/// The maximum possible random component in a [Timeflake].
///
/// This value represents the upper bound of the 80-bit random component,
/// which ensures uniqueness across multiple Timeflake generations.
pub const MAX_RANDOM: &str = "1208925819614629174706175";
/// The maximum possible integer value of a [Timeflake].
///
/// This is the largest possible 128-bit integer, covering both the timestamp
/// and random components.
pub const MAX_TIMEFLAKE: &str = "340282366920938463463374607431768211455";

/// Represents a Timeflake, a unique identifier combining timestamp and random data.
///
/// A Timeflake is a 128-bit, roughly-ordered, URL-safe UUID compatible with
/// the existing UUID ecosystem.
///
/// # Example
///
/// ```
/// use timeflake::Timeflake;
///
/// fn main() {
///     let mut rng = rand::rng();
///     let flake = Timeflake::new_random(&mut rng);
///     println!("{flake}");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Timeflake {
    /// Raw bytes representation of this Timeflake
    bytes: [u8; 16],
    /// Integer representation of this Timeflake
    int_value: BigUint,
}

impl Timeflake {
    /// Create a new [Timeflake] with generated random component and current UNIX timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use timeflake::Timeflake;
    ///
    /// let mut rng = rand::rng();
    /// let flake = Timeflake::new_random(&mut rng);
    /// ```
    #[must_use]
    pub fn new_random<R: Rng>(rng: &mut R) -> Self {
        let utc_time = UtcTime::now().unwrap();
        let now = utc_time.as_millis() as u64;

        let mut random_bytes = [0u8; 10];
        rng.fill(&mut random_bytes);
        let random = BigUint::from_bytes_be(&random_bytes);

        Self::from_components(now, &random).unwrap()
    }

    /// Create a new [Timeflake] from full 16 bytes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidFlake`] if the bytes represent a value outside the valid range.
    #[must_use]
    pub fn from_bytes(bytes: [u8; 16]) -> Result<Self> {
        let int_value = bytes_to_biguint(&bytes);
        if int_value > max_timeflake_biguint() {
            return Err(Error::InvalidFlake);
        }

        Ok(Timeflake { bytes, int_value })
    }

    /// Create a new [Timeflake] from 16 bytes, panicking if the value is invalid.
    ///
    /// This function behaves similarly to [`Timeflake::from_bytes`], but will panic if the value
    /// exceeds the maximum allowed range.
    ///
    /// # Panics
    ///
    /// Panics if the bytes represent a value outside the valid range.
    ///
    /// # Examples
    ///
    /// ```
    /// use timeflake::Timeflake;
    ///
    /// let bytes: [u8; 16] = [0x00; 16];
    /// let flake = Timeflake::from_bytes_checked(bytes);
    /// ```
    #[must_use]
    pub fn from_bytes_checked(bytes: [u8; 16]) -> Self {
        Self::from_bytes(bytes).unwrap()
    }

    /// Create a new [Timeflake] from UNIX timestamp and random components.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidTimestamp`] if the timestamp exceeds the maximum allowed value.
    /// Returns [`Error::InvalidRandom`] if the random component exceeds the maximum allowed value.
    #[must_use]
    pub fn from_components(timestamp: u64, random: &BigUint) -> Result<Self> {
        if timestamp > MAX_TIMESTAMP {
            return Err(Error::InvalidTimestamp(timestamp));
        }

        if random > &max_random_biguint() {
            return Err(Error::InvalidRandom);
        }

        // Combine timestamp and random
        let ts_biguint = BigUint::from(timestamp);
        let int_value = (ts_biguint << 80) | random;
        let bytes = biguint_to_bytes(&int_value)?;

        Ok(Timeflake { bytes, int_value })
    }

    /// Create a new [Timeflake] from timestamp and random components, panicking if the values are invalid.
    ///
    /// This function behaves similarly to [`Timeflake::from_components`], but will panic if either
    /// the timestamp or the random component exceeds the maximum allowed value.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The `timestamp` exceeds the maximum allowed value (`MAX_TIMESTAMP`).
    /// - The `random` component exceeds the maximum allowed value.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_bigint::BigUint;
    /// use timeflake::Timeflake;
    ///
    /// let timestamp: u64 = 1_674_354_800; // Valid timestamp
    /// let random = BigUint::from(12345u64); // Valid random component
    /// let flake = Timeflake::from_components_checked(timestamp, &random);
    /// ```
    #[must_use]
    pub fn from_components_checked(timestamp: u64, random: &BigUint) -> Self {
        Self::from_components(timestamp, random).unwrap()
    }

    /// Create a new [Timeflake] from a base62-encoded string.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ParseError`] if the input string is not a valid base62 encoding.
    /// Returns [`Error::InvalidFlake`] if the decoded value exceeds the maximum allowed value.
    #[must_use]
    pub fn from_base62<S: AsRef<str>>(s: S) -> Result<Self> {
        let decoded = match base62::decode(s.as_ref()) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(Error::ParseError {
                    input: s.as_ref().to_string(),
                    reason: "Invalid base62 encoding".to_string(),
                });
            }
        };

        let int_value = BigUint::from_bytes_be(&decoded.to_be_bytes());
        if int_value > max_timeflake_biguint() {
            return Err(Error::InvalidFlake);
        }
        let bytes = biguint_to_bytes(&int_value)?;

        Ok(Timeflake { bytes, int_value })
    }

    /// Create a new [Timeflake] from a base62-encoded string, panicking if the value is invalid.
    ///
    /// This function behaves similarly to [`Timeflake::from_base62`], but will panic if the value
    /// exceeds the maximum allowed range or if the input is not a valid base62 encoding.
    ///
    /// # Panics
    ///
    /// Panics if the input string is not valid base62 or if the decoded value exceeds the maximum allowed range.
    ///
    /// # Examples
    ///
    /// ```
    /// use timeflake::Timeflake;
    ///
    /// let flake = Timeflake::from_base62_checked("00000000000000000000");
    /// ```
    #[must_use]
    pub fn from_base62_checked<S: AsRef<str>>(s: S) -> Self {
        Self::from_base62(s).unwrap()
    }

    /// Create a new [Timeflake] from a [BigUint].
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidFlake`] if the value exceeds the maximum allowed range.
    ///
    /// # Examples
    ///
    /// ```
    /// use timeflake::Timeflake;
    /// use num_bigint::BigUint;
    ///
    /// let value = BigUint::from(12345u64);
    /// let flake = Timeflake::from_bigint(value).unwrap();
    /// ```
    #[must_use]
    pub fn from_bigint(value: BigUint) -> Result<Self> {
        let bytes = biguint_to_bytes(&value)?;
        Self::from_bytes(bytes)
    }

    /// Create a new [Timeflake] from a [BigUint], panicking if the value is invalid.
    ///
    /// This function behaves similarly to [`Timeflake::from_bigint`], but will panic if the value
    /// exceeds the maximum allowed range.
    ///
    /// # Panics
    ///
    /// Panics if the value exceeds the maximum allowed range.
    ///
    /// # Examples
    ///
    /// ```
    /// use timeflake::Timeflake;
    /// use num_bigint::BigUint;
    ///
    /// let value = BigUint::from(12345u64);
    /// let flake = Timeflake::from_bigint_checked(value);
    /// ```
    #[must_use]
    pub fn from_bigint_checked(value: BigUint) -> Self {
        Self::from_bigint(value).unwrap()
    }

    /// Create a new [Timeflake] from a UUID.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidFlake`] if the UUID represents a value outside the valid range.
    #[cfg(feature = "uuid")]
    #[must_use]
    pub fn from_uuid(uuid: Uuid) -> Result<Self> {
        Self::from_bytes(uuid.into_bytes())
    }

    /// Create a new [Timeflake] from a UUID, panicking if the value is invalid.
    ///
    /// This function behaves similarly to [`Timeflake::from_uuid`], but will panic if the UUID
    /// represents a value outside the valid range.
    ///
    /// # Panics
    ///
    /// Panics if the UUID represents a value outside the valid range.
    ///
    /// # Examples
    ///
    /// ```
    /// use timeflake::Timeflake;
    /// use uuid::Uuid;
    ///
    /// let uuid = Uuid::parse_str("00000000-0000-4000-8000-000000000000").unwrap();
    /// let flake = Timeflake::from_uuid_checked(uuid);
    /// ```
    #[cfg(feature = "uuid")]
    #[must_use]
    pub fn from_uuid_checked(uuid: Uuid) -> Self {
        Self::from_uuid(uuid).unwrap()
    }

    /// Returns the UUID representation of this Timeflake.
    #[cfg(feature = "uuid")]
    pub fn to_uuid(&self) -> Uuid {
        Uuid::from_bytes(self.bytes)
    }

    /// Returns the base62 string representation of this Timeflake.
    pub fn to_base62(&self) -> String {
        let bytes = u128::from_be_bytes(self.bytes);
        let encoded = base62::encode(bytes);

        // Pad with leading zeros if necessary
        let padding = 22;
        if encoded.len() < padding {
            let zeros = "0".repeat(padding - encoded.len());
            return zeros + &encoded;
        }

        encoded
    }

    /// Returns the timestamp component of this Timeflake.
    pub fn timestamp(&self) -> u64 {
        let shifted: BigUint = &self.int_value >> 80;
        shifted.to_u64().unwrap()
    }

    /// Returns the random component of this Timeflake.
    pub fn random(&self) -> BigUint {
        &self.int_value & &max_random_biguint()
    }

    /// Returns the hexadecimal string representation of this Timeflake.
    pub fn to_hex(&self) -> String {
        hex::encode(self.bytes)
    }

    /// Returns the raw bytes of this Timeflake.
    pub fn to_bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Returns the integer value of this Timeflake.
    pub fn to_bigint(&self) -> &BigUint {
        &self.int_value
    }
}

impl FromStr for Timeflake {
    type Err = Error;

    /// Parse a string as a [Timeflake] accepting both hexadecimal and base62 encodings.
    fn from_str(s: &str) -> Result<Self> {
        // Try parsing as hex
        if s.len() == 32 && s.chars().all(|c| HEX.contains(c)) {
            let bytes = hex::decode(s).map_err(|e| Error::ParseError {
                input: s.to_string(),
                reason: format!("Invalid hex: {}", e),
            })?;
            if bytes.len() != 16 {
                return Err(Error::ParseError {
                    input: s.to_string(),
                    reason: format!("Expected 16 bytes, got {}", bytes.len()),
                });
            }

            let mut array = [0u8; 16];
            array.copy_from_slice(&bytes);
            return Self::from_bytes(array);
        }

        // Try parsing as base62
        if s.len() <= 22 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Self::from_base62(s);
        }

        Err(Error::ParseError {
            input: s.to_string(),
            reason: "String must be either a 32-character hex string or a base62 string"
                .to_string(),
        })
    }
}

impl PartialEq for Timeflake {
    fn eq(&self, other: &Self) -> bool {
        self.int_value == other.int_value
    }
}

impl Eq for Timeflake {}

impl PartialOrd for Timeflake {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timeflake {
    fn cmp(&self, other: &Self) -> Ordering {
        self.int_value.cmp(&other.int_value)
    }
}

impl Hash for Timeflake {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl fmt::Display for Timeflake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_base62())
    }
}

/// Helper routine to convert bytes to BigUint
#[inline(always)]
fn bytes_to_biguint(bytes: &[u8; 16]) -> BigUint {
    let mut result = BigUint::from(0u8);
    for &byte in bytes {
        result = (result << 8) | BigUint::from(byte);
    }
    result
}

/// Helper function to convert BigUint to bytes
#[inline(always)]
fn biguint_to_bytes(n: &BigUint) -> Result<[u8; 16]> {
    let bytes = n.to_bytes_be();
    let mut result = [0u8; 16];

    if bytes.len() > 16 {
        return Err(Error::ConversionError(format!(
            "BigUint is too large to fit in 16 bytes (got {} bytes)",
            bytes.len()
        )));
    }

    // Pad with leading zeros if necessary
    let offset = 16 - bytes.len();
    result[offset..].copy_from_slice(&bytes);

    Ok(result)
}

/// Reinterpret the [MAX_RANDOM] as a [BigUint]
#[inline(always)]
pub fn max_random_biguint() -> BigUint {
    BigUint::parse_bytes(MAX_RANDOM.as_bytes(), 10).unwrap()
}

/// Reinterpret the [MAX_TIMEFLAKE] as a [BigUint]
#[inline(always)]
pub fn max_timeflake_biguint() -> BigUint {
    BigUint::parse_bytes(MAX_TIMEFLAKE.as_bytes(), 10).unwrap()
}
