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
    pub top: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Serialize, Deserialize)]
pub struct JsDocument {
    pub title: String,
    pub template: JsTemplate,
    pub contents: Vec<JsContent>,
    #[serde(default = "default_image_data")]
    pub image_data: HashMap<String, String>,
    #[serde(default = "default_image_sizes")]
    pub image_widths: HashMap<String, f32>,
    #[serde(default = "default_image_sizes")]
    pub image_heights: HashMap<String, f32>,
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
