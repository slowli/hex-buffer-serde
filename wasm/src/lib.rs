#![no_std]

extern crate alloc;

use hex_buffer_serde::{Hex as _, HexForm};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[wasm_bindgen]
extern "C" {
    pub type Error;

    #[wasm_bindgen(constructor)]
    pub fn new(message: &str) -> Error;
}

#[derive(Serialize, Deserialize)]
struct TestData {
    #[serde(with = "HexForm")]
    buffer: Vec<u8>,
    #[serde(with = "HexForm")]
    array_buffer: [u8; 4],
    other_data: String,
}

impl TestData {
    fn reverse(&mut self) {
        self.buffer.reverse();
        self.array_buffer.reverse();
    }
}

fn to_js_error(err: &impl ToString) -> JsValue {
    let err = Error::new(&err.to_string());
    JsValue::from(err)
}

#[wasm_bindgen]
pub fn reverse(value: JsValue) -> Result<JsValue, JsValue> {
    let mut parsed: TestData =
        serde_wasm_bindgen::from_value(value).map_err(|err| to_js_error(&err))?;
    parsed.reverse();
    serde_wasm_bindgen::to_value(&parsed).map_err(|err| to_js_error(&err))
}
