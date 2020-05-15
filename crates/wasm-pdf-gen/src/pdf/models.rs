#![allow(dead_code)]
use super::canvas::Canvas;
use super::font::{get_font, Font};
use super::styles::{CellStyle, Color, ImageStyle, ParagraphStyle, PathStyle, TableStyle};
use super::text::TextSpan;
use super::units::Point;

use super::json::{
    get_bool_from_js, get_number_from_js, get_text_from_js, JsContent, JsDocument, JsParamValue,
};

pub enum ContentType {
    Paragraph,
    Image,
    Spacer,
    Table,
    Path,
}

// Content Trait is the center piece here.
pub trait Content {
    // draw element to canvas with available width
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), &'static str>;
    // wrap element, takes available width, height and returns actual width, height
    fn wrap(&self, area: (f32, f32)) -> (f32, f32);
    // define content type
    fn content_type(&self) -> ContentType;
}

// Using enums instead of structs/trait objects, since the amount of different stationary
// elements will remain low. Stationary elements are also simpler than "Content" objects.
#[derive(Debug, Clone)]
pub enum Stationary {
    PageNumber {
        font_size: f32,
        font: &'static Font,
        x: f32,
        y: f32,
        color: Color,
    },
    Text {
        text: String,
        font_size: f32,
        font: &'static Font,
        x: f32,
        y: f32,
        color: Color,
    },
}

impl Stationary {
    pub fn page_number(content: &JsContent) -> Stationary {
        let p_font_name = get_text_from_js(content.params.get("font_name"), "Helvetica");
        let font_size = get_number_from_js(content.params.get("font_size"), 12.0);
        let x = get_number_from_js(content.params.get("x"), 50.0);
        let y = get_number_from_js(content.params.get("y"), 50.0);
        let font = get_font(p_font_name.to_lowercase().as_str());
        let color =
            Color::from_param_or_default(content.params.get("color"), Color::new(0.0, 0.0, 0.0));
        Stationary::PageNumber {
            font,
            font_size,
            x,
            y,
            color,
        }
    }
    pub fn text(content: &JsContent) -> Stationary {
        let text = get_text_from_js(content.params.get("text"), "");
        let p_font_name = get_text_from_js(content.params.get("font_name"), "Helvetica");
        let font_size = get_number_from_js(content.params.get("font_size"), 12.0);
        let x = get_number_from_js(content.params.get("x"), 50.0);
        let y = get_number_from_js(content.params.get("y"), 50.0);
        let font = get_font(p_font_name.to_lowercase().as_str());
        let color =
            Color::from_param_or_default(content.params.get("color"), Color::new(0.0, 0.0, 0.0));
        Stationary::Text {
            text,
            font,
            font_size,
            x,
            y,
            color,
        }
    }
}

pub struct Document {
    title: String,
    content: Vec<Box<dyn Content>>,
}

impl Document {
    pub fn new(title: &str) -> Document {
        Document {
            title: String::from(title),
            content: Vec::new(),
        }
    }
    pub fn add(&mut self, object: Box<dyn Content>) {
        self.content.push(object);
    }
    pub fn get_content(&self) -> &Vec<Box<dyn Content>> {
        &self.content
    }
}

pub struct Paragraph {
    font_size: f32,
    font: &'static Font,
    style: ParagraphStyle,
    spans: Vec<TextSpan>,
}

impl Paragraph {
    pub fn new(text: &str, font_name: &str, font_size: f32, style: ParagraphStyle) -> Paragraph {
        let text_spans = TextSpan::extract_spans(&text);
        Paragraph {
            font_size,
            font: get_font(font_name.to_lowercase().as_str()),
            style,
            spans: text_spans,
        }
    }
    pub fn get_font_size(&self) -> f32 {
        self.font_size
    }
    pub fn get_font(&self) -> &'static Font {
        self.font
    }
    pub fn get_style(&self) -> &ParagraphStyle {
        &self.style
    }
    pub fn get_spans(&self) -> &Vec<TextSpan> {
        &self.spans
    }
    pub fn from_content(content: &JsContent) -> Paragraph {
        let p_font_name = get_text_from_js(content.params.get("font_name"), "Helvetica");
        let p_font_size = get_number_from_js(content.params.get("font_size"), 12.0);
        let p_style = ParagraphStyle::from_content(&content, p_font_size);
        let text_value = get_text_from_js(content.params.get("text"), "");
        Paragraph::new(&text_value, &p_font_name, p_font_size, p_style)
    }

    /// Generate wrapped text spans, a line may contain multiple spans
    /// and a span may split to next line. This is NOT optimal, but it works..
    pub fn wrap_to_width(&self, available_width: f32) -> Vec<Vec<TextSpan>> {
        let available_width = if !self.style.wrap {
            f32::MAX
        } else {
            available_width
        };
        let font = self.font;
        let size = self.font_size;
        // contain lines of lines of spans
        let mut wrapped: Vec<Vec<TextSpan>> = Vec::new();
        // contains line of spans
        let mut line_spans: Vec<TextSpan> = Vec::new();
        // line of text
        let mut line = String::new();
        let num_spans = self.get_spans().len();
        for (i, span) in self.get_spans().iter().enumerate() {
            let words: Vec<&str> = span.text.split_whitespace().collect();
            let mut next_word: Option<String> = None;
            let mut span_text = String::new();
            for word in words {
                if let Some(_next_word) = next_word {
                    line += &_next_word;
                    line += " ";
                    span_text += &_next_word;
                    span_text += " ";
                    next_word = None;
                }
                line += word;
                line += " ";
                let current_width = font.get_width(size, &line.trim_end());
                if current_width > available_width {
                    next_word = Some(word.to_string());
                    if !span_text.is_empty() {
                        let text_span = TextSpan::new(&span_text.trim_end(), span.tag.clone());
                        line_spans.push(text_span);
                        wrapped.push(line_spans);
                    }
                    line = String::new();
                    line_spans = Vec::new();
                    span_text = String::new();
                } else {
                    span_text += word;
                    span_text += " ";
                }
            }
            if !span_text.is_empty() || next_word != None {
                if let Some(_next_word) = next_word {
                    span_text += &_next_word;
                }
                if i == num_spans - 1 {
                    // Remove trailing space from last span
                    span_text = span_text.trim_end().to_owned();
                }
                let text_span = TextSpan::new(&span_text, span.tag.clone());
                line_spans.push(text_span);
            }
        }
        if !line_spans.is_empty() {
            wrapped.push(line_spans);
        }
        wrapped
    }

    pub fn wrapped_size(&self, wrapped: &[Vec<TextSpan>]) -> (f32, f32) {
        let vertical_padding = self.style.padding.0 + self.style.padding.2;
        if !self.style.wrap {
            return (f32::MAX, self.style.leading + vertical_padding);
        }
        let mut width: f32 = 0.0;
        for line in wrapped {
            let mut max_line: f32 = 0.0;
            for span in line {
                max_line += span.get_width(&self.font, self.font_size);
            }
            if width < max_line {
                width = max_line;
            }
        }
        let height = self.style.leading * wrapped.len() as f32 + vertical_padding;
        (width, height)
    }
}

impl Content for Paragraph {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), &'static str> {
        let padding_left = self.style.padding.1;
        let padding_right = self.style.padding.3;
        let horizontal_padding = padding_left + padding_right;
        let bullet_indent = self.style.bullet_indent;
        let available_width = available_width - horizontal_padding - bullet_indent;
        let wrapped = self.wrap_to_width(available_width);
        canvas.draw_text(&self, &wrapped, available_width)
    }
    fn wrap(&self, area: (f32, f32)) -> (f32, f32) {
        // Calculate width and height according to wrapped
        let wrapped = self.wrap_to_width(area.0);
        self.wrapped_size(&wrapped)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Paragraph
    }
}

pub struct Spacer {
    width: f32,
    height: f32,
}

impl Spacer {
    pub fn new(width: f32, height: f32) -> Spacer {
        Spacer { width, height }
    }
    pub fn from_content(content: &JsContent) -> Spacer {
        let p_width = get_number_from_js(content.params.get("width"), 0.0);
        let p_height = get_number_from_js(content.params.get("height"), 0.0);
        Spacer::new(p_width, p_height)
    }
    pub fn get_dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}

impl Content for Spacer {
    fn draw(&self, canvas: &mut Canvas, _available_width: f32) -> Result<(), &'static str> {
        canvas.draw_spacer(&self)
    }
    fn wrap(&self, area: (f32, f32)) -> (f32, f32) {
        (area.0, self.height)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Spacer
    }
}

pub struct Image {
    data: Vec<u8>,
    width: f32,
    height: f32,
    fit_width: bool,
    style: ImageStyle,
}

impl Image {
    pub fn new(
        data: Vec<u8>,
        width: f32,
        height: f32,
        fit_width: bool,
        style: ImageStyle,
    ) -> Image {
        Image {
            data,
            width,
            height,
            fit_width,
            style,
        }
    }
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
    pub fn get_dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }
    pub fn fits_width(&self) -> bool {
        self.fit_width
    }
    pub fn get_style(&self) -> &ImageStyle {
        &self.style
    }
    pub fn from_content(content: &JsContent, js_doc: &JsDocument) -> Option<Image> {
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
                    let image_style = ImageStyle::from_content(&content);
                    let image = Image::new(image_data, p_width, p_height, fit_width, image_style);
                    return Some(image);
                }
            }
        }
        None
    }
}

impl Content for Image {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), &'static str> {
        canvas.draw_image(&self, false, available_width)
    }
    fn wrap(&self, area: (f32, f32)) -> (f32, f32) {
        let width = if self.fit_width || self.width > area.0 {
            area.0
        } else {
            self.width
        };
        let height = if self.fit_width {
            area.0 / self.width * self.height
        } else {
            self.height
        };
        (width, height)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Image
    }
}

pub struct Cell {
    contents: Vec<Box<dyn Content>>,
    span: f32,
    style: CellStyle,
}

impl Cell {
    pub fn new(span: f32) -> Cell {
        Cell {
            contents: Vec::new(),
            span,
            style: CellStyle::new(),
        }
    }
    pub fn add(&mut self, object: Box<dyn Content>) {
        self.contents.push(object);
    }
    pub fn get_contents(&self) -> &Vec<Box<dyn Content>> {
        &self.contents
    }
    pub fn get_span(&self) -> f32 {
        self.span
    }
    pub fn get_style(&self) -> &CellStyle {
        &self.style
    }
}

pub struct Row {
    cells: Vec<Cell>,
}

impl Row {
    pub fn new() -> Row {
        Row { cells: Vec::new() }
    }
    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }
    pub fn get_cells(&self) -> &Vec<Cell> {
        &self.cells
    }
}

pub struct Table {
    rows: Vec<Row>,
    style: TableStyle,
}

impl Table {
    pub fn new(style: TableStyle) -> Table {
        Table {
            rows: Vec::new(),
            style,
        }
    }
    pub fn add_row(&mut self, row: Row) {
        self.rows.push(row);
    }
    pub fn get_rows(&self) -> &Vec<Row> {
        &self.rows
    }
    pub fn get_style(&self) -> &TableStyle {
        &self.style
    }
    pub fn from_content(content: &JsContent, js_doc: &JsDocument) -> Option<Table> {
        let table_style = TableStyle::from_content(content);
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
                                                    let paragraph =
                                                        Paragraph::from_content(&cell_content);
                                                    c.add(Box::new(paragraph));
                                                }
                                                "image" => {
                                                    if let Some(image) =
                                                        Image::from_content(&cell_content, &js_doc)
                                                    {
                                                        c.add(Box::new(image));
                                                    }
                                                }
                                                "path" => {
                                                    if let Some(path) =
                                                        Path::from_content(&cell_content)
                                                    {
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
                                            c.style.background_color = Color::from_param(bg_color);
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
        Some(table)
    }
}

impl Content for Table {
    fn draw(&self, canvas: &mut Canvas, _available_width: f32) -> Result<(), &'static str> {
        canvas.draw_table(&self)
    }
    fn wrap(&self, area: (f32, f32)) -> (f32, f32) {
        // table is just a placeholder for keeping rows
        (area.0, area.1)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Table
    }
}

pub struct Path {
    points: Vec<Point>,
    stroke_color: Option<Color>,
    stroke_width: f32,
    fill_color: Option<Color>,
    width: f32,
    height: f32,
    style: PathStyle,
}

impl Path {
    pub fn new(
        points: Vec<Point>,
        stroke_color: Option<Color>,
        stroke_width: f32,
        fill_color: Option<Color>,
        style: PathStyle,
    ) -> Path {
        let min_x = points.iter().fold(std::f32::MAX, |acc, b| acc.min(b.x));
        let max_x = points.iter().fold(std::f32::MIN, |acc, b| acc.max(b.x));
        let width = max_x - min_x;
        let min_y = points.iter().fold(std::f32::MAX, |acc, b| acc.min(b.y));
        let max_y = points.iter().fold(std::f32::MIN, |acc, b| acc.max(b.y));
        let height = max_y - min_y;
        Path {
            points,
            stroke_color,
            stroke_width,
            fill_color,
            width,
            height,
            style,
        }
    }
    pub fn get_points(&self) -> &Vec<Point> {
        &self.points
    }
    pub fn get_stroke_color(&self) -> Option<Color> {
        self.stroke_color
    }
    pub fn get_stroke_width(&self) -> f32 {
        self.stroke_width
    }
    pub fn get_fill_color(&self) -> Option<Color> {
        self.fill_color
    }
    pub fn get_width(&self) -> f32 {
        self.width
    }
    pub fn get_height(&self) -> f32 {
        self.height
    }
    pub fn get_style(&self) -> &PathStyle {
        &self.style
    }
    pub fn from_content(content: &JsContent) -> Option<Path> {
        let stroke_color = if let Some(color) = content.params.get("stroke_color") {
            Color::from_param(color)
        } else {
            None
        };
        let fill_color = if let Some(color) = content.params.get("fill_color") {
            Color::from_param(color)
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
                    let style = PathStyle::from_content(&content);
                    let path = Path::new(points, stroke_color, stroke_width, fill_color, style);
                    return Some(path);
                }
            }
        }
        None
    }
}

impl Content for Path {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), &'static str> {
        canvas.draw_path(&self, available_width)
    }
    fn wrap(&self, _area: (f32, f32)) -> (f32, f32) {
        (self.width, self.height)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Path
    }
}
