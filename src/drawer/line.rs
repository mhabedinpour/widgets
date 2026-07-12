use crate::drawer::{Color, Drawer, Point};

/// @wasm module="drawer_backend" fn="draw_line" executor="execute_line" required="start,end"
#[derive(Clone, Copy)]
pub struct LineData {
    pub start: Point,
    pub end: Point,
    /// @default Color::WHITE
    pub color: Color,
    /// @default 1
    pub thickness: u32,
}

pub struct LineBuilder {
    pub data: LineData,
}

impl LineBuilder {
    #[inline(always)]
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            data: LineData {
                start,
                end,
                color: Color::WHITE,
                thickness: 1,
            },
        }
    }

    #[inline(always)]
    pub fn color(mut self, color: Color) -> Self {
        self.data.color = color;
        self
    }

    #[inline(always)]
    pub fn thickness(mut self, thickness: u32) -> Self {
        self.data.thickness = thickness;
        self
    }

    #[inline(always)]
    pub fn draw(self, drawer: &mut dyn Drawer) {
        drawer.execute_line(self.data);
    }
}
