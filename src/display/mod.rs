//! Display subsystem for the Sipeed M1s Dock
//!
//! Provides support for the on-board 1.69" 240×280 ST7789 IPS LCD panel.

pub mod st7789;

pub use st7789::{
    DEFAULT_ROW_OFFSET, Orientation, SCREEN_HEIGHT, SCREEN_WIDTH, St7789Display,
};
