use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;
extern crate pdf_gen;

use pdf_gen::pdf::json::JsDocument;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = generatePDF)]
    fn generate_file(s: &[u8]);
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);
    #[wasm_bindgen(js_name = jsonOut)]
    pub fn json_out(data: &JsValue);
}

#[wasm_bindgen]
pub fn run(json: &JsValue) -> Result<(), JsValue> {
    // output panics to console.error
    console_error_panic_hook::set_once();
    let js_doc = get_js_doc(&json).unwrap();
    let bytes = match pdf_gen::pdf::create(&js_doc) {
        Ok(b) => b,
        Err(s) => return Err(JsValue::from_str(s)),
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
