use criterion::{criterion_group, criterion_main, Criterion};
extern crate wasm_pdf_gen;
use serde_json;
use wasm_pdf_gen::pdf::create;
use wasm_pdf_gen::pdf::json::JsDocument;

fn run_create() {
    let data = r#"
        {
            "title": "Example Document",
            "contents": [{
                    "obj_type": "Paragraph",
                    "params": {
                        "text": "Hello World! <b>This is WASM-PDF.</b> <a href='https://github.com/jussiniinikoski/wasm-pdf'>Read More</a>",
                        "font_size": 18,
                        "leading": 24,
                        "align": "center",
                        "font_name": "Helvetica-Bold"
                    }
                }
            ]
        }"#;
    let js_doc: JsDocument = serde_json::from_str(data).unwrap();
    let bytes = match create(&js_doc) {
        Ok(b) => b,
        Err(s) => format!("{}", s).into(),
    };
    assert!(bytes.starts_with(b"%PDF-1.4\n%\x93\x8C\x8B\x9E WASM-PDF library\n"));
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("pdf::create", |b| b.iter(|| run_create()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
