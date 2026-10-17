#![no_std]
#![no_main]

use core::fmt::Write;
use suitable_bsp_m1s_dock::prelude::*;
use panic_halt as _;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    let mut board = Board::take().unwrap();

    writeln!(board.console, "Sipeed M1s Dock running on: {}", board.core_name()).unwrap();

    loop {
        // Cycle RGB LEDs
        board.leds.red.on();
        board.delay_ms(250);
        board.leds.red.off();

        board.leds.green.on();
        board.delay_ms(250);
        board.leds.green.off();

        board.leds.blue.on();
        board.delay_ms(250);
        board.leds.blue.off();

        if board.buttons.user.is_pressed() {
            writeln!(board.console, "User button is pressed!").unwrap();
        }
    }
}
