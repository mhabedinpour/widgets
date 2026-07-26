#![no_std]
#![feature(allocator_api)]
extern crate alloc;

pub mod admin;
pub mod allocator;
pub mod backend;
pub mod boot_screen;
pub mod console;
pub mod drawer;
pub mod http;
pub mod network;
pub mod time;
pub mod time_sync;
pub mod widget;

pub use allocator::{set_psram_alloc, use_psram_heap, use_sram_heap};

pub mod compiled_widgets {
    include!(concat!(env!("OUT_DIR"), "/widgets/mod.rs"));
}
