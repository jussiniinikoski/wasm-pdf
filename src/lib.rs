use wasm_bindgen::prelude::*;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate console_error_panic_hook;

mod pdf;
use pdf::json::JsDocument;
use pdf::json_out;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = generatePDF)]
    fn generate_file(s: &[u8]);
}

#[wasm_bindgen]
pub fn run(json: &JsValue) -> Result<(), JsValue> {
    // output panics to console.error
    console_error_panic_hook::set_once();
    let js_doc = get_js_doc(&json).unwrap();
    let bytes = match pdf::create(&js_doc) {
        Ok(b) => b,
        Err(s) => return Err(s),
    };
    generate_file(&bytes);
    Ok(())
}

/// Test Utility: Exports serde objects to json_out function (JS)
#[wasm_bindgen]
pub fn print_document(json: &JsValue) -> Result<(), JsValue> {
    let js_doc = get_js_doc(&json)?;
    let out = JsValue::from_serde(&js_doc).unwrap();
    json_out(&out);
    Ok(())
}

fn get_js_doc(json: &JsValue) -> Result<JsDocument, JsValue> {
    match json.into_serde() {
        Ok(doc) => Ok(doc),
        Err(e) => Err(JsValue::from_str(&format!(
            "Error. Could not parse JSON data. {}",
            e
        ))),
    }
}
