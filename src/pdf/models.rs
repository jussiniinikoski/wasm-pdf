#![allow(dead_code)]
use wasm_bindgen::prelude::*;

use super::canvas::Canvas;
use super::font::{
    courier, courier_bold, courier_bold_oblique, courier_oblique, helvetica, helvetica_bold,
    helvetica_bold_oblique, helvetica_oblique, times_bold, times_bold_italic, times_italic,
    times_roman, Font,
};
use super::styles::{CellStyle, TableStyle, ParagraphStyle};
use super::text::Text;

// Content Trait is the center piece here.
pub trait Content {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), JsValue>;
    // wrap element, takes available width, height and returns actual width, height
    fn wrap(&self, area: (f32, f32)) -> (f32, f32);
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
    pub style: ParagraphStyle
}

impl Paragraph {
    pub fn new(
        text: &str,
        font_name: &str,
        font_size: f32,
        style: ParagraphStyle
    ) -> Paragraph {
        Paragraph {
            text: String::from(text),
            font_size,
            font: match font_name.to_lowercase().as_str() {
                "helvetica" => helvetica(),
                "courier" => courier(),
                "times" => times_roman(),
                "helvetica-bold" => helvetica_bold(),
                "helvetica-oblique" => helvetica_oblique(),
                "helvetica-bold-oblique" => helvetica_bold_oblique(),
                "courier-bold" => courier_bold(),
                "courier-oblique" => courier_oblique(),
                "courier-bold-oblique" => courier_bold_oblique(),
                "times-bold" => times_bold(),
                "times-italic" => times_italic(),
                "times-bold-italic" => times_bold_italic(),
                _ => helvetica(), // default font
            },
            style
        }
    }
}

impl Content for Paragraph {
    fn draw(&self, canvas: &mut Canvas, available_width: f32) -> Result<(), JsValue> {
        canvas.draw_text(&self, &self.text, available_width)
    }
    fn wrap(&self, area: (f32, f32)) -> (f32, f32) {
        let (encoded_lines, _text_lines) = Text::get_text_lines(&self, &self.text, area.0);
        let height = self.style.leading * encoded_lines.len() as f32;
        let width = Text::get_text_width(&self, &self.text, area.0);
        (width, height)
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
}

pub struct Image {
    pub data: Vec<u8>,
    pub width: f32,
    pub height: f32,
    pub fit_width: bool,
}

impl Image {
    pub fn new(data: Vec<u8>, width: f32, height: f32, fit_width: bool) -> Image {
        Image {
            data,
            width,
            height,
            fit_width,
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
}
