//! Example demonstrating how to use the crate with external types which don't implement
//! any "useful" traits (e.g., `AsRef<[u8]>` or `FromHex`).
//!
//! Also, checks that the crate is usable with `no_std`.

#![no_std]

extern crate alloc;

use ed25519::{PublicKey, SecretKey};
use rand::thread_rng;
use serde_derive::*;

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use hex_buffer_serde::Hex;

struct PublicKeyHex(());

impl Hex<PublicKey> for PublicKeyHex {
    type Error = ed25519::SignatureError;

    fn create_bytes(value: &PublicKey) -> Cow<'_, [u8]> {
        Cow::Borrowed(&*value.as_bytes())
    }

    fn from_bytes(bytes: &[u8]) -> Result<PublicKey, Self::Error> {
        PublicKey::from_bytes(bytes)
    }
}

#[derive(Serialize, Deserialize)]
struct SomeData {
    // Note that we have enabled `serde` feature in `Cargo.toml`. Thus,
    // `PublicKey` implements `Serialize` / `Deserialize`, but not in the way we want
    // (the value is just written as an array of separate bytes).
    #[serde(with = "PublicKeyHex")]
    public_key: PublicKey,
    name: Option<String>,
}

fn main() {
    let secret_key = SecretKey::generate(&mut thread_rng());
    let public_key: PublicKey = (&secret_key).into();

    let key_hex = hex::encode(public_key.as_bytes());

    let data = SomeData {
        public_key,
        name: Some("our precious".to_owned()),
    };

    let json = serde_json::to_string_pretty(&data).unwrap();
    assert!(json.contains(&key_hex));

    let bin = bincode::serialize(&data).unwrap();
    assert!(bin
        .windows(key_hex.len())
        .all(|window| window != key_hex.as_bytes()));
    let bin = hex::encode(&bin);
    assert!(bin.contains(&key_hex));
}
