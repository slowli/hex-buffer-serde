//! Example demonstrating how to use the crate with external types which don't implement
//! any "useful" traits (e.g., `AsRef<[u8]>` or `FromHex`).
//!
//! Also, checks that the crate is usable with `no_std`.

#![no_std]

extern crate alloc;

use ed25519_compact::PublicKey;
use serde_derive::*;

use alloc::{
    borrow::{Cow, ToOwned},
    string::String,
};

use hex_buffer_serde::Hex;

struct PublicKeyHex(());

impl Hex<PublicKey> for PublicKeyHex {
    type Error = ed25519_compact::Error;

    fn create_bytes(value: &PublicKey) -> Cow<'_, [u8]> {
        Cow::Borrowed(&value[..])
    }

    fn from_bytes(bytes: &[u8]) -> Result<PublicKey, Self::Error> {
        PublicKey::from_slice(bytes)
    }
}

#[derive(Serialize, Deserialize)]
struct SomeData {
    #[serde(with = "PublicKeyHex")]
    public_key: PublicKey,
    name: Option<String>,
}

fn main() {
    let public_key =
        hex::decode("06fac1f22240cffd637ead6647188429fafda9c9cb7eae43386ac17f61115075").unwrap();
    let public_key = PublicKey::from_slice(&public_key).unwrap();

    let key_hex = hex::encode(&public_key[..]);

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
