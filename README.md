# Hex encoding helper for serde

[![Build Status](https://github.com/slowli/hex-buffer-serde/workflows/Rust/badge.svg?branch=master)](https://github.com/slowli/hex-buffer-serde/actions) 
[![License: Apache-2.0](https://img.shields.io/github/license/slowli/hex-buffer-serde.svg)](https://github.com/slowli/hex-buffer-serde/blob/master/LICENSE)
![rust 1.41.0+ required](https://img.shields.io/badge/rust-1.41.0+-blue.svg?label=Required%20Rust)

**Documentation:** [![Docs.rs](https://docs.rs/hex-buffer-serde/badge.svg)](https://docs.rs/hex-buffer-serde/) 
[![crate docs (master)](https://img.shields.io/badge/master-yellow.svg?label=docs)](https://slowli.github.io/hex-buffer-serde/hex_buffer_serde/)

`hex-buffer-serde` is a helper crate allowing to serialize types,
which logically correspond to a byte buffer, in hex encoding within `serde`.

## Usage

Add this to your `Crate.toml`:

```toml
[dependencies]
hex-buffer-serde = "0.3.0"
```

Basic usage:

```rust
use hex_buffer_serde::{Hex as _, HexForm};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Example {
    // will be serialized as hex string in human-readable formats
    // (e.g., JSON or TOML), and as a byte array in binary formats
    // (e.g., bincode).
    #[serde(with = "HexForm")]
    buffer: [u8; 32],
    // other fields...
}
```

See crate docs for more examples of usage.

## Supported Rust Versions

The `hex-buffer-serde` crate is built against recent stable Rust versions.
The minimum of these versions is indicated in the badge at the top of this file.
The `const_len` crate feature requires a newer Rust version (1.51+),
but it is disabled by default.

A bump of the minimum supported Rust version **will not** be considered
a semantically breaking change. It is, however, guaranteed that the crate
will build on some stable Rust toolchain.

## Alternatives

[`hex-serde`] provides similar functionality and is a viable alternative
if you have the control over the type that needs hex-encoding.
This crate differs from `hex-serde` in the following ways:

- You don't need control over the (de)serialized type; it does not need
  to implement any specific "useful" traits (such as `AsRef<[u8]>`).
- Hex encoding is used only with human-readable (de)serializers (e.g., JSON or TOML).
  If the (de)serializer is not human-readable (e.g., bincode),
  the type is serialized as a byte array.

## License

`hex-buffer-serde` is licensed under the Apache License (Version 2.0).
See [LICENSE](LICENSE) for details.

[`hex-serde`]: https://crates.io/crates/hex-serde
