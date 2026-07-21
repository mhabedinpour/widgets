use crate::drawer::{Color, Drawer, Point, StrokeAlignment};

/// @wasm required="center,radius"
#[derive(Clone, Copy)]
pub struct ArcData {
    pub center: Point,
    pub radius: u32,
    /// @default 0
    pub angle_start: i32,
    /// @default 360
    pub angle_sweep: i32,
    /// @default Color::WHITE
    pub stroke_color: Color,
    /// @default 1
    pub stroke_width: u32,
    /// @default StrokeAlignment::Center
    pub stroke_alignment: StrokeAlignment,
}

pub struct ArcBuilder {
    pub data: ArcData,
}

impl ArcBuilder {
    #[inline(always)]
    pub fn new(cx: u32, cy: u32, r: u32) -> Self {
        Self {
            data: ArcData {
                center: Point { x: cx, y: cy },
                radius: r,
                angle_start: 0,
                angle_sweep: 360,
                stroke_color: Color::WHITE,
                stroke_width: 1,
                stroke_alignment: StrokeAlignment::Center,
            },
        }
    }

    /// Sets start angle in degrees.
    #[inline(always)]
    pub fn angle_start(mut self, degrees: i32) -> Self {
        self.data.angle_start = degrees;
        self
    }

    /// Sets sweep angle in degrees.
    #[inline(always)]
    pub fn angle_sweep(mut self, degrees: i32) -> Self {
        self.data.angle_sweep = degrees;
        self
    }

    /// Sets both start and sweep angles in degrees.
    #[inline(always)]
    pub fn angles(mut self, start: i32, sweep: i32) -> Self {
        self.data.angle_start = start;
        self.data.angle_sweep = sweep;
        self
    }

    /// Sets the stroke color.
    #[inline(always)]
    pub fn color(mut self, color: Color) -> Self {
        self.data.stroke_color = color;
        self
    }

    /// Sets the stroke color.
    #[inline(always)]
    pub fn stroke_color(mut self, color: Color) -> Self {
        self.data.stroke_color = color;
        self
    }

    /// Sets the stroke width.
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
        drawer.execute_arc(self.data);
    }
}
