# Changelog

All notable changes to this project will be documented in this file.
The project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Switch to 2021 Rust edition and bump the minimum supported Rust version to 1.57.

### Internal improvements

- Take advantage of owned byte buffers for binary serializers.

## 0.3.0 - 2021-05-03

### Added

- Add `ConstHex` / `ConstHexForm` for types with constant-length
  hex serialization (gated behind the `const_len` feature).
  Using `ConstHex` allows avoiding dependency on the `alloc` crate.

### Changed

- Add `Error` associated type to the `Hex` trait to avoid mandatory `String`
  allocations.

### Fixed

- Fix no-std support: replace `std` feature with `alloc` and propagate it
  to the `hex` crate. The `alloc` feature is required unless const generics
  are used (see above).

## 0.2.2 - 2020-12-05

### Internal improvements

- Use 2018 edition idioms and improve code style in general.

## 0.2.1 - 2020-03-15

### Added

- Mark the crate as not needing the standard library.

## 0.2.0 - 2019-04-15

### Changed

- Use 2018 Rust edition.
- Use `TryFrom` trait, which was stabilized in Rust v1.34.

## 0.1.1 - 2018-11-30

### Fixed

- Fix bug with deserializing fields in flattened structs.

## 0.1.0 - 2018-11-27

The initial release of `hex-buffer-serde`.
