//! Serial Console Driver for Sipeed M1s Dock
//!
//! Provides the primary text console for diagnostics and user interaction.
//! - On **D0 (C906)**: Uses UART3 (2 000 000 or 115 200 baud).
//! - On **M0 (E907)**: Uses UART0 (115 200 baud).

use bl808_hal::gpio::{Alternate, Pin};
use bl808_hal::uart::{Serial, SerialError};
use core::fmt::{self, Write};

#[cfg(feature = "d0")]
pub type ConsoleUart = bl808_pac::Uart3;

#[cfg(feature = "d0")]
pub type ConsoleTxPin = Pin<16, Alternate<7>>;

#[cfg(feature = "d0")]
pub type ConsoleRxPin = Pin<17, Alternate<7>>;

#[cfg(feature = "m0")]
pub type ConsoleUart = bl808_pac::Uart0;

#[cfg(feature = "m0")]
pub type ConsoleTxPin = Pin<14, Alternate<7>>;

#[cfg(feature = "m0")]
pub type ConsoleRxPin = Pin<15, Alternate<7>>;

/// Serial debug console for the M1s Dock.
pub struct Console {
    serial: Serial<ConsoleUart, ConsoleTxPin, ConsoleRxPin>,
}

impl Console {
    /// Creates a new `Console` instance wrapping the configured UART peripheral and pins.
    pub fn new(serial: Serial<ConsoleUart, ConsoleTxPin, ConsoleRxPin>) -> Self {
        Self { serial }
    }

    /// Writes a single byte to the console output.
    pub fn write_byte(&mut self, byte: u8) {
        self.serial.write_byte(byte);
    }

    /// Writes a slice of bytes to the console output.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.serial.write_bytes(bytes);
    }

    /// Flushes any pending data in the console output buffer.
    pub fn flush(&mut self) {
        self.serial.flush_tx();
    }

    /// Reads a single byte from the console input (non-blocking).
    pub fn read_byte(&mut self) -> nb::Result<u8, SerialError> {
        embedded_hal_nb::serial::Read::read(&mut self.serial)
    }

    /// Releases the inner HAL serial peripheral.
    pub fn release(self) -> Serial<ConsoleUart, ConsoleTxPin, ConsoleRxPin> {
        self.serial
    }
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.serial.write_str(s)
    }
}
