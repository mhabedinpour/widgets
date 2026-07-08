pub mod circle;
pub mod embedded_graphics;
pub mod line;
pub mod rect;
pub mod text;
pub mod triangle;
pub mod types;

pub use circle::{CircleBuilder, CircleData};
pub use embedded_graphics::*;
pub use line::{LineBuilder, LineData};
pub use rect::{RectBuilder, RectData};
pub use text::{TextBuilder, TextData};
pub use triangle::{TriangleBuilder, TriangleData};
pub use types::*;

pub trait Drawer {
    // Underlying primitive drawing execution hooks
    fn execute_rect(&mut self, data: RectData);
    fn execute_circle(&mut self, data: CircleData);
    fn execute_triangle(&mut self, data: TriangleData);
    fn execute_line(&mut self, data: LineData);
    fn execute_text(&mut self, data: TextData<'_>);
    fn execute_clear(&mut self, color: Color);

    fn with_viewport(&mut self, bounds: Rect, f: &mut dyn FnMut(&mut dyn Drawer));
}
