#![no_std]

//! Board Support Package (BSP) for the Sipeed M1s Dock (Bouffalo Lab BL808).
//!
//! Provides high-level hardware bindings for the Sipeed M1s Dock development board:
//! - On-board RGB Status LEDs (GPIO 8, 7, 3)
//! - User-programmable Button (GPIO 23)
//! - Pre-configured Serial Console (UART3 on D0 / UART0 on M0)
//! - Calibrated Hardware Delay (`DelayNs`)
//!
//! # Getting Started
//!
//! ```no_run
//! use m1s_dock_bsp::prelude::*;
//!
//! // Obtain ownership of the board
//! let mut board = Board::take().unwrap();
//!
//! // Blink the red LED
//! board.leds.red.on();
//! board.delay_ms(500);
//! board.leds.red.off();
//!
//! // Write to the console
//! writeln!(board.console, "Hello from {}", board.core_name()).unwrap();
//! ```

#[cfg(test)]
extern crate std;

pub mod audio;
pub mod board;
pub mod button;
#[cfg(feature = "d0")]
pub mod camera;
pub mod console;
pub mod display;
pub mod led;
pub mod prelude;
pub mod storage;
pub mod touch;
pub mod xram;

pub use audio::{MicChannel, Microphone, SampleRate};
pub use board::{Board, M1sTouch, TouchParts};
#[cfg(feature = "d0")]
pub use board::{DisplayParts, M1sDisplay};
pub use button::{Button, Buttons};
#[cfg(feature = "d0")]
pub use camera::{Camera, CameraError, PixelFormat, Resolution};
pub use console::Console;
pub use display::St7789Display;
pub use led::{Led, Leds};
#[cfg(feature = "m0")]
pub use storage::SdCard;
pub use storage::{StorageError, SECTOR_SIZE};
pub use touch::Cst816Touch;
pub use xram::{PacketHeader, XramError, XramMailbox, XramRingBuffer, msg_type};

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn test_board_core_name() {
        // Install mock GLB register block
        let glb_mock = [0u32; 4096];
        bl808_hal::glb::install_test_mock(
            glb_mock.as_ptr() as *const bl808_pac::glb::RegisterBlock,
        );

        // Install mock UART register block
        let uart_mock = [0u32; 256];
        #[cfg(feature = "d0")]
        bl808_hal::uart::MOCK_UART3_PTR.with(|p| {
            p.set(uart_mock.as_ptr() as *const bl808_pac::uart0::RegisterBlock);
        });
        #[cfg(feature = "m0")]
        bl808_hal::uart::MOCK_UART0_PTR.with(|p| {
            p.set(uart_mock.as_ptr() as *const bl808_pac::uart0::RegisterBlock);
        });

        // Install mock Timer register block
        let timer_mock = [0u32; 1024];
        bl808_hal::timer::MOCK_TIMER0_PTR.with(|p| {
            p.set(timer_mock.as_ptr() as *const bl808_pac::timer0::RegisterBlock);
        });

        let mut board = unsafe { Board::steal() };

        #[cfg(feature = "d0")]
        assert!(board.core_name().contains("C906 D0"));

        #[cfg(feature = "m0")]
        assert!(board.core_name().contains("E907 M0"));

        // Test LED controls
        board.leds.all_off();
        board.leds.red.on();
        board.leds.green.on();
        board.leds.blue.off();
        board.leds.red.toggle();

        // Test button query
        let _ = board.buttons.user.is_pressed();
        let _ = board.buttons.user.is_released();

        // Test console write
        writeln!(board.console, "Core: {}", board.core_name()).expect("Console write must succeed");

        // Test delays
        board.delay_ms(1);
        board.delay_us(10);
    }
}
