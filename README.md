# Hex encoding helper for serde

[![Travis Build Status](https://img.shields.io/travis/com/slowli/hex-buffer-serde/master.svg?label=Linux%20Build)](https://travis-ci.com/slowli/hex-buffer-serde) 
[![License: Apache-2.0](https://img.shields.io/github/license/slowli/hex-buffer-serde.svg)](https://github.com/slowli/hex-buffer-serde/blob/master/LICENSE)
![rust 1.36.0+ required](https://img.shields.io/badge/rust-1.36.0+-blue.svg?label=Required%20Rust)

**Documentation:** [![Docs.rs](https://docs.rs/hex-buffer-serde/badge.svg)](https://docs.rs/hex-buffer-serde/) 
[![crate docs (master)](https://img.shields.io/badge/master-yellow.svg?label=docs)](https://slowli.github.io/hex-buffer-serde/hex_buffer_serde/)

`hex-buffer-serde` is a helper crate allowing to serialize types, which logically correspond to a byte buffer,
in hex encoding within `serde`.

## Usage

See crate docs for the examples of usage.

## Alternatives

[`hex-serde`] provides similar functionality and is a viable alternative if you have the control over the type
that needs hex-encoding. This crate differs from `hex-serde` in the following ways:

- You don't need control over the (de)serialized type; it does not need to implement any specific "useful" traits
  (such as `AsRef<[u8]>`).
- Hex encoding is used only with human-readable (de)serializers (e.g., JSON or TOML). If the (de)serializer
  is not human-readable (e.g., bincode), the type is serialized as a byte array.

## License

`hex-buffer-serde` is licensed under the Apache License (Version 2.0). See [LICENSE](LICENSE) for details.

[`hex-serde`]: https://crates.io/crates/hex-serde
