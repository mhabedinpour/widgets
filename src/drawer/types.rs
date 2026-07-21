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

#[derive(Clone, Copy)]
pub struct ClearData {
    /// @default Color::BLACK
    pub color: Color,
}

/// Stroke alignment relative to the shape's boundary.
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum StrokeAlignment {
    /// Stroke drawn inside the shape boundary.
    Inside = 0,
    /// Stroke centered on the shape boundary (default).
    Center = 1,
    /// Stroke drawn outside the shape boundary.
    Outside = 2,
}

impl StrokeAlignment {
    pub fn from_int(v: u32) -> Self {
        match v {
            0 => Self::Inside,
            2 => Self::Outside,
            _ => Self::Center,
        }
    }
}

/// Built-in monospaced bitmap fonts (embedded-graphics `mono_font::ascii`).
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Font {
    Font4x6 = 0,
    Font5x7 = 1,
    Font5x8 = 2,
    Font6x9 = 3,
    /// Default font.
    Font6x10 = 4,
    Font6x12 = 5,
    Font6x13 = 6,
    Font6x13Bold = 7,
    Font6x13Italic = 8,
    Font7x13 = 9,
    Font7x13Bold = 10,
    Font7x13Italic = 11,
    Font7x14 = 12,
    Font7x14Bold = 13,
    Font8x13 = 14,
    Font8x13Bold = 15,
    Font8x13Italic = 16,
    Font9x15 = 17,
    Font9x15Bold = 18,
    Font9x18 = 19,
    Font9x18Bold = 20,
    Font10x20 = 21,
    // u8g2 fonts — rendered via the u8g2-fonts crate (smaller sizes not in mono_font)
    /// 3×5 px — smallest available. Uses u8g2_font_3x5im_mf.
    U8g2Font3x5 = 22,
    /// 4×6 px u8g2 variant. Uses u8g2_font_4x6_mf.
    U8g2Font4x6 = 23,
    /// 5×7 px u8g2 variant. Uses u8g2_font_5x7_mf.
    U8g2Font5x7 = 24,
    /// 5×8 px u8g2 variant. Uses u8g2_font_5x8_mf.
    U8g2Font5x8 = 25,
}

impl Font {
    pub fn from_int(v: u32) -> Self {
        if v <= 25 {
            // SAFETY: Font is #[repr(u32)] with variants 0..=25
            unsafe { core::mem::transmute(v) }
        } else {
            Self::Font6x10
        }
    }
}

/// Horizontal text alignment.
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum TextAlignment {
    Left = 0,
    Center = 1,
    Right = 2,
}

impl TextAlignment {
    pub fn from_int(v: u32) -> Self {
        match v {
            1 => Self::Center,
            2 => Self::Right,
            _ => Self::Left,
        }
    }
}

/// Vertical text baseline.
#[derive(Clone, Copy)]
#[repr(u32)]
pub enum Baseline {
    /// Bottom of descenders aligned to position (default).
    Alphabetic = 0,
    /// Top of EM box aligned to position.
    Top = 1,
    /// Middle of EM box aligned to position.
    Middle = 2,
    /// Bottom of EM box aligned to position.
    Bottom = 3,
}

impl Baseline {
    pub fn from_int(v: u32) -> Self {
        match v {
            1 => Self::Top,
            2 => Self::Middle,
            3 => Self::Bottom,
            _ => Self::Alphabetic,
        }
    }
}
