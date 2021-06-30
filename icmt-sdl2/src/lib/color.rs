//! Color.

use crate::types::{byte_ceil, graphics};

pub fn translate_color(c: &graphics::Color) -> sdl2::pixels::Color {
    match c {
        (r, g, b) => sdl2::pixels::Color::RGB(byte_ceil(r), byte_ceil(g), byte_ceil(b)),
    }
}
