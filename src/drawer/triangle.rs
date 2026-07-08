use crate::drawer::{Color, Drawer, Point};

#[derive(Clone, Copy)]
pub struct TriangleData {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub color: Color,
    pub fill: bool,
    pub stroke_width: u32,
}

pub struct TriangleBuilder {
    pub data: TriangleData,
}

impl TriangleBuilder {
    #[inline(always)]
    pub fn new(p1: Point, p2: Point, p3: Point) -> Self {
        Self {
            data: TriangleData {
                p1,
                p2,
                p3,
                color: Color::WHITE,
                fill: true,
                stroke_width: 0,
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
    pub fn draw(self, drawer: &mut dyn Drawer) {
        drawer.execute_triangle(self.data);
    }
}
