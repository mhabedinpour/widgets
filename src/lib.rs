#![no_std]
extern crate alloc;

pub mod backend;
pub mod drawer;
pub mod http;
pub mod widget;

pub mod compiled_widgets {
    include!(concat!(env!("OUT_DIR"), "/widgets/mod.rs"));
}
