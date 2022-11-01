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
//! [`ConstHex`] is an analogue of [`Hex`] that can be used if the serialized buffer has
//! constant length known in compile time.
//!
//! # Crate Features
//!
//! - `alloc` (enabled by default). Enables types that depend on the `alloc` crate:
//!   [`Hex`] and [`HexForm`].
//! - `const_len` (disabled by default). Enables types that depend on const generics:
//!   [`ConstHex`] and [`ConstHexForm`].
//!
//! [`sodiumoxide`]: https://crates.io/crates/sodiumoxide
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
//! use serde_derive::{Deserialize, Serialize};
//!
//! # use std::borrow::Cow;
//! struct BufferHex; // a single-purpose type for use in `#[serde(with)]`
//! impl Hex<Buffer> for BufferHex {
//!     type Error = &'static str;
//!
//!     fn create_bytes(buffer: &Buffer) -> Cow<[u8]> {
//!         buffer.as_ref().into()
//!     }
//!
//!     fn from_bytes(bytes: &[u8]) -> Result<Buffer, Self::Error> {
//!         Buffer::from_slice(bytes).ok_or_else(|| "invalid byte length")
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
//! ## Use with internal types
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
// Documentation settings.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_root_url = "https://docs.rs/hex-buffer-serde/0.4.0")]
// Linter settings.
#![warn(missing_docs, missing_debug_implementations)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::must_use_candidate)]

#[cfg(any(test, feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "const_len")]
mod const_len;
#[cfg(feature = "const_len")]
pub use self::const_len::{ConstHex, ConstHexForm};
#[cfg(feature = "alloc")]
mod var_len;
#[cfg(feature = "alloc")]
pub use self::var_len::{Hex, HexForm};

#[cfg(not(any(feature = "const_len", feature = "alloc")))]
compile_error!(
    "At least one of `const_len` and `alloc` features must be enabled; \
     the crate is useless otherwise"
);

#[cfg(doctest)]
doc_comment::doctest!("../README.md");
