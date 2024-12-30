/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief lib
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use thiserror::Error;
use wasm_bindgen::prelude::*;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to serialize")]
    Serialization(#[from] serde_wasm_bindgen::Error),

    #[error("Optics error: {0}")]
    OpticParse(#[from] optics::Error),

    #[error("Json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl From<Error> for JsValue {
    fn from(val: Error) -> Self {
        JsValue::from_str(&format!("{val:?}"))
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

#[wasm_bindgen]
pub struct Optic;

#[wasm_bindgen]
impl Optic {
    #[wasm_bindgen(js_name = parsePreferenceOptic)]
    pub fn parse_preference_optic(contents: JsValue) -> Result<JsValue, Error> {
        let optic_contents: String = serde_wasm_bindgen::from_value(contents)?;
        let host_rankings = optics::Optic::parse(&optic_contents)?.host_rankings;

        let rankings_json = serde_json::to_string(&host_rankings)?;

        console_log(&("Parsed rankings to JSON: ".to_owned() + &rankings_json));

        Ok(serde_wasm_bindgen::to_value(&rankings_json)?)
    }
}
