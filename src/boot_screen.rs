// Boot screen — shown while waiting for Wi-Fi DHCP.
//
// Draws a cyan scan-line that sweeps top→bottom across the 64×64 display
// with an 8-pixel fading tail.  Runs until `stack.is_config_up()` returns
// true (or, as a safety valve, for at most MAX_FRAMES frames).

use crate::drawer::{ClearData, Color, GlobalDrawer, LineData, Point, Rect, Size};
use embassy_time::Timer;

/// Frames before we give up waiting (300 × 50 ms = 15 s).
const MAX_FRAMES: u32 = 300;

/// Head-to-tail colour ramp: bright cyan → near-black.
const TAIL: [Color; 8] = [
    Color::Rgb(0, 220, 255), // head — bright cyan
    Color::Rgb(0, 220, 255),
    Color::Rgb(0, 140, 170), // dim cyan
    Color::Rgb(0, 140, 170),
    Color::Rgb(60, 80, 110), // blue-gray
    Color::Rgb(40, 55, 80),
    Color::Rgb(25, 35, 55),
    Color::Rgb(10, 15, 25), // near-black tail tip
];

pub async fn run(drawer: &dyn GlobalDrawer, stack: embassy_net::Stack<'static>) {
    let full = Rect::new(Point::new(0, 0), Size::new(64, 64));
    let mut frame: u32 = 0;

    loop {
        if stack.is_config_up() || frame >= MAX_FRAMES {
            break;
        }

        {
            let mut d = drawer.scoped(full);

            d.execute_clear(ClearData {
                color: Color::BLACK,
            });

            // Head advances 2 px per frame at 50 ms → full 64-px pass ≈ 1.6 s.
            let head_y = (frame * 2) % 64;
            for (i, &color) in TAIL.iter().enumerate() {
                // Wrap upward so the tail doesn't bleed through the top.
                let y = (head_y + 64 - i as u32) % 64;
                d.execute_line(LineData {
                    start: Point::new(0, y),
                    end: Point::new(63, y),
                    color,
                    thickness: 1,
                });
            }
        }

        drawer.flush();
        frame += 1;
        Timer::after_millis(50).await;
    }
}
