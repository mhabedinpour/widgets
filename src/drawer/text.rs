use alloc::string::String;
use crate::drawer::{Color, Drawer, Point};

/// @wasm required="text,position"
#[derive(Clone)]
pub struct TextData {
    pub text: String,
    pub position: Point,
    /// @default Color::WHITE
    pub color: Color,
}

pub struct TextBuilder {
    pub data: TextData,
}

impl<'a> TextBuilder {
    #[inline(always)]
    pub fn new(text: String, x: u32, y: u32) -> Self {
        Self {
            data: TextData {
                text,
                position: Point { x, y },
                color: Color::WHITE,
            },
        }
    }

    #[inline(always)]
    pub fn color(mut self, color: Color) -> Self {
        self.data.color = color;
        self
    }

    #[inline(always)]
    pub fn draw(self, drawer: &mut dyn Drawer) {
        drawer.execute_text(self.data);
    }
}
