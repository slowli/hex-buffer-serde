//! Types dependent on the `alloc` crate.

use serde::{
    de::{Error as DeError, Unexpected, Visitor},
    Deserializer, Serializer,
};

use alloc::{borrow::Cow, vec::Vec};
use core::{convert::TryFrom, fmt, marker::PhantomData};

/// Provides hex-encoded (de)serialization for `serde`.
///
/// Note that the trait is automatically implemented for types that
/// implement [`AsRef`]`<[u8]>` and [`TryFrom`]`<&[u8]>`.
///
/// # Examples
///
/// See [the crate-level docs](index.html#examples) for the examples of usage.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait Hex<T> {
    /// Error returned on unsuccessful deserialization.
    type Error: fmt::Display;

    /// Converts the value into bytes. This is used for serialization.
    ///
    /// The returned buffer can be either borrowed from the type, or created by the method.
    fn create_bytes(value: &T) -> Cow<'_, [u8]>;

    /// Creates a value from the byte slice.
    ///
    /// # Errors
    ///
    /// If this method fails, it should return a human-readable error description conforming
    /// to `serde` conventions (no upper-casing of the first letter, no punctuation at the end).
    fn from_bytes(bytes: &[u8]) -> Result<T, Self::Error>;

    /// Serializes the value for `serde`. This method is not meant to be overridden.
    ///
    /// The serialization is a lower-case hex string
    /// for [human-readable][hr] serializers (e.g., JSON or TOML), and the original bytes
    /// returned by [`Self::create_bytes()`] for non-human-readable ones.
    ///
    /// [hr]: serde::Serializer::is_human_readable()
    /// [`create_bytes`]: #tymethod.create_bytes
    fn serialize<S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        let value = Self::create_bytes(value);
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(value))
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
        struct HexVisitor;

        impl Visitor<'_> for HexVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("hex-encoded byte array")
            }

            fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
                hex::decode(value).map_err(|_| E::invalid_type(Unexpected::Str(value), &self))
            }

            // See the `deserializing_flattened_field` test for an example why this is needed.
            fn visit_bytes<E: DeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }
        }

        struct BytesVisitor;

        impl Visitor<'_> for BytesVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("byte array")
            }

            fn visit_bytes<E: DeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }

            fn visit_byte_buf<E: DeError>(self, value: Vec<u8>) -> Result<Self::Value, E> {
                Ok(value)
            }
        }

        let maybe_bytes = if deserializer.is_human_readable() {
            deserializer.deserialize_str(HexVisitor)
        } else {
            deserializer.deserialize_byte_buf(BytesVisitor)
        };
        maybe_bytes.and_then(|bytes| Self::from_bytes(&bytes).map_err(D::Error::custom))
    }
}

/// A dummy container for use inside `#[serde(with)]` attribute if the underlying type
/// implements [`Hex`].
///
/// # Why a separate container?
///
/// We need a separate type (instead of just using `impl<T> Hex<T> for T`)
/// both for code clarity and because otherwise invocations within generated `serde` code
/// would be ambiguous for types implementing `Serialize` / `Deserialize`.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Debug)]
pub struct HexForm<T>(PhantomData<T>);

impl<T, E> Hex<T> for HexForm<T>
where
    T: AsRef<[u8]> + for<'a> TryFrom<&'a [u8], Error = E>,
    E: fmt::Display,
{
    type Error = E;

    fn create_bytes(buffer: &T) -> Cow<'_, [u8]> {
        Cow::Borrowed(buffer.as_ref())
    }

    fn from_bytes(bytes: &[u8]) -> Result<T, Self::Error> {
        T::try_from(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_derive::{Deserialize, Serialize};
    use serde_json::json;

    use alloc::{
        borrow::ToOwned,
        string::{String, ToString},
        vec,
    };
    use core::array::TryFromSliceError;

    #[derive(Debug, Serialize, Deserialize)]
    struct Buffer([u8; 8]);

    impl AsRef<[u8]> for Buffer {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    impl TryFrom<&[u8]> for Buffer {
        type Error = TryFromSliceError;

        fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
            <[u8; 8]>::try_from(slice).map(Buffer)
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Test {
        #[serde(with = "HexForm::<Buffer>")]
        buffer: Buffer,
        other_field: String,
    }

    #[test]
    fn internal_type() {
        let json = json!({ "buffer": "0001020304050607", "other_field": "abc" });
        let value: Test = serde_json::from_value(json.clone()).unwrap();
        assert!(value
            .buffer
            .0
            .iter()
            .enumerate()
            .all(|(i, &byte)| i == usize::from(byte)));

        let json_copy = serde_json::to_value(&value).unwrap();
        assert_eq!(json, json_copy);
    }

    #[test]
    fn error_reporting() {
        let bogus_jsons = vec![
            serde_json::json!({
                "buffer": "bogus",
                "other_field": "test",
            }),
            serde_json::json!({
                "buffer": "c0ffe",
                "other_field": "test",
            }),
        ];

        for bogus_json in bogus_jsons {
            let err = serde_json::from_value::<Test>(bogus_json)
                .unwrap_err()
                .to_string();
            assert!(err.contains("expected hex-encoded byte array"), "{}", err);
        }
    }

    #[test]
    fn internal_type_with_derived_serde_code() {
        // ...and here, we may use original `serde` code.
        #[derive(Serialize, Deserialize)]
        struct OriginalTest {
            buffer: Buffer,
            other_field: String,
        }

        let test = Test {
            buffer: Buffer([1; 8]),
            other_field: "a".to_owned(),
        };
        assert_eq!(
            serde_json::to_value(test).unwrap(),
            json!({
                "buffer": "0101010101010101",
                "other_field": "a",
            })
        );

        let test = OriginalTest {
            buffer: Buffer([1; 8]),
            other_field: "a".to_owned(),
        };
        assert_eq!(
            serde_json::to_value(test).unwrap(),
            json!({
                "buffer": [1, 1, 1, 1, 1, 1, 1, 1],
                "other_field": "a",
            })
        );
    }

    #[test]
    fn external_type() {
        #[derive(Debug, PartialEq, Eq)]
        pub struct Buffer([u8; 8]);

        struct BufferHex(());

        impl Hex<Buffer> for BufferHex {
            type Error = &'static str;

            fn create_bytes(buffer: &Buffer) -> Cow<'_, [u8]> {
                Cow::Borrowed(&buffer.0)
            }

            fn from_bytes(bytes: &[u8]) -> Result<Buffer, Self::Error> {
                if bytes.len() == 8 {
                    let mut inner = [0; 8];
                    inner.copy_from_slice(bytes);
                    Ok(Buffer(inner))
                } else {
                    Err("invalid buffer length")
                }
            }
        }

        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
        struct Test {
            #[serde(with = "BufferHex")]
            buffer: Buffer,
            other_field: String,
        }

        let json = json!({ "buffer": "0001020304050607", "other_field": "abc" });
        let value: Test = serde_json::from_value(json.clone()).unwrap();
        assert!(value
            .buffer
            .0
            .iter()
            .enumerate()
            .all(|(i, &byte)| i == usize::from(byte)));

        let json_copy = serde_json::to_value(&value).unwrap();
        assert_eq!(json, json_copy);

        // Test binary / non-human readable format.
        let buffer = bincode::serialize(&value).unwrap();
        // Conversion to hex is needed to be able to search for a pattern.
        let buffer_hex = hex::encode(&buffer);
        // Check that the buffer is stored in the serialization compactly,
        // as original bytes.
        let needle = "0001020304050607";
        assert!(buffer_hex.contains(needle));

        let value_copy: Test = bincode::deserialize(&buffer).unwrap();
        assert_eq!(value_copy, value);
    }

    #[test]
    fn deserializing_flattened_field() {
        // The fields in the flattened structure are somehow read with
        // a human-readable `Deserializer`, even if the original `Deserializer`
        // is not human-readable.
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Inner {
            #[serde(with = "HexForm")]
            x: Vec<u8>,
            #[serde(with = "HexForm")]
            y: [u8; 16],
        }

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Outer {
            #[serde(flatten)]
            inner: Inner,
            z: String,
        }

        let value = Outer {
            inner: Inner {
                x: vec![1; 8],
                y: [0; 16],
            },
            z: "test".to_owned(),
        };

        let mut bytes = vec![];
        ciborium::into_writer(&value, &mut bytes).unwrap();
        let bytes_hex = hex::encode(&bytes);
        // Check that byte buffers are stored in the binary form.
        assert!(bytes_hex.contains(&"01".repeat(8)));
        assert!(bytes_hex.contains(&"00".repeat(16)));
        let value_copy = ciborium::from_reader(&bytes[..]).unwrap();
        assert_eq!(value, value_copy);
    }
}
