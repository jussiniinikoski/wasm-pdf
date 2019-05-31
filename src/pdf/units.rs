pub const INCH: f32 = 72.0;
pub const CM: f32 = INCH / 2.54;
pub const MM: f32 = CM * 0.1;

pub const A4: (f32, f32) = (210.0 * MM, 297.0 * MM);

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Line {
    pub x: f32,
    pub y: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Line {
    pub fn new(x: f32, y: f32, x2: f32, y2: f32) -> Line {
        Line { x, y, x2, y2 }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }
}
