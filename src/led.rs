//! On-board LED Driver for Sipeed M1s Dock
//!
//! The M1s Dock has an on-board RGB LED connected to:
//! - **Red**: GPIO 8 (Active-Low)
//! - **Green**: GPIO 7 (Active-Low)
//! - **Blue**: GPIO 3 (Active-Low)

use bl808_hal::gpio::{Output, Pin, PushPull};
use embedded_hal::digital::{OutputPin, StatefulOutputPin};

/// An on-board LED configured with active-low polarity.
pub struct Led<const P: u8> {
    pin: Pin<P, Output<PushPull>>,
}

impl<const P: u8> Led<P> {
    /// Creates a new `Led` from a configured output pin and turns it OFF initially.
    pub fn new(mut pin: Pin<P, Output<PushPull>>) -> Self {
        let _ = pin.set_high(); // Active-low: high = off
        Self { pin }
    }

    /// Turns the LED ON (drives pin LOW).
    pub fn on(&mut self) {
        let _ = self.pin.set_low();
    }

    /// Turns the LED OFF (drives pin HIGH).
    pub fn off(&mut self) {
        let _ = self.pin.set_high();
    }

    /// Toggles the LED state.
    pub fn toggle(&mut self) {
        let _ = self.pin.toggle();
    }

    /// Returns `true` if the LED is currently ON.
    pub fn is_on(&mut self) -> bool {
        self.pin.is_set_low().unwrap_or(false)
    }

    /// Returns `true` if the LED is currently OFF.
    pub fn is_off(&mut self) -> bool {
        self.pin.is_set_high().unwrap_or(true)
    }

    /// Releases the inner GPIO pin.
    pub fn release(self) -> Pin<P, Output<PushPull>> {
        self.pin
    }
}

/// Collection of the three on-board RGB LEDs on the Sipeed M1s Dock.
pub struct Leds {
    /// Red status LED (GPIO 8).
    pub red: Led<8>,
    /// Green status LED (GPIO 7).
    pub green: Led<7>,
    /// Blue status LED (GPIO 3).
    pub blue: Led<3>,
}

impl Leds {
    /// Creates the RGB LED bundle from raw push-pull output pins.
    pub fn new(red_pin: Pin<8, Output<PushPull>>, green_pin: Pin<7, Output<PushPull>>, blue_pin: Pin<3, Output<PushPull>>) -> Self {
        Self {
            red: Led::new(red_pin),
            green: Led::new(green_pin),
            blue: Led::new(blue_pin),
        }
    }

    /// Turns all three LEDs OFF.
    pub fn all_off(&mut self) {
        self.red.off();
        self.green.off();
        self.blue.off();
    }

    /// Sets the color combination of the RGB LED.
    pub fn set_rgb(&mut self, red: bool, green: bool, blue: bool) {
        if red { self.red.on(); } else { self.red.off(); }
        if green { self.green.on(); } else { self.green.off(); }
        if blue { self.blue.on(); } else { self.blue.off(); }
    }
}
