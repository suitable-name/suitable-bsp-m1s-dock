#![no_std]
#![no_main]

use m1s_dock_bsp::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - Button Event & Debounce Demo").unwrap();
    writeln!(board.console, "Core: {}", board.core_name()).unwrap();

    let mut press_count = 0u32;
    let mut was_pressed = false;

    loop {
        let is_pressed = board.buttons.user.is_pressed();

        if is_pressed && !was_pressed {
            press_count += 1;
            writeln!(board.console, "Button PRESSED! Total count: {}", press_count).unwrap();

            match press_count % 3 {
                1 => board.leds.set_rgb(true, false, false), // Red
                2 => board.leds.set_rgb(false, true, false), // Green
                _ => board.leds.set_rgb(false, false, true), // Blue
            }
            board.delay_ms(50);
        } else if !is_pressed && was_pressed {
            writeln!(board.console, "Button RELEASED").unwrap();
            board.delay_ms(50);
        }

        was_pressed = is_pressed;
        board.delay_ms(10);
    }
}
