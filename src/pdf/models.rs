#![allow(dead_code)]
use super::canvas::Canvas;
use super::font::{get_font, Font};
use super::styles::{
    CellStyle, HorizontalAlign, ImageStyle, ParagraphStyle, PathStyle, TableStyle,
};
use super::text::Text;
use super::units::{Color, Point};
use wasm_bindgen::prelude::*;

pub enum ContentType {
    Paragraph,
    Image,
    Spacer,
    Table,
    Path,
}

// Content Trait is the center piece here.
pub trait Content {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), JsValue>;
    // wrap element, takes available width, height and returns actual width, height
    fn wrap(&self, area: (f32, f32)) -> (f32, f32);
    // define content type
    fn content_type(&self) -> ContentType;
}

// Using enums instead of structs/trait objects, since the amount of different stationary
// elements will remain low. Stationary elements are also simpler than "Content" objects.
#[derive(Debug, Copy, Clone)]
pub enum Stationary {
    PageNumber {
        font_size: f32,
        font: &'static Font,
        x: f32,
        y: f32,
        align: HorizontalAlign,
    },
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
    pub text: String,
    pub font_size: f32,
    pub font: &'static Font,
    pub style: ParagraphStyle,
}

impl Paragraph {
    pub fn new(text: &str, font_name: &str, font_size: f32, style: ParagraphStyle) -> Paragraph {
        Paragraph {
            text: String::from(text),
            font_size,
            font: get_font(font_name.to_lowercase().as_str()),
            style,
        }
    }
}

impl Content for Paragraph {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), JsValue> {
        canvas.draw_text(&self, &self.text, available_width)
    }
    fn wrap(&self, area: (f32, f32)) -> (f32, f32) {
        let (encoded_lines, _text_lines) = Text::get_text_lines(&self, &self.text, area.0);
        let vertical_padding = self.style.padding.0 + self.style.padding.2;
        let height = self.style.leading * encoded_lines.len() as f32 + vertical_padding;
        let width = Text::get_text_width(&self, &self.text, area.0);
        (width, height)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Paragraph
    }
}

pub struct Spacer {
    pub width: f32,
    pub height: f32,
}

impl Spacer {
    pub fn new(width: f32, height: f32) -> Spacer {
        Spacer { width, height }
    }
}

impl Content for Spacer {
    fn draw(&self, canvas: &mut Canvas, _available_width: f32) -> Result<(), JsValue> {
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
    pub data: Vec<u8>,
    pub width: f32,
    pub height: f32,
    pub fit_width: bool,
    pub style: ImageStyle,
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
}

impl Content for Image {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), JsValue> {
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
    pub contents: Vec<Box<dyn Content>>,
    pub span: f32,
    pub style: CellStyle,
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
}

pub struct Row {
    pub cells: Vec<Cell>,
}

impl Row {
    pub fn new() -> Row {
        Row { cells: Vec::new() }
    }
    pub fn add_cell(&mut self, cell: Cell) {
        self.cells.push(cell);
    }
}

pub struct Table {
    pub rows: Vec<Row>,
    pub style: TableStyle,
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
}

impl Content for Table {
    fn draw(&self, canvas: &mut Canvas, _available_width: f32) -> Result<(), JsValue> {
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
    pub points: Vec<Point>,
    pub stroke_color: Option<Color>,
    pub stroke_width: f32,
    pub fill_color: Option<Color>,
    pub width: f32,
    pub height: f32,
    pub style: PathStyle,
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
}

impl Content for Path {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), JsValue> {
        canvas.draw_path(&self, available_width)
    }
    fn wrap(&self, _area: (f32, f32)) -> (f32, f32) {
        (self.width, self.height)
    }
    fn content_type(&self) -> ContentType {
        ContentType::Path
    }
}
