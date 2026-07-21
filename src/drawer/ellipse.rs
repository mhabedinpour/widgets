use crate::drawer::{Color, Drawer, Rect, StrokeAlignment};

/// @wasm required="bounding_box"
#[derive(Clone, Copy)]
pub struct EllipseData {
    pub bounding_box: Rect,
    /// @default Color::WHITE
    pub fill_color: Color,
    /// @default true
    pub fill: bool,
    /// @default Color::WHITE
    pub stroke_color: Color,
    /// @default 0
    pub stroke_width: u32,
    /// @default StrokeAlignment::Center
    pub stroke_alignment: StrokeAlignment,
}

pub struct EllipseBuilder {
    pub data: EllipseData,
}

impl EllipseBuilder {
    #[inline(always)]
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        use crate::drawer::{Point, Size};
        Self {
            data: EllipseData {
                bounding_box: Rect::new(Point { x, y }, Size { width, height }),
                fill_color: Color::WHITE,
                fill: true,
                stroke_color: Color::WHITE,
                stroke_width: 0,
                stroke_alignment: StrokeAlignment::Center,
            },
        }
    }

    /// Sets the fill color. Enables fill if it was disabled.
    #[inline(always)]
    pub fn color(mut self, color: Color) -> Self {
        self.data.fill_color = color;
        self.data.fill = true;
        self
    }

    /// Sets the fill color explicitly.
    #[inline(always)]
    pub fn fill_color(mut self, color: Color) -> Self {
        self.data.fill_color = color;
        self.data.fill = true;
        self
    }

    /// Disables fill, leaving only stroke visible.
    #[inline(always)]
    pub fn no_fill(mut self) -> Self {
        self.data.fill = false;
        self
    }

    /// Sets the stroke width. Backward-compat: also disables fill.
    #[inline(always)]
    pub fn stroke(mut self, width: u32) -> Self {
        self.data.fill = false;
        self.data.stroke_width = width;
        self
    }

    /// Sets the stroke color independently of fill.
    #[inline(always)]
    pub fn stroke_color(mut self, color: Color) -> Self {
        self.data.stroke_color = color;
        self
    }

    /// Sets stroke width without affecting fill.
    #[inline(always)]
    pub fn stroke_width(mut self, width: u32) -> Self {
        self.data.stroke_width = width;
        self
    }

    /// Sets stroke alignment (Inside / Center / Outside).
    #[inline(always)]
    pub fn stroke_alignment(mut self, alignment: StrokeAlignment) -> Self {
        self.data.stroke_alignment = alignment;
        self
    }

    #[inline(always)]
    pub fn draw(self, drawer: &mut dyn Drawer) {
        drawer.execute_ellipse(self.data);
    }
}
