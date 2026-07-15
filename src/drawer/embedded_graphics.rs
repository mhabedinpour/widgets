use crate::drawer::{
    CircleData, ClearData, Color, Drawer, LineData, Rect, RectData, TextData, TriangleData,
};
use alloc::boxed::Box;
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X10},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::*,
    text::Text,
};

pub fn color_to_rgb888(color: Color) -> Rgb888 {
    match color {
        Color::Rgb(r, g, b) => Rgb888::new(r, g, b),
    }
}

pub struct EmbeddedGraphicsDrawer<T: DrawTarget<Color = Rgb888> + Clone> {
    target: Box<T>,
    clip: Rectangle,
}

impl<T: DrawTarget<Color = Rgb888> + Clone> EmbeddedGraphicsDrawer<T> {
    pub fn root(target: Box<T>, size: crate::drawer::Size) -> EmbeddedGraphicsDrawer<T> {
        let base_rect = Rectangle::new(Point::new(0, 0), Size::new(size.width, size.height));

        EmbeddedGraphicsDrawer {
            target,
            clip: base_rect,
        }
    }

    pub fn clone_target(&mut self) -> Box<T> {
        self.target.clone()
    }
}

impl<T: DrawTarget<Color = Rgb888> + Clone> Drawer for EmbeddedGraphicsDrawer<T> {
    fn execute_rect(&mut self, data: RectData) {
        let c = color_to_rgb888(data.color);
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(c);
        } else {
            style = style.stroke_color(c).stroke_width(data.stroke_width);
        }

        let base_rect = Rectangle::new(
            Point::new(data.rect.origin.x as i32, data.rect.origin.y as i32),
            Size::new(data.rect.size.width, data.rect.size.height),
        );

        if data.corner_radius > 0 {
            RoundedRectangle::new(
                base_rect,
                CornerRadii::new(Size::new(data.corner_radius, data.corner_radius)),
            )
            .into_styled(style.build())
            .draw(&mut self.target.clipped(&self.clip).cropped(&self.clip))
            .ok();
        } else {
            base_rect
                .into_styled(style.build())
                .draw(&mut self.target.clipped(&self.clip).cropped(&self.clip))
                .ok();
        }
    }

    fn execute_circle(&mut self, data: CircleData) {
        let c = color_to_rgb888(data.color);
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(c);
        } else {
            style = style.stroke_color(c).stroke_width(data.stroke_width);
        }

        Circle::with_center(
            Point::new(data.center.x as i32, data.center.y as i32),
            data.radius * 2,
        )
        .into_styled(style.build())
        .draw(&mut self.target.clipped(&self.clip).cropped(&self.clip))
        .ok();
    }

    fn execute_triangle(&mut self, data: TriangleData) {
        let c = color_to_rgb888(data.color);
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(c);
        } else {
            style = style.stroke_color(c).stroke_width(data.stroke_width);
        }

        Triangle::new(
            Point::new(data.p1.x as i32, data.p1.y as i32),
            Point::new(data.p2.x as i32, data.p2.y as i32),
            Point::new(data.p3.x as i32, data.p3.y as i32),
        )
        .into_styled(style.build())
        .draw(&mut self.target.clipped(&self.clip).cropped(&self.clip))
        .ok();
    }

    fn execute_line(&mut self, data: LineData) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color_to_rgb888(data.color))
            .stroke_width(data.thickness)
            .build();

        Line::new(
            Point::new(data.start.x as i32, data.start.y as i32),
            Point::new(data.end.x as i32, data.end.y as i32),
        )
        .into_styled(style)
        .draw(&mut self.target.clipped(&self.clip).cropped(&self.clip))
        .ok();
    }

    fn execute_text(&mut self, data: TextData<'_>) {
        let character_style = MonoTextStyle::new(&FONT_6X10, color_to_rgb888(data.color));

        Text::new(
            data.text,
            Point::new(data.position.x as i32, data.position.y as i32),
            character_style,
        )
        .draw(&mut self.target.clipped(&self.clip).cropped(&self.clip))
        .ok();
    }

    fn execute_clear(&mut self, data: ClearData) {
        self.target
            .clipped(&self.clip)
            .cropped(&self.clip)
            .clear(color_to_rgb888(data.color))
            .ok();
    }

    fn with_viewport(&mut self, bounds: Rect, f: &mut dyn FnMut(&mut dyn Drawer)) {
        let bounds_origin = Point::new(bounds.origin.x as i32, bounds.origin.y as i32);
        let bounds_size = Size::new(bounds.size.width, bounds.size.height);
        let viewport_rect = Rectangle::new(bounds_origin, bounds_size);

        let previous_clip = self.clip;
        self.clip = self
            .target
            .clipped(&previous_clip)
            .clipped(&viewport_rect)
            .bounding_box();

        let result = f(self);

        self.clip = previous_clip;

        result
    }
}
