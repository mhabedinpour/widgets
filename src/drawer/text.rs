use crate::drawer::{Baseline, Color, Drawer, Font, Point, TextAlignment};
use alloc::string::String;

/// @wasm required="text,position"
#[derive(Clone)]
pub struct TextData {
    pub text: String,
    pub position: Point,
    /// @default Color::WHITE
    pub color: Color,
    /// @default Color::BLACK
    pub background_color: Color,
    /// @default false
    pub has_background: bool,
    /// @default Font::Font6x10
    pub font: Font,
    /// @default false
    pub underline: bool,
    /// @default false
    pub strikethrough: bool,
    /// @default TextAlignment::Left
    pub alignment: TextAlignment,
    /// @default Baseline::Alphabetic
    pub baseline: Baseline,
}

pub struct TextBuilder {
    pub data: TextData,
}

impl TextBuilder {
    #[inline(always)]
    pub fn new(text: String, x: u32, y: u32) -> Self {
        Self {
            data: TextData {
                text,
                position: Point { x, y },
                color: Color::WHITE,
                background_color: Color::BLACK,
                has_background: false,
                font: Font::Font6x10,
                underline: false,
                strikethrough: false,
                alignment: TextAlignment::Left,
                baseline: Baseline::Alphabetic,
            },
        }
    }

    /// Sets the text (foreground) color.
    #[inline(always)]
    pub fn color(mut self, color: Color) -> Self {
        self.data.color = color;
        self
    }

    /// Sets a background fill color behind the glyphs.
    #[inline(always)]
    pub fn background_color(mut self, color: Color) -> Self {
        self.data.background_color = color;
        self.data.has_background = true;
        self
    }

    /// Selects the bitmap font.
    #[inline(always)]
    pub fn font(mut self, font: Font) -> Self {
        self.data.font = font;
        self
    }

    /// Enables underline decoration (uses text color).
    #[inline(always)]
    pub fn underline(mut self) -> Self {
        self.data.underline = true;
        self
    }

    /// Enables strikethrough decoration (uses text color).
    #[inline(always)]
    pub fn strikethrough(mut self) -> Self {
        self.data.strikethrough = true;
        self
    }

    /// Sets horizontal text alignment (Left / Center / Right).
    #[inline(always)]
    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.data.alignment = alignment;
        self
    }

    /// Sets the vertical baseline anchor (Alphabetic / Top / Middle / Bottom).
    #[inline(always)]
    pub fn baseline(mut self, baseline: Baseline) -> Self {
        self.data.baseline = baseline;
        self
    }

    #[inline(always)]
    pub fn draw(self, drawer: &mut dyn Drawer) {
        drawer.execute_text(self.data);
    }
}
