//! Capacitive Touch subsystem for the Sipeed M1s Dock
//!
//! Provides gesture and touch coordinate tracking via the on-board CST816S touch controller.

pub mod cst816;

pub use cst816::{CST816_I2C_ADDR, Cst816Touch, Gesture, TouchEvent, TouchPoint};
