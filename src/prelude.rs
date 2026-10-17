//! Prelude for the Sipeed M1s Dock BSP
//!
//! Convenient re-exports of core board structures, extension traits, and formatting macros.

pub use crate::audio::{MicChannel, Microphone, SampleRate};
pub use crate::board::Board;
#[cfg(feature = "d0")]
pub use crate::board::M1sDisplay;
pub use crate::board::M1sTouch;
pub use crate::button::{Button, Buttons};
#[cfg(feature = "d0")]
pub use crate::camera::{Camera, CameraError, PixelFormat, Resolution};
pub use crate::console::Console;
pub use crate::display::St7789Display;
pub use crate::led::{Led, Leds};
#[cfg(feature = "m0")]
pub use crate::storage::SdCard;
pub use crate::storage::{StorageError, SECTOR_SIZE};
pub use crate::touch::{Cst816Touch, Gesture, TouchEvent, TouchPoint};
pub use crate::xram::{PacketHeader, XramError, XramMailbox, msg_type};
pub use bl808_hal::prelude::*;
pub use core::fmt::Write as _;
pub use embedded_hal::delay::DelayNs;
