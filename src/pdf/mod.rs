use wasm_bindgen::prelude::*;

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

use font::get_font;
use json::{
    get_bool_from_js, get_number_from_js, get_text_from_js, JsContent, JsDocument, JsParamValue,
};
use models::{Cell, Document, Image, Paragraph, Path, Row, Spacer, Stationary, Table};
use styles::{
    get_color, get_color_from_js, get_image_style, get_paragraph_style, get_path_style,
    get_table_style,
};
use template::PageTemplate;
use units::{Color, Point};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(msg: &str);
    #[wasm_bindgen(js_name = jsonOut)]
    pub fn json_out(data: &JsValue);
}

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
            let page_number = get_page_number(&element);
            template.add_stationary(page_number);
        } else if let "text" = element.obj_type.to_lowercase().as_str() {
            let text = get_text_line(&element);
            template.add_stationary(text);
        }
    }
    // parse contents of JSON Document
    for content in &js_doc.contents {
        match content.obj_type.to_lowercase().as_str() {
            "table" => {
                let table = get_table(&content, js_doc)?;
                doc.add(Box::new(table));
            }
            "image" => {
                if let Some(image) = get_image(&content, &js_doc) {
                    doc.add(Box::new(image));
                }
            }
            "paragraph" => {
                let paragraph = get_paragraph(&content);
                doc.add(Box::new(paragraph));
            }
            "spacer" => {
                let spacer = get_spacer(&content);
                doc.add(Box::new(spacer));
            }
            "path" => {
                if let Some(path) = get_path(&content) {
                    doc.add(Box::new(path));
                }
            }
            _ => (),
        }
    }
    // build document -> return bytes
    template.build(&doc)
}

fn get_table(content: &JsContent, js_doc: &JsDocument) -> Result<Table, &'static str> {
    let table_style = get_table_style(content);
    let mut table = Table::new(table_style);
    if let Some(rows) = content.params.get("rows") {
        if let JsParamValue::Children(rows) = rows {
            for row in rows {
                let mut r = Row::new();
                if let Some(cells) = row.params.get("cells") {
                    if let JsParamValue::Children(cells) = cells {
                        //log(&format!("number of cells: {}", cells.len()));
                        for cell in cells {
                            let cell_span = get_number_from_js(cell.params.get("span"), 1.0);
                            let mut c = Cell::new(cell_span);
                            if let Some(cell_contents) = cell.params.get("contents") {
                                if let JsParamValue::Children(contents) = cell_contents {
                                    for cell_content in contents {
                                        match cell_content.obj_type.to_lowercase().as_str() {
                                            "paragraph" => {
                                                let paragraph = get_paragraph(&cell_content);
                                                c.add(Box::new(paragraph));
                                            }
                                            "image" => {
                                                if let Some(image) =
                                                    get_image(&cell_content, &js_doc)
                                                {
                                                    c.add(Box::new(image));
                                                }
                                            }
                                            "path" => {
                                                if let Some(path) = get_path(&cell_content) {
                                                    c.add(Box::new(path));
                                                }
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                            }
                            if let Some(cell_style) = cell.params.get("style") {
                                if let JsParamValue::Object(cell_style) = cell_style {
                                    if let Some(bg_color) = cell_style.get("background_color") {
                                        c.style.background_color = get_color(bg_color);
                                    }
                                }
                            }
                            r.add_cell(c);
                        }
                    }
                }
                table.add_row(r);
            }
        }
    }
    Ok(table)
}

fn get_image(content: &JsContent, js_doc: &JsDocument) -> Option<Image> {
    let fit_width = get_bool_from_js(content.params.get("fit_width"), false);
    if let Some(src) = content.params.get("src") {
        if let JsParamValue::Text(s) = src {
            if let Some(image_data_str) = js_doc.image_data.get(s) {
                let p_width = if let Some(width) = js_doc.image_widths.get(s) {
                    *width
                } else {
                    0.0
                };
                let p_height = if let Some(height) = js_doc.image_heights.get(s) {
                    *height
                } else {
                    0.0
                };
                let image_data = base64::decode(&image_data_str).unwrap();
                let image_style = get_image_style(&content);
                let image = Image::new(image_data, p_width, p_height, fit_width, image_style);
                return Some(image);
            }
        }
    }
    None
}

fn get_path(content: &JsContent) -> Option<Path> {
    let stroke_color = if let Some(color) = content.params.get("stroke_color") {
        get_color(color)
    } else {
        None
    };
    let fill_color = if let Some(color) = content.params.get("fill_color") {
        get_color(color)
    } else {
        None
    };
    let stroke_width = get_number_from_js(content.params.get("stroke_width"), 0.0);
    if let Some(points) = content.params.get("points") {
        if let JsParamValue::Array(js_points) = points {
            let mut points: Vec<Point> = Vec::new();
            for point in js_points {
                if let JsParamValue::Array(js_point) = point {
                    if let JsParamValue::Number(x) = js_point[0] {
                        if let JsParamValue::Number(y) = js_point[1] {
                            let p = Point { x, y };
                            points.push(p);
                        }
                    }
                }
            }
            if points.len() > 1 {
                let style = get_path_style(&content);
                let path = Path::new(points, stroke_color, stroke_width, fill_color, style);
                return Some(path);
            }
        }
    }
    None
}

fn get_spacer(content: &JsContent) -> Spacer {
    let p_width = get_number_from_js(content.params.get("width"), 0.0);
    let p_height = get_number_from_js(content.params.get("height"), 0.0);
    Spacer::new(p_width, p_height)
}

fn get_paragraph(content: &JsContent) -> Paragraph {
    let p_font_name = get_text_from_js(content.params.get("font_name"), "Helvetica");
    let p_font_size = get_number_from_js(content.params.get("font_size"), 12.0);
    let p_style = get_paragraph_style(&content, p_font_size);
    let text_value = get_text_from_js(content.params.get("text"), "");
    Paragraph::new(&text_value, &p_font_name, p_font_size, p_style)
}

fn get_page_number(content: &JsContent) -> Stationary {
    let p_font_name = get_text_from_js(content.params.get("font_name"), "Helvetica");
    let font_size = get_number_from_js(content.params.get("font_size"), 12.0);
    let x = get_number_from_js(content.params.get("x"), 50.0);
    let y = get_number_from_js(content.params.get("y"), 50.0);
    let font = get_font(p_font_name.to_lowercase().as_str());
    let color = get_color_from_js(content.params.get("color"), Color::new(0.0, 0.0, 0.0));
    Stationary::PageNumber {
        font,
        font_size,
        x,
        y,
        color,
    }
}

fn get_text_line(content: &JsContent) -> Stationary {
    let text = get_text_from_js(content.params.get("text"), "");
    let p_font_name = get_text_from_js(content.params.get("font_name"), "Helvetica");
    let font_size = get_number_from_js(content.params.get("font_size"), 12.0);
    let x = get_number_from_js(content.params.get("x"), 50.0);
    let y = get_number_from_js(content.params.get("y"), 50.0);
    let font = get_font(p_font_name.to_lowercase().as_str());
    let color = get_color_from_js(content.params.get("color"), Color::new(0.0, 0.0, 0.0));
    Stationary::Text {
        text,
        font,
        font_size,
        x,
        y,
        color,
    }
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
