use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
extern crate wasm_pdf_gen;

use wasm_pdf_gen::pdf::create;
use wasm_pdf_gen::pdf::json::JsDocument;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    Ok(())
}

#[wasm_bindgen]
pub fn generate_pdf_bytes(json: &JsValue) -> Result<Vec<u8>, JsValue> {
    let js_doc = get_js_doc(json)?;
    let bytes = match create(&js_doc) {
        Ok(b) => b,
        Err(s) => return Err(JsValue::from_str(s)),
    };
    Ok(bytes)
}

fn get_js_doc(json: &JsValue) -> Result<JsDocument, JsValue> {
    match serde_wasm_bindgen::from_value(json.into()) {
        Ok(doc) => Ok(doc),
        Err(e) => Err(JsValue::from_str(&format!(
            "Error. Could not parse JSON data. {}",
            e
        ))),
    }
}
