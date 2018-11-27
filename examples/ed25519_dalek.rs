// Copyright 2018 Alex Ostrovski
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

//! Example demonstrating how to use the crate with external types which don't implement
//! any "useful" traits (e.g., `AsRef<[u8]>` or `FromHex`).

extern crate ed25519_dalek as ed25519;
extern crate hex;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate hex_buffer_serde;
extern crate serde_json;

use ed25519::{ExpandedSecretKey, PublicKey, SecretKey};
use rand::thread_rng;

use std::borrow::Cow;

use hex_buffer_serde::Hex;

enum PublicKeyHex {}
impl Hex<PublicKey> for PublicKeyHex {
    fn create_bytes(value: &PublicKey) -> Cow<[u8]> {
        Cow::Borrowed(&*value.as_bytes())
    }

    fn from_bytes(bytes: &[u8]) -> Result<PublicKey, String> {
        PublicKey::from_bytes(bytes).map_err(|e| e.to_string())
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
    let secret_key: ExpandedSecretKey = (&secret_key).into();
    let public_key: PublicKey = secret_key.into();

    println!("Key hex: {}", hex::encode(public_key.as_bytes()));

    let data = SomeData {
        public_key,
        name: Some("our precious".to_owned()),
    };

    let json = serde_json::to_string_pretty(&data).unwrap();
    println!("JSON: {}", json);

    let bin = bincode::serialize(&data).unwrap();
    println!("bincode: {}", hex::encode(&bin));
}
