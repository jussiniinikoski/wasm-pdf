use super::canvas::Canvas;
use super::models::{Document, Stationary};

#[derive(Debug, Clone)]
pub struct PageTemplate {
    page_size: (f32, f32),
    frame: Frame,
    stationary: Vec<Stationary>,
}

impl PageTemplate {
    pub fn new(
        size: (f32, f32),
        top_margin: f32,
        left_margin: f32,
        right_margin: f32,
        bottom_margin: f32,
    ) -> PageTemplate {
        let frame = Frame::new(
            left_margin,
            size.1 - top_margin,
            size.0 - left_margin - right_margin,
            size.1 - top_margin - bottom_margin,
        );
        PageTemplate {
            page_size: size,
            frame,
            stationary: Vec::new(),
        }
    }
    pub fn build(&self, doc: &Document) -> Result<Vec<u8>, &'static str> {
        let mut canvas = Canvas::new(&self);
        for element in doc.get_content() {
            element.draw(&mut canvas, self.frame.width)?;
        }
        canvas.build()
    }
    pub fn get_size(&self) -> (f32, f32) {
        self.page_size
    }
    pub fn get_frame(&self) -> Frame {
        self.frame
    }
    pub fn add_stationary(&mut self, object: Stationary) {
        self.stationary.push(object);
    }
    pub fn stationary(&self) -> Vec<Stationary> {
        self.stationary.clone()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Frame {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl Frame {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Frame {
        Frame {
            x,
            y,
            width,
            height,
        }
    }
    pub fn get_rect(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }
}
