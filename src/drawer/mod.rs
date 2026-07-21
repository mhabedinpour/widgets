pub mod circle;
pub mod embedded_graphics;
pub mod line;
pub mod rect;
pub mod text;
pub mod triangle;
pub mod types;

use alloc::boxed::Box;
pub use circle::{CircleBuilder, CircleData};
pub use embedded_graphics::*;
pub use line::{LineBuilder, LineData};
pub use rect::{RectBuilder, RectData};
pub use text::{TextBuilder, TextData};
pub use triangle::{TriangleBuilder, TriangleData};
pub use types::*;

/// @wasm
pub trait Drawer {
    // Underlying primitive drawing execution hooks
    /// @wasm builder_name="rect"
    fn execute_rect(&mut self, data: RectData);
    /// @wasm builder_name="circle"
    fn execute_circle(&mut self, data: CircleData);
    /// @wasm builder_name="triangle"
    fn execute_triangle(&mut self, data: TriangleData);
    /// @wasm builder_name="line"
    fn execute_line(&mut self, data: LineData);
    /// @wasm builder_name="text"
    fn execute_text(&mut self, data: TextData);
    /// @wasm builder_name="clear"
    fn execute_clear(&mut self, data: ClearData);
    /// @wasm builder_name="boundsX"
    fn bounds_x(&mut self) -> u32;
    /// @wasm builder_name="boundsY"
    fn bounds_y(&mut self) -> u32;
    /// @wasm builder_name="boundsWidth"
    fn bounds_width(&mut self) -> u32;
    /// @wasm builder_name="boundsHeight"
    fn bounds_height(&mut self) -> u32;
}

pub trait GlobalDrawer {
    fn scoped(&self, bounds: Rect) -> Box<dyn Drawer>;
    fn flush(&self);
}
