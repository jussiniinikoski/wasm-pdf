mod canvas;
mod encoders;
mod font;
pub mod json;
mod models;
mod objects;
mod styles;
mod template;
mod text;
mod units;

use json::JsDocument;
use models::{Document, Image, Paragraph, Path, Spacer, Stationary, Table};
use template::PageTemplate;

/// Create PDF file from JSON input
pub fn create(js_doc: &JsDocument) -> Result<Vec<u8>, &'static str> {
    // add document content to template and build
    let mut template = PageTemplate::new(
        js_doc.template.size,
        js_doc.template.top,
        js_doc.template.left,
        js_doc.template.right,
        js_doc.template.bottom,
    );
    let mut doc = Document::new(&js_doc.title);
    // parse stationary elements
    for element in &js_doc.stationary {
        if let "pagenumber" = element.obj_type.to_lowercase().as_str() {
            let page_number = Stationary::page_number(&element);
            template.add_stationary(page_number);
        } else if let "text" = element.obj_type.to_lowercase().as_str() {
            let text = Stationary::text(&element);
            template.add_stationary(text);
        }
    }
    // parse contents of JSON Document
    for content in &js_doc.contents {
        match content.obj_type.to_lowercase().as_str() {
            "table" => {
                if let Some(table) = Table::from_content(&content, &js_doc) {
                    doc.add(Box::new(table));
                }
            }
            "image" => {
                if let Some(image) = Image::from_content(&content, &js_doc) {
                    doc.add(Box::new(image));
                }
            }
            "paragraph" => {
                let paragraph = Paragraph::from_content(&content);
                doc.add(Box::new(paragraph));
            }
            "spacer" => {
                let spacer = Spacer::from_content(&content);
                doc.add(Box::new(spacer));
            }
            "path" => {
                if let Some(path) = Path::from_content(&content) {
                    doc.add(Box::new(path));
                }
            }
            _ => (),
        }
    }
    // build document -> return bytes
    let bytes = template.build(&doc)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::create;
    use super::json::JsDocument;
    use serde_json;

    #[test]
    fn test_create() {
        let data = r#"
        {
            "title": "Example Document",
            "contents": [{
                    "obj_type": "Paragraph",
                    "params": {
                        "text": "Hello World!",
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
}
