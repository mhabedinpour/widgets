# Embedded Wasm Widget OS: Architecture Specification

A bare-metal, `no_std` asynchronous operating system architecture for an ESP32 driving a 64x64 HUB75 LED matrix display. This system provides a sandboxed, event-driven WebAssembly runtime for dynamic widget deployment without sacrificing low-latency hardware rendering boundaries.

---

## 1. System Topology

The architecture separates hardware, core domain rendering math, and asynchronous event logic into isolated execution layers. Core 0 manages non-deterministic I/O tasks (Wi-Fi, Timers, HTTP requests), while Core 1 executes a non-blocking hardware parallel DMA loop.

```text
 ┌────────────────────────────────────────────────────────┐
 │                   Widget Manager                       │
 │  (Listens to SIGNAL_BUS, Drives Wasm3, Maps viewports) │
 └───────────────────────────┬────────────────────────────┘
                             │
                             ▼ Invokes `.rect().color().draw()`
 ┌────────────────────────────────────────────────────────┐
 │                    Drawer Trait                        │
 │  (Agnostic core interface. Spawns transient Builders)  │
 └───────────────────────────┬────────────────────────────┘
                             │
              ┌──────────────┴──────────────┐
              ▼ Passed into                 ▼ Implemented by
 ┌────────────────────────┐    ┌──────────────────────────┐
 │    Viewport Adapter    │    │  FramebufferDrawTarget   │
 │ (Relational Shifts &   │    │ (Direct embedded-graphics│
 │ Screen Edge Clipping)  │    │  rasterization wrapper)  │
 └────────────────────────┘    └────────────┬─────────────┘
                                            │
                                            ▼ Extended by
                               ┌──────────────────────────┐
                               │      Display Trait       │
                               │  (Async flush via i2s)   │
                               └──────────────────────────┘
```

2. Core Modules Implementation
   Module 1: Agnostic Domain Primitives & Fluent Drawer (src/drawer.rs)
   Defines the core spatial datatypes and a stripped-down primitive execution interface. It features stack-allocated, zero-overhead builders to facilitate method chaining without dynamic heap allocations.

```rust
// src/drawer.rs

pub struct Point { pub x: i32, pub y: i32 }
pub struct Size { pub width: u32, pub height: u32 }
pub struct Rect { pub origin: Point, pub size: Size }

pub trait Drawer {
    // Underlying primitive drawing execution hooks
    fn execute_rect(&mut self, rect: Rect, color: u32, fill: bool, stroke_width: u32, corner_radius: u32);
    fn execute_circle(&mut self, center: Point, radius: u32, color: u32, fill: bool, stroke_width: u32);
    fn execute_triangle(&mut self, p1: Point, p2: Point, p3: Point, color: u32, fill: bool, stroke_width: u32);
    fn execute_line(&mut self, start: Point, end: Point, color: u32, thickness: u32);
    fn execute_text(&mut self, text: &str, position: Point, color: u32);
    fn execute_clear(&mut self, color: u32);

    // Fluent entry points spawning transient builders
    #[inline(always)]
    fn rect(&mut self, x: i32, y: i32, w: u32, h: u32) -> RectBuilder<'_, Self> where Self: Sized {
        RectBuilder::new(self, x, y, w, h)
    }

    #[inline(always)]
    fn circle(&mut self, cx: i32, cy: i32, r: u32) -> CircleBuilder<'_, Self> where Self: Sized {
        CircleBuilder::new(self, cx, cy, r)
    }

    #[inline(always)]
    fn get_viewport<'a>(&'a mut self, bounds: Rect) -> crate::viewport::Viewport<'a> where Self: Sized {
        crate::viewport::Viewport::new(self, bounds)
    }
}

// Fluent Builder Example
pub struct RectBuilder<'a, D: Drawer> {
    drawer: &'a mut D,
    rect: Rect,
    color: u32,
    fill: bool,
    stroke_width: u32,
    corner_radius: u32,
}

impl<'a, D: Drawer> RectBuilder<'a, D> {
    #[inline(always)]
    pub fn new(drawer: &'a mut D, x: i32, y: i32, w: u32, h: u32) -> Self {
        Self {
            drawer,
            rect: Rect { origin: Point { x, y }, size: Size { width: w, height: h } },
            color: 0xFFFFFF,
            fill: true,
            stroke_width: 0,
            corner_radius: 0,
        }
    }

    #[inline(always)]
    pub fn color(mut self, hex: u32) -> Self { self.color = hex; self }

    #[inline(always)]
    pub fn stroke(mut self, width: u32) -> Self { self.fill = false; self.stroke_width = width; self }

    #[inline(always)]
    pub fn rounded(mut self, radius: u32) -> Self { self.corner_radius = radius; self }

    #[inline(always)]
    pub fn draw(self) {
        self.drawer.execute_rect(self.rect, self.color, self.fill, self.stroke_width, self.corner_radius);
    }
}
```

Module 2: Relational Viewport Adapter (src/viewport.rs)
Intercepts raw configuration callbacks. It shifts incoming relational coordinates by the target widget's spatial boundaries and clips coordinates that cross the viewport's hardware boundaries.
```rust
// src/viewport.rs
use crate::drawer::{Drawer, Point, Rect};

pub struct Viewport<'a> {
    parent: &'a mut dyn Drawer,
    bounds: Rect,
}

impl<'a> Viewport<'a> {
    pub fn new(parent: &'a mut dyn Drawer, bounds: Rect) -> Self {
        Self { parent, bounds }
    }

    #[inline(always)]
    fn translate(&self, p: Point) -> Point {
        Point {
            x: p.x + self.bounds.origin.x,
            y: p.y + self.bounds.origin.y,
        }
    }
}

impl<'a> Drawer for Viewport<'a> {
    fn execute_rect(&mut self, rect: Rect, color: u32, fill: bool, stroke_width: u32, corner_radius: u32) {
        let abs_origin = self.translate(rect.origin);
        // Guard checking the physical viewport region bounding box
        if abs_origin.x >= self.bounds.origin.x && abs_origin.y >= self.bounds.origin.y {
            self.parent.execute_rect(
                Rect { origin: abs_origin, size: rect.size },
                color, fill, stroke_width, corner_radius
            );
        }
    }

    fn execute_circle(&mut self, center: Point, radius: u32, color: u32, fill: bool, stroke_width: u32) {
        self.parent.execute_circle(self.translate(center), radius, color, fill, stroke_width);
    }

    fn execute_triangle(&mut self, p1: Point, p2: Point, p3: Point, color: u32, fill: bool, stroke_width: u32) {
        self.parent.execute_triangle(self.translate(p1), self.translate(p2), self.translate(p3), color, fill, stroke_width);
    }

    fn execute_line(&mut self, start: Point, end: Point, color: u32, thickness: u32) {
        self.parent.execute_line(self.translate(start), self.translate(end), color, thickness);
    }

    fn execute_text(&mut self, text: &str, position: Point, color: u32) {
        self.parent.execute_text(text, self.translate(position), color);
    }

    fn execute_clear(&mut self, color: u32) {
        let local_rect = Rect { origin: Point { x: 0, y: 0 }, size: self.bounds.size };
        self.execute_rect(local_rect, color, true, 0, 0);
    }
}
```

Module 3: Graphics Backend Wrapper & Hardware Core (src/backend.rs)
Binds custom drawing configurations directly to embedded-graphics primitives within isolated structures. It exposes a hardware Display layer that pumps the 64x64 buffer array via an asynchronous i2s parallel interface.
```rust
// src/backend.rs
use crate::drawer::{Drawer, Rect, Point, Size};
use embedded_graphics::{prelude::*, primitives::*, mono_font::{ascii::FONT_6X10, MonoTextStyle}, text::Text};

pub struct FramebufferDrawTarget {
    pub buffer: [u32; 4096], // Static 64x64 array allocation
}

pub trait Display: Drawer {
    type Error;
    async fn flush(&mut self) -> Result<(), Self::Error>;
}

impl Drawer for FramebufferDrawTarget {
    fn execute_rect(&mut self, rect: Rect, color: u32, fill: bool, stroke_width: u32, corner_radius: u32) {
        let c = RawU32::new(color);
        let mut style = PrimitiveStyleBuilder::new();
        if fill { style = style.fill_color(c); } 
        else { style = style.stroke_color(c).stroke_width(stroke_width); }

        let base_rect = Rectangle::new(Point::new(rect.origin.x, rect.origin.y), Size::new(rect.size.width, rect.size.height));

        if corner_radius > 0 {
            RoundedRectangle::new(base_rect, CornerRadii::new(Size::new(corner_radius, corner_radius)))
                .into_styled(style.build()).draw(self).unwrap();
        } else {
            base_rect.into_styled(style.build()).draw(self).unwrap();
        }
    }

    fn execute_circle(&mut self, center: Point, radius: u32, color: u32, fill: bool, stroke_width: u32) {
        let c = RawU32::new(color);
        let mut style = PrimitiveStyleBuilder::new();
        if fill { style = style.fill_color(c); } else { style = style.stroke_color(c).stroke_width(stroke_width); }
        Circle::with_center(Point::new(center.x, center.y), radius * 2).into_styled(style.build()).draw(self).unwrap();
    }

    fn execute_triangle(&mut self, p1: Point, p2: Point, p3: Point, color: u32, fill: bool, stroke_width: u32) {
        let c = RawU32::new(color);
        let mut style = PrimitiveStyleBuilder::new();
        if fill { style = style.fill_color(c); } else { style = style.stroke_color(c).stroke_width(stroke_width); }
        Triangle::new(Point::new(p1.x, p1.y), Point::new(p2.x, p2.y), Point::new(p3.x, p3.y)).into_styled(style.build()).draw(self).unwrap();
    }

    fn execute_line(&mut self, start: Point, end: Point, color: u32, thickness: u32) {
        let style = PrimitiveStyleBuilder::new().stroke_color(RawU32::new(color)).stroke_width(thickness).build();
        Line::new(Point::new(start.x, start.y), Point::new(end.x, end.y)).into_styled(style).draw(self).unwrap();
    }

    fn execute_text(&mut self, text: &str, position: Point, color: u32) {
        let character_style = MonoTextStyle::new(&FONT_6X10, RawU32::new(color));
        Text::new(text, Point::new(position.x, position.y), character_style).draw(self).unwrap();
    }

    fn execute_clear(&mut self, color: u32) {
        self.buffer.fill(color);
    }
}

impl DrawTarget for FramebufferDrawTarget {
    type Color = RawU32;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error> where I: IntoIterator<Item = Pixel<Self::Color>> {
        for Pixel(coord, color) in pixels.into_iter() {
            if coord.x >= 0 && coord.x < 64 && coord.y >= 0 && coord.y < 64 {
                self.buffer[(coord.y * 64 + coord.x) as usize] = color.into_inner();
            }
        }
        Ok(())
    }
}
```

Module 4: Async Coordination Loop & Wasm Bridge (src/manager.rs)
Consumes internal network, timer, or layout system notifications using lock-free Embassy channels. It maps host FFI callbacks to the static linear memory buffer space allocated for the active wasm3 module instances.

```rust
// src/manager.rs
use crate::drawer::{Drawer, Rect};
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

pub enum WidgetEvent {
    BootSignal,
    TimerInterrupt { widget_id: u32, timer_id: u32 },
    NetworkResponse { widget_id: u32, request_id: u32, status_code: u16 },
}

pub static SIGNAL_BUS: Channel<CriticalSectionRawMutex, WidgetEvent, 16> = Channel::new();

pub struct WasmWidget {
    pub id: u32,
    pub placement: Rect,
    pub wasm_instance: wasm3::Instance, 
}

pub struct WidgetManager {
    widgets: heapless::Vec<WasmWidget, 4>,
}

impl WidgetManager {
    pub fn execute_render_pass(&mut self, master_canvas: &mut dyn Drawer) {
        for widget in &mut self.widgets {
            // Bind transient relative viewport coordinates wrapper
            let mut view = master_canvas.get_viewport(widget.placement);
            
            // Invoke the Wasm3 guest execution hook
            execute_wasm_frame(&mut widget.wasm_instance, &mut view);
        }
    }

    pub async fn main_event_loop(&mut self) -> ! {
        for w in &self.widgets {
            SIGNAL_BUS.send(WidgetEvent::BootSignal).await;
        }

        loop {
            match SIGNAL_BUS.receive().await {
                WidgetEvent::BootSignal => { /* Run entry load functions */ },
                WidgetEvent::TimerInterrupt { widget_id, timer_id } => {
                    if let Some(w) = self.widgets.iter_mut().find(|x| x.id == widget_id) {
                        w.wasm_instance.call::<(u32,), ()>("on_timer", (timer_id,)).unwrap();
                    }
                },
                WidgetEvent::NetworkResponse { widget_id, request_id, status_code } => {
                    if let Some(w) = self.widgets.iter_mut().find(|x| x.id == widget_id) {
                        w.wasm_instance.call::<(u32, u32), ()>("on_network_data", (request_id, status_code as u32)).unwrap();
                    }
                }
            }
        }
    }
}

fn execute_wasm_frame(_instance: &mut wasm3::Instance, _canvas: &mut dyn Drawer) {
    // Marshals structural frame data across the virtual machine interface boundary
}
```

3. Coding Agent Instantiation Instructions
   When initiating development cycles with an automated software engineering agent, pass the following localized execution prompt block:
```text
[AGENT INTERCEPT PROTOCOL: BARE-METAL GRAPHICS CORE]
Environment: Bare-metal Rust (#![no_std]), esp-hal, embassy-executor loop framework.
Target: Implement decoupled primitives, viewports, and builders without third-party leakages.

Directives:
1. Deny access to the Rust standard ('std') library or global dynamic allocation crates.
2. Build the Fluent Builder structs using value-semantic ownership (`self`) across operations, appending `#[inline(always)]` to eliminate execution call stacks.
3. Keep logic paths in `src/drawer.rs` and `src/viewport.rs` isolated from raw hardware or embedded-graphics layout types.
```