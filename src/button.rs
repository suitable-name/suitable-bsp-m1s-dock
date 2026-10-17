//! On-board Button Driver for Sipeed M1s Dock
//!
//! The M1s Dock provides a user-programmable button on **GPIO 23** (Active-Low).

use bl808_hal::gpio::{Input, Pin, PullUp};
use embedded_hal::digital::InputPin;

/// An on-board button configured with pull-up input and active-low logic.
pub struct Button<const P: u8> {
    pin: Pin<P, Input<PullUp>>,
}

impl<const P: u8> Button<P> {
    /// Creates a new `Button` from a configured pull-up input pin.
    pub fn new(pin: Pin<P, Input<PullUp>>) -> Self {
        Self { pin }
    }

    /// Returns `true` if the button is currently held down (pin is LOW).
    pub fn is_pressed(&mut self) -> bool {
        self.pin.is_low().unwrap_or(false)
    }

    /// Returns `true` if the button is currently released (pin is HIGH).
    pub fn is_released(&mut self) -> bool {
        self.pin.is_high().unwrap_or(true)
    }

    /// Releases the inner GPIO pin.
    pub fn release(self) -> Pin<P, Input<PullUp>> {
        self.pin
    }
}

/// Collection of on-board buttons on the Sipeed M1s Dock.
pub struct Buttons {
    /// User button (GPIO 23).
    pub user: Button<23>,
}

impl Buttons {
    /// Creates the button bundle from raw pull-up input pins.
    pub fn new(user_pin: Pin<23, Input<PullUp>>) -> Self {
        Self {
            user: Button::new(user_pin),
        }
    }
}
