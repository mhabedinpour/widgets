use crate::drawer::{Color, Drawer, Point};

/// @wasm required="text,position"
#[derive(Clone, Copy)]
pub struct TextData<'a> {
    pub text: &'a str,
    pub position: Point,
    /// @default Color::WHITE
    pub color: Color,
}

pub struct TextBuilder<'a> {
    pub data: TextData<'a>,
}

impl<'a> TextBuilder<'a> {
    #[inline(always)]
    pub fn new(text: &'a str, x: u32, y: u32) -> Self {
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
