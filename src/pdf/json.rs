use super::units::A4;
use std::collections::HashMap;

/// Parameter values from JSON
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsParamValue {
    Null,
    Boolean(bool),
    Text(String),
    Number(f32),                                // treat all numbers as floats
    Children(Vec<JsContent>),                   // child elements
    Object(Box<HashMap<String, JsParamValue>>), // nested parameters
    Array(Vec<JsParamValue>),
}

#[derive(Serialize, Deserialize)]
pub struct JsContent {
    #[serde(default = "default_obj_type")]
    pub obj_type: String,
    pub params: HashMap<String, JsParamValue>,
}

#[derive(Serialize, Deserialize)]
pub struct JsTemplate {
    #[serde(default = "default_template_size")]
    pub size: (f32, f32),
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Serialize, Deserialize)]
pub struct JsDocument {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_template")]
    pub template: JsTemplate,
    pub contents: Vec<JsContent>,
    #[serde(default = "default_image_data")]
    pub image_data: HashMap<String, String>,
    #[serde(default = "default_image_sizes")]
    pub image_widths: HashMap<String, f32>,
    #[serde(default = "default_image_sizes")]
    pub image_heights: HashMap<String, f32>,
}

fn default_title() -> String {
    "Untitled".to_string()
}

fn default_template() -> JsTemplate {
    JsTemplate {
        size: A4,
        top: 50.0,
        left: 50.0,
        bottom: 50.0,
        right: 50.0,
    }
}

fn default_template_size() -> (f32, f32) {
    A4
}

fn default_obj_type() -> String {
    "Paragraph".to_string()
}

fn default_image_data() -> HashMap<String, String> {
    HashMap::new()
}

fn default_image_sizes() -> HashMap<String, f32> {
    HashMap::new()
}

pub fn get_number_from_js(value: Option<&JsParamValue>, default: f32) -> f32 {
    if let Some(value) = value {
        match value {
            JsParamValue::Number(i) => *i,
            _ => default,
        }
    } else {
        default
    }
}

pub fn get_text_from_js(value: Option<&JsParamValue>, default: &str) -> String {
    if let Some(value) = value {
        match value {
            JsParamValue::Text(t) => t.clone(),
            _ => String::from(default),
        }
    } else {
        String::from(default)
    }
}

pub fn get_bool_from_js(value: Option<&JsParamValue>, default: bool) -> bool {
    if let Some(value) = value {
        match value {
            JsParamValue::Boolean(b) => *b,
            _ => default,
        }
    } else {
        default
    }
}
