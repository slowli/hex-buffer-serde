// Copyright 2020 Alex Ostrovski
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Serializing byte buffers as hex strings with `serde`.
//!
//! # Problem
//!
//! Sometimes, you need to serialize a byte buffer (say, a newtype around `[u8; 32]` or `Vec<u8>`)
//! as a hex string. The problem is, the newtype in question can be defined in another crate
//! (for example, cryptographic types from [`sodiumoxide`]), so you can't implement `Serialize` /
//! `Deserialize` for the type due to Rust orphaning rules. (Or maybe `Serialize` / `Deserialize`
//! *are* implemented, but not in the desirable way.)
//!
//! # Solution
//!
//! The core of this crate is the [`Hex`] trait. It provides methods `serialize`
//! and `deserialize`, which signatures match the ones expected by `serde`. These methods
//! use the other two required methods of the trait. As all trait methods have no `self` argument,
//! the trait *can* be implemented for external types; the implementor may be an empty `enum`
//! designated specifically for this purpose. The implementor can then be used
//! for (de)serialization with the help of the `#[serde(with)]` attribute.
//!
//! [`sodiumoxide`]: https://crates.io/crates/sodiumoxide
//! [`Hex`]: trait.Hex.html
//!
//! # Crate Features
//!
//! - `std` (enabled by default): Enables types from the Rust standard library. Switching
//!   this feature off can be used for constrained environments, such as WASM. Note that
//!   the crate still requires an allocator (that is, the `alloc` crate) even
//!   if the `std` feature is disabled.
//!
//! # Examples
//!
//! ```
//! // Assume this type is defined in an external crate.
//! pub struct Buffer([u8; 8]);
//!
//! impl Buffer {
//!     pub fn from_slice(slice: &[u8]) -> Option<Self> {
//!         // snip
//! #       unimplemented!()
//!     }
//! }
//!
//! impl AsRef<[u8]> for Buffer {
//!     fn as_ref(&self) -> &[u8] {
//!         &self.0
//!     }
//! }
//!
//! // We define in our crate:
//! use hex_buffer_serde::Hex;
//! use serde_derive::*;
//!
//! # use std::borrow::Cow;
//! enum BufferHex {} // a single-purpose type for use in `#[serde(with)]`
//! impl Hex<Buffer> for BufferHex {
//!     fn create_bytes(buffer: &Buffer) -> Cow<[u8]> {
//!         buffer.as_ref().into()
//!     }
//!
//!     fn from_bytes(bytes: &[u8]) -> Result<Buffer, String> {
//!         Buffer::from_slice(bytes).ok_or_else(|| "invalid byte length".to_owned())
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct Example {
//!     #[serde(with = "BufferHex")]
//!     buffer: Buffer,
//!     // other fields...
//! }
//!
//! # fn main() {}
//! ```
//!
//! # Use with internal types
//!
//! The crate could still be useful if you have control over the serialized buffer type.
//! `Hex<T>` has a blanket implementation for types `T` satisfying certain constraints:
//! `AsRef<[u8]>` and `TryFrom<&[u8]>`. If these constraints are satisfied, you can
//! use `HexForm::<T>` in `#[serde(with)]`:
//!
//! ```
//! // It is necessary for `Hex` to be in scope in order
//! // for `serde`-generated code to use its `serialize` / `deserialize` methods.
//! use hex_buffer_serde::{Hex, HexForm};
//! # use serde_derive::*;
//! use core::{array::TryFromSliceError, convert::TryFrom};
//!
//! pub struct OurBuffer([u8; 8]);
//!
//! impl TryFrom<&[u8]> for OurBuffer {
//!     type Error = TryFromSliceError;
//!
//!     fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
//!         // snip
//! #       unimplemented!()
//!     }
//! }
//!
//! impl AsRef<[u8]> for OurBuffer {
//!     fn as_ref(&self) -> &[u8] {
//!         &self.0
//!     }
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct Example {
//!     #[serde(with = "HexForm::<OurBuffer>")]
//!     buffer: OurBuffer,
//!     // other fields...
//! }
//!
//! # fn main() {}
//! ```

#![no_std]
#![deny(missing_docs, missing_debug_implementations)]

extern crate alloc;

use serde::{de::Visitor, Deserializer, Serializer};

use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};
use core::{convert::TryFrom, fmt, marker::PhantomData};

/// Provides hex-encoded (de)serialization for `serde`.
///
/// Note that the trait is automatically implemented for types that
/// implement `AsRef<[u8]>` and `TryFrom<&[u8]>`.
pub trait Hex<T> {
    /// Converts the value into bytes. This is used for serialization.
    ///
    /// The returned buffer can be either borrowed from the type, or created by the method.
    fn create_bytes(value: &T) -> Cow<[u8]>;

    /// Creates a value from the byte slice.
    ///
    /// # Errors
    ///
    /// If this method fails, it should return a human-readable error description conforming
    /// to `serde` conventions (no upper-casing of the first letter, no punctuation at the end).
    fn from_bytes(bytes: &[u8]) -> Result<T, String>;

    /// Serializes the value for `serde`. This method is not meant to be overridden.
    ///
    /// The serialization is a lower-case hex string
    /// for [human-readable][hr] serializers (e.g., JSON or TOML), and the original bytes
    /// returned by [`create_bytes`] for non-human-readable ones.
    ///
    /// [hr]: https://docs.rs/serde/^1.0/serde/trait.Serializer.html#method.is_human_readable
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
    /// [hr]: https://docs.rs/serde/^1.0/serde/trait.Deserializer.html#method.is_human_readable
    fn deserialize<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error as DeError;

        struct HexVisitor;

        impl<'de> Visitor<'de> for HexVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("hex-encoded byte array")
            }

            fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
                hex::decode(value).map_err(E::custom)
            }

            // See the `deserializing_flattened_field` test for an example why this is needed.
            fn visit_bytes<E: DeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }
        }

        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("byte array")
            }

            fn visit_bytes<E: DeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(value.to_vec())
            }
        }

        let maybe_bytes = if deserializer.is_human_readable() {
            deserializer.deserialize_str(HexVisitor)
        } else {
            deserializer.deserialize_bytes(BytesVisitor)
        };
        maybe_bytes.and_then(|bytes| Self::from_bytes(&bytes).map_err(D::Error::custom))
    }
}

/// A dummy container for use inside `#[serde(with)]` attribute.
///
/// # Why a separate container?
///
/// We need a separate type (instead of just using `impl<T> Hex<T> for T`)
/// both for code clarity and because otherwise invocations within generated `serde` code
/// would be ambiguous for types implementing `Serialize` / `Deserialize`.
#[derive(Debug)]
pub struct HexForm<T>(PhantomData<T>);

impl<T> Hex<T> for HexForm<T>
where
    T: AsRef<[u8]> + for<'a> TryFrom<&'a [u8]>,
    for<'a> <T as TryFrom<&'a [u8]>>::Error: ToString,
{
    fn create_bytes(buffer: &T) -> Cow<[u8]> {
        Cow::Borrowed(buffer.as_ref())
    }

    fn from_bytes(bytes: &[u8]) -> Result<T, String> {
        T::try_from(bytes).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::*;
    use serde_json::json;

    use super::*;
    use alloc::{borrow::ToOwned, vec};
    use core::array::TryFromSliceError;

    #[test]
    fn internal_type() {
        pub struct Buffer([u8; 8]);

        impl AsRef<[u8]> for Buffer {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl TryFrom<&[u8]> for Buffer {
            type Error = String;

            fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
                <[u8; 8]>::try_from(slice)
                    .map(Buffer)
                    .map_err(|_| "!".to_owned())
            }
        }

        #[derive(Serialize, Deserialize)]
        struct Test {
            #[serde(with = "HexForm::<Buffer>")]
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
    }

    #[test]
    fn internal_type_with_derived_serde_code() {
        #[derive(Serialize, Deserialize)]
        pub struct Buffer([u8; 8]);

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

        // here, a hex form should be used.
        #[derive(Serialize, Deserialize)]
        struct HexTest {
            #[serde(with = "HexForm::<Buffer>")]
            buffer: Buffer,
            other_field: String,
        }

        // ...and here, we may use original `serde` code.
        #[derive(Serialize, Deserialize)]
        struct OriginalTest {
            buffer: Buffer,
            other_field: String,
        }

        let test = HexTest {
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

        enum BufferHex {}
        impl Hex<Buffer> for BufferHex {
            fn create_bytes(buffer: &Buffer) -> Cow<[u8]> {
                Cow::Borrowed(&buffer.0)
            }

            fn from_bytes(bytes: &[u8]) -> Result<Buffer, String> {
                if bytes.len() == 8 {
                    let mut inner = [0; 8];
                    inner.copy_from_slice(bytes);
                    Ok(Buffer(inner))
                } else {
                    Err("invalid buffer length".to_owned())
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

        let bytes = serde_cbor::to_vec(&value).unwrap();
        let bytes_hex = hex::encode(&bytes);
        // Check that byte buffers are stored in the binary form.
        assert!(bytes_hex.contains(&"01".repeat(8)));
        assert!(bytes_hex.contains(&"00".repeat(16)));
        let value_copy = serde_cbor::from_slice(&bytes).unwrap();
        assert_eq!(value, value_copy);
    }
}
