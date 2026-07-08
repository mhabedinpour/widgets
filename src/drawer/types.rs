#[derive(Clone, Copy)]
pub enum Color {
    Rgb(u8, u8, u8),
}

impl Color {
    pub const WHITE: Color = Color::Rgb(255, 255, 255);
    pub const BLACK: Color = Color::Rgb(0, 0, 0);
}

#[derive(Clone, Copy)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

impl Point {
    pub fn new(x: u32, y: u32) -> Point {
        Point { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Size {
        Size { width, height }
    }
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(origin: Point, size: Size) -> Rect {
        Rect { origin, size }
    }
}
