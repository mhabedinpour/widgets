use crate::drawer::{Color, Drawer, Point, Rect, Size};

#[derive(Clone, Copy)]
pub struct RectData {
    pub rect: Rect,
    pub color: Color,
    pub fill: bool,
    pub stroke_width: u32,
    pub corner_radius: u32,
}

pub struct RectBuilder {
    pub data: RectData,
}

impl RectBuilder {
    #[inline(always)]
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Self {
            data: RectData {
                rect: Rect {
                    origin: Point { x, y },
                    size: Size {
                        width: w,
                        height: h,
                    },
                },
                color: Color::WHITE,
                fill: true,
                stroke_width: 0,
                corner_radius: 0,
            },
        }
    }

    #[inline(always)]
    pub fn color(mut self, color: Color) -> Self {
        self.data.color = color;
        self
    }

    #[inline(always)]
    pub fn stroke(mut self, width: u32) -> Self {
        self.data.fill = false;
        self.data.stroke_width = width;
        self
    }

    #[inline(always)]
    pub fn rounded(mut self, radius: u32) -> Self {
        self.data.corner_radius = radius;
        self
    }

    #[inline(always)]
    pub fn draw(self, drawer: &mut dyn Drawer) {
        drawer.execute_rect(self.data);
    }
}
