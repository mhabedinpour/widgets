use crate::drawer::{
    ArcData, Baseline, CircleData, ClearData, Color, Drawer, EllipseData, Font, LineData, Rect,
    RectData, SectorData, StrokeAlignment, TextAlignment, TextData, TriangleData,
};
use alloc::rc::Rc;
use core::cell::RefCell;
use embedded_graphics::{
    geometry::Angle,
    mono_font::{
        MonoFont, MonoTextStyleBuilder,
        ascii::{
            FONT_4X6, FONT_5X7, FONT_5X8, FONT_6X9, FONT_6X10, FONT_6X12, FONT_6X13,
            FONT_6X13_BOLD, FONT_6X13_ITALIC, FONT_7X13, FONT_7X13_BOLD, FONT_7X13_ITALIC,
            FONT_7X14, FONT_7X14_BOLD, FONT_8X13, FONT_8X13_BOLD, FONT_8X13_ITALIC, FONT_9X15,
            FONT_9X15_BOLD, FONT_9X18, FONT_9X18_BOLD, FONT_10X20,
        },
    },
    pixelcolor::Rgb888,
    prelude::*,
    primitives::*,
    text::{Alignment, Baseline as EgBaseline, Text, TextStyleBuilder},
};
use u8g2_fonts::{
    FontRenderer, fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
};

pub fn color_to_rgb888(color: Color) -> Rgb888 {
    match color {
        Color::Rgb(r, g, b) => Rgb888::new(r, g, b),
    }
}

fn is_u8g2_font(font: Font) -> bool {
    matches!(
        font,
        Font::U8g2Font3x3
            | Font::U8g2Font3x5
            | Font::U8g2Font4x6
            | Font::U8g2Font5x7
            | Font::U8g2Font5x8
    )
}

fn font_ref(font: Font) -> &'static MonoFont<'static> {
    match font {
        Font::Font4x6 => &FONT_4X6,
        Font::Font5x7 => &FONT_5X7,
        Font::Font5x8 => &FONT_5X8,
        Font::Font6x9 => &FONT_6X9,
        Font::Font6x10 => &FONT_6X10,
        Font::Font6x12 => &FONT_6X12,
        Font::Font6x13 => &FONT_6X13,
        Font::Font6x13Bold => &FONT_6X13_BOLD,
        Font::Font6x13Italic => &FONT_6X13_ITALIC,
        Font::Font7x13 => &FONT_7X13,
        Font::Font7x13Bold => &FONT_7X13_BOLD,
        Font::Font7x13Italic => &FONT_7X13_ITALIC,
        Font::Font7x14 => &FONT_7X14,
        Font::Font7x14Bold => &FONT_7X14_BOLD,
        Font::Font8x13 => &FONT_8X13,
        Font::Font8x13Bold => &FONT_8X13_BOLD,
        Font::Font8x13Italic => &FONT_8X13_ITALIC,
        Font::Font9x15 => &FONT_9X15,
        Font::Font9x15Bold => &FONT_9X15_BOLD,
        Font::Font9x18 => &FONT_9X18,
        Font::Font9x18Bold => &FONT_9X18_BOLD,
        Font::Font10x20 => &FONT_10X20,
        // u8g2 variants are handled separately — should never reach here
        Font::U8g2Font3x3
        | Font::U8g2Font3x5
        | Font::U8g2Font4x6
        | Font::U8g2Font5x7
        | Font::U8g2Font5x8 => {
            unreachable!("u8g2 font passed to mono font_ref()")
        }
    }
}

fn stroke_alignment_to_eg(a: StrokeAlignment) -> embedded_graphics::primitives::StrokeAlignment {
    match a {
        StrokeAlignment::Inside => embedded_graphics::primitives::StrokeAlignment::Inside,
        StrokeAlignment::Center => embedded_graphics::primitives::StrokeAlignment::Center,
        StrokeAlignment::Outside => embedded_graphics::primitives::StrokeAlignment::Outside,
    }
}

pub struct EmbeddedGraphicsDrawer<T: DrawTarget<Color = Rgb888>> {
    target: Rc<RefCell<T>>,
    clip: Rectangle,
}

impl<T: DrawTarget<Color = Rgb888>> EmbeddedGraphicsDrawer<T> {
    pub fn new(target: Rc<RefCell<T>>, clip: Rect) -> EmbeddedGraphicsDrawer<T> {
        EmbeddedGraphicsDrawer {
            target,
            clip: Rectangle::new(
                Point::new(clip.origin.x as i32, clip.origin.y as i32),
                Size::new(clip.size.width, clip.size.height),
            ),
        }
    }
}

impl<T: DrawTarget<Color = Rgb888>> Drawer for EmbeddedGraphicsDrawer<T> {
    fn execute_rect(&mut self, data: RectData) {
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(color_to_rgb888(data.fill_color));
        }
        if data.stroke_width > 0 {
            style = style
                .stroke_color(color_to_rgb888(data.stroke_color))
                .stroke_width(data.stroke_width)
                .stroke_alignment(stroke_alignment_to_eg(data.stroke_alignment));
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
            .draw(
                &mut self
                    .target
                    .borrow_mut()
                    .clipped(&self.clip)
                    .cropped(&self.clip),
            )
            .ok();
        } else {
            base_rect
                .into_styled(style.build())
                .draw(
                    &mut self
                        .target
                        .borrow_mut()
                        .clipped(&self.clip)
                        .cropped(&self.clip),
                )
                .ok();
        }
    }

    fn execute_circle(&mut self, data: CircleData) {
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(color_to_rgb888(data.fill_color));
        }
        if data.stroke_width > 0 {
            style = style
                .stroke_color(color_to_rgb888(data.stroke_color))
                .stroke_width(data.stroke_width)
                .stroke_alignment(stroke_alignment_to_eg(data.stroke_alignment));
        }

        Circle::with_center(
            Point::new(data.center.x as i32, data.center.y as i32),
            data.radius * 2,
        )
        .into_styled(style.build())
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
        .ok();
    }

    fn execute_triangle(&mut self, data: TriangleData) {
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(color_to_rgb888(data.fill_color));
        }
        if data.stroke_width > 0 {
            style = style
                .stroke_color(color_to_rgb888(data.stroke_color))
                .stroke_width(data.stroke_width)
                .stroke_alignment(stroke_alignment_to_eg(data.stroke_alignment));
        }

        Triangle::new(
            Point::new(data.p1.x as i32, data.p1.y as i32),
            Point::new(data.p2.x as i32, data.p2.y as i32),
            Point::new(data.p3.x as i32, data.p3.y as i32),
        )
        .into_styled(style.build())
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
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
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
        .ok();
    }

    fn execute_text(&mut self, data: TextData) {
        if is_u8g2_font(data.font) {
            self.execute_text_u8g2(data);
            return;
        }
        let text_color = color_to_rgb888(data.color);
        let mono_font = font_ref(data.font);

        let mut char_style_builder = MonoTextStyleBuilder::new()
            .font(mono_font)
            .text_color(text_color);

        if data.has_background {
            char_style_builder =
                char_style_builder.background_color(color_to_rgb888(data.background_color));
        }
        if data.underline {
            char_style_builder = char_style_builder.underline();
        }
        if data.strikethrough {
            char_style_builder = char_style_builder.strikethrough();
        }

        let char_style = char_style_builder.build();

        let eg_alignment = match data.alignment {
            TextAlignment::Left => Alignment::Left,
            TextAlignment::Center => Alignment::Center,
            TextAlignment::Right => Alignment::Right,
        };
        let eg_baseline = match data.baseline {
            Baseline::Alphabetic => EgBaseline::Alphabetic,
            Baseline::Top => EgBaseline::Top,
            Baseline::Middle => EgBaseline::Middle,
            Baseline::Bottom => EgBaseline::Bottom,
        };

        let text_style = TextStyleBuilder::new()
            .alignment(eg_alignment)
            .baseline(eg_baseline)
            .build();

        Text::with_text_style(
            &data.text,
            Point::new(data.position.x as i32, data.position.y as i32),
            char_style,
            text_style,
        )
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
        .ok();
    }

    fn execute_ellipse(&mut self, data: EllipseData) {
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(color_to_rgb888(data.fill_color));
        }
        if data.stroke_width > 0 {
            style = style
                .stroke_color(color_to_rgb888(data.stroke_color))
                .stroke_width(data.stroke_width)
                .stroke_alignment(stroke_alignment_to_eg(data.stroke_alignment));
        }

        Ellipse::new(
            Point::new(
                data.bounding_box.origin.x as i32,
                data.bounding_box.origin.y as i32,
            ),
            Size::new(data.bounding_box.size.width, data.bounding_box.size.height),
        )
        .into_styled(style.build())
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
        .ok();
    }

    fn execute_arc(&mut self, data: ArcData) {
        let style = PrimitiveStyleBuilder::new()
            .stroke_color(color_to_rgb888(data.stroke_color))
            .stroke_width(data.stroke_width)
            .stroke_alignment(stroke_alignment_to_eg(data.stroke_alignment))
            .build();

        Arc::with_center(
            Point::new(data.center.x as i32, data.center.y as i32),
            data.radius * 2,
            Angle::from_degrees(data.angle_start as f32),
            Angle::from_degrees(data.angle_sweep as f32),
        )
        .into_styled(style)
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
        .ok();
    }

    fn execute_sector(&mut self, data: SectorData) {
        let mut style = PrimitiveStyleBuilder::new();
        if data.fill {
            style = style.fill_color(color_to_rgb888(data.fill_color));
        }
        if data.stroke_width > 0 {
            style = style
                .stroke_color(color_to_rgb888(data.stroke_color))
                .stroke_width(data.stroke_width)
                .stroke_alignment(stroke_alignment_to_eg(data.stroke_alignment));
        }

        Sector::with_center(
            Point::new(data.center.x as i32, data.center.y as i32),
            data.radius * 2,
            Angle::from_degrees(data.angle_start as f32),
            Angle::from_degrees(data.angle_sweep as f32),
        )
        .into_styled(style.build())
        .draw(
            &mut self
                .target
                .borrow_mut()
                .clipped(&self.clip)
                .cropped(&self.clip),
        )
        .ok();
    }

    fn execute_clear(&mut self, data: ClearData) {
        self.target
            .borrow_mut()
            .clipped(&self.clip)
            .cropped(&self.clip)
            .clear(color_to_rgb888(data.color))
            .ok();
    }

    fn bounds_x(&mut self) -> u32 {
        self.clip.top_left.x as u32
    }

    fn bounds_y(&mut self) -> u32 {
        self.clip.top_left.y as u32
    }

    fn bounds_width(&mut self) -> u32 {
        self.clip.size.width
    }

    fn bounds_height(&mut self) -> u32 {
        self.clip.size.height
    }
}

impl<T: DrawTarget<Color = Rgb888>> EmbeddedGraphicsDrawer<T> {
    fn execute_text_u8g2(&mut self, data: TextData) {
        let color = FontColor::Transparent(color_to_rgb888(data.color));
        let pos = Point::new(data.position.x as i32, data.position.y as i32);
        let h_align = match data.alignment {
            TextAlignment::Left => HorizontalAlignment::Left,
            TextAlignment::Center => HorizontalAlignment::Center,
            TextAlignment::Right => HorizontalAlignment::Right,
        };
        let v_pos = match data.baseline {
            Baseline::Top => VerticalPosition::Top,
            Baseline::Middle => VerticalPosition::Center,
            Baseline::Alphabetic | Baseline::Bottom => VerticalPosition::Baseline,
        };

        macro_rules! render_u8g2 {
            ($font:ty) => {
                FontRenderer::new::<$font>()
                    .render_aligned(
                        data.text.as_str(),
                        pos,
                        v_pos,
                        h_align,
                        color,
                        &mut self
                            .target
                            .borrow_mut()
                            .clipped(&self.clip)
                            .cropped(&self.clip),
                    )
                    .ok()
            };
        }

        match data.font {
            Font::U8g2Font3x3 => {
                render_u8g2!(fonts::u8g2_font_tiny_simon_tr);
            }
            Font::U8g2Font3x5 => {
                render_u8g2!(fonts::u8g2_font_3x5im_mr);
            }
            Font::U8g2Font4x6 => {
                render_u8g2!(fonts::u8g2_font_4x6_mf);
            }
            Font::U8g2Font5x7 => {
                render_u8g2!(fonts::u8g2_font_5x7_mf);
            }
            Font::U8g2Font5x8 => {
                render_u8g2!(fonts::u8g2_font_5x8_mf);
            }
            _ => {}
        }
    }
}

impl<T: DrawTarget<Color = Rgb888>> Drop for EmbeddedGraphicsDrawer<T> {
    fn drop(&mut self) {
        self.execute_clear(ClearData {
            color: Color::BLACK,
        });
    }
}
