//! Fixed-length hex (de)serialization.

use serde::{
    de::{Error as DeError, Unexpected, Visitor},
    Deserializer, Serializer,
};

use core::{array::TryFromSliceError, convert::TryFrom, fmt, marker::PhantomData, mem, slice, str};

/// Analogue of [`Hex`](crate::Hex) for values that have constant-length byte presentation.
/// This allows to avoid dependency on the `alloc` crate and expresses the byte length constraint
/// via types.
///
/// # Examples
///
/// ```
/// use hex_buffer_serde::{ConstHex, ConstHexForm};
/// # use serde_derive::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Simple {
///     #[serde(with = "ConstHexForm")]
///     array: [u8; 16],
///     // `array` will be serialized as 32-char hex string
/// }
/// ```
///
/// Similarly to `Hex`, it is possible to define proxies implementing `ConstHex` for external
/// types, for example, keys from [`ed25519-dalek`](https://crates.io/crates/ed25519-dalek):
///
/// ```
/// use ed25519::{PublicKey, SecretKey};
/// use hex_buffer_serde::ConstHex;
/// # use serde_derive::{Deserialize, Serialize};
///
/// struct KeyHex(());
///
/// impl ConstHex<PublicKey, 32> for KeyHex {
///     type Error = ed25519::SignatureError;
///
///     fn create_bytes(pk: &PublicKey) -> [u8; 32] {
///         pk.to_bytes()
///     }
///
///     fn from_bytes(bytes: [u8; 32]) -> Result<PublicKey, Self::Error> {
///         PublicKey::from_bytes(&bytes)
///         // although `bytes` always has correct length, not all
///         // 32-byte sequences are valid Ed25519 public keys.
///     }
/// }
///
/// impl ConstHex<SecretKey, 32> for KeyHex {
///     type Error = core::convert::Infallible;
///
///     fn create_bytes(sk: &SecretKey) -> [u8; 32] {
///         sk.to_bytes()
///     }
///
///     fn from_bytes(bytes: [u8; 32]) -> Result<SecretKey, Self::Error> {
///         Ok(SecretKey::from_bytes(&bytes).unwrap())
///         // ^ unwrap() is safe; any 32-byte sequence is a valid
///         // Ed25519 secret key.
///     }
/// }
///
/// #[derive(Serialize, Deserialize)]
/// struct KeyPair {
///     #[serde(with = "KeyHex")]
///     public: PublicKey,
///     #[serde(with = "KeyHex")]
///     secret: SecretKey,
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "const_len")))]
pub trait ConstHex<T, const N: usize> {
    /// Error returned on unsuccessful deserialization.
    type Error: fmt::Display;

    /// Converts the value into bytes. This is used for serialization.
    fn create_bytes(value: &T) -> [u8; N];

    /// Creates a value from the byte slice.
    ///
    /// # Errors
    ///
    /// If this method fails, it should return a human-readable error description conforming
    /// to `serde` conventions (no upper-casing of the first letter, no punctuation at the end).
    fn from_bytes(bytes: [u8; N]) -> Result<T, Self::Error>;

    /// Serializes the value for `serde`. This method is not meant to be overridden.
    ///
    /// The serialization is a lower-case hex string
    /// for [human-readable][hr] serializers (e.g., JSON or TOML), and the original bytes
    /// returned by [`Self::create_bytes()`] for non-human-readable ones.
    ///
    /// [hr]: serde::Serializer::is_human_readable()
    fn serialize<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        // Transmutes a `u16` slice as a `u8` one. This is needed because it's currently
        // impossible to declare a buffer as `[u8; N * 2]`.
        fn as_u8_slice(slice: &mut [u16]) -> &mut [u8] {
            if slice.is_empty() {
                // Empty slices need special handling since `from_raw_parts_mut` doesn't accept
                // an empty pointer.
                &mut []
            } else {
                let byte_len = slice.len() * mem::size_of::<u16>();
                let data = (slice as *mut [u16]).cast::<u8>();
                unsafe {
                    // SAFETY: length is trivially correct, and `[u8]` does not require
                    // additional alignment compared to `[u16]`.
                    slice::from_raw_parts_mut(data, byte_len)
                }
            }
        }

        let value = Self::create_bytes(value);
        if serializer.is_human_readable() {
            let mut hex_slice = [0_u16; N];
            let hex_slice = as_u8_slice(&mut hex_slice);

            hex::encode_to_slice(value, hex_slice).unwrap();
            // ^ `unwrap` is safe: the length is statically correct.
            serializer.serialize_str(unsafe {
                // SAFETY: hex output is always valid UTF-8.
                str::from_utf8_unchecked(hex_slice)
            })
        } else {
            serializer.serialize_bytes(value.as_ref())
        }
    }

    /// Deserializes a value using `serde`. This method is not meant to be overridden.
    ///
    /// If the deserializer is [human-readable][hr] (e.g., JSON or TOML), this method
    /// expects a hex-encoded string. Otherwise, the method expects a byte array.
    ///
    /// [hr]: serde::Serializer::is_human_readable()
    fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default)]
        struct HexVisitor<const M: usize>;

        impl<'de, const M: usize> Visitor<'de> for HexVisitor<M> {
            type Value = [u8; M];

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "hex-encoded byte array of length {}", M)
            }

            fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
                let mut decoded = [0_u8; M];
                hex::decode_to_slice(value, &mut decoded)
                    .map_err(|_| E::invalid_type(Unexpected::Str(value), &self))?;
                Ok(decoded)
            }

            fn visit_bytes<E: DeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                <[u8; M]>::try_from(value).map_err(|_| E::invalid_length(value.len(), &self))
            }
        }

        #[derive(Default)]
        struct BytesVisitor<const M: usize>;

        impl<'de, const M: usize> Visitor<'de> for BytesVisitor<M> {
            type Value = [u8; M];

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "byte array of length {}", M)
            }

            fn visit_bytes<E: DeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                <[u8; M]>::try_from(value).map_err(|_| E::invalid_length(value.len(), &self))
            }
        }

        let maybe_bytes = if deserializer.is_human_readable() {
            deserializer.deserialize_str(HexVisitor::default())
        } else {
            deserializer.deserialize_bytes(BytesVisitor::default())
        };
        maybe_bytes.and_then(|bytes| Self::from_bytes(bytes).map_err(D::Error::custom))
    }
}

/// A dummy container for use inside `#[serde(with)]` attribute if the underlying type
/// implements [`ConstHex`].
#[cfg_attr(docsrs, doc(cfg(feature = "const_len")))]
#[derive(Debug)]
pub struct ConstHexForm<T>(PhantomData<T>);

impl<const N: usize> ConstHex<[u8; N], N> for ConstHexForm<[u8; N]> {
    type Error = TryFromSliceError;

    fn create_bytes(buffer: &[u8; N]) -> [u8; N] {
        *buffer
    }

    fn from_bytes(bytes: [u8; N]) -> Result<[u8; N], Self::Error> {
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::string::ToString;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Arrays {
        #[serde(with = "ConstHexForm")]
        array: [u8; 16],
        #[serde(with = "ConstHexForm")]
        longer_array: [u8; 32],
    }

    #[test]
    fn serializing_arrays() {
        let arrays = Arrays {
            array: [11; 16],
            longer_array: [240; 32],
        };
        let json = serde_json::to_string(&arrays).unwrap();
        assert!(json.contains(&"0b".repeat(16)));

        let arrays_copy: Arrays = serde_json::from_str(&json).unwrap();
        assert_eq!(arrays_copy, arrays);
    }

    #[test]
    fn deserializing_array_with_incorrect_length() {
        let json = serde_json::json!({
            "array": "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
            "longer_array": "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
        });
        let err = serde_json::from_value::<Arrays>(json)
            .unwrap_err()
            .to_string();

        assert!(err.contains("invalid type"), "{}", err);
        assert!(err.contains("expected hex-encoded byte array"), "{}", err);
    }

    #[test]
    fn deserializing_array_with_incorrect_length_from_binary_format() {
        #[derive(Debug, Serialize, Deserialize)]
        struct ArrayHolder<const N: usize>(#[serde(with = "ConstHexForm")] [u8; N]);

        let buffer = bincode::serialize(&ArrayHolder([5; 6])).unwrap();
        let err = bincode::deserialize::<ArrayHolder<4>>(&buffer).unwrap_err();

        assert_eq!(
            err.to_string(),
            "invalid length 6, expected byte array of length 4"
        );
    }

    #[test]
    fn custom_type() {
        use ed25519_compact::PublicKey;

        struct PublicKeyHex(());
        impl ConstHex<PublicKey, 32> for PublicKeyHex {
            type Error = ed25519_compact::Error;

            fn create_bytes(pk: &PublicKey) -> [u8; 32] {
                **pk
            }

            fn from_bytes(bytes: [u8; 32]) -> Result<PublicKey, Self::Error> {
                PublicKey::from_slice(&bytes)
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        struct Holder {
            #[serde(with = "PublicKeyHex")]
            public_key: PublicKey,
        }

        let json = serde_json::json!({
            "public_key": "06fac1f22240cffd637ead6647188429fafda9c9cb7eae43386ac17f61115075",
        });
        let holder: Holder = serde_json::from_value(json).unwrap();
        assert_eq!(holder.public_key[0], 6);

        let bogus_json = serde_json::json!({
            "public_key": "06fac1f22240cffd637ead6647188429fafda9c9cb7eae43386ac17f6111507",
        });
        let err = serde_json::from_value::<Holder>(bogus_json).unwrap_err();
        assert!(err
            .to_string()
            .contains("expected hex-encoded byte array of length 32"));
    }
}
