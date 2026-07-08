use crate::drawer::{Color, Drawer, Point};

#[derive(Clone, Copy)]
pub struct CircleData {
    pub center: Point,
    pub radius: u32,
    pub color: Color,
    pub fill: bool,
    pub stroke_width: u32,
}

pub struct CircleBuilder {
    pub data: CircleData,
}

impl CircleBuilder {
    #[inline(always)]
    pub fn new(cx: u32, cy: u32, r: u32) -> Self {
        Self {
            data: CircleData {
                center: Point { x: cx, y: cy },
                radius: r,
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
        drawer.execute_circle(self.data);
    }
}
