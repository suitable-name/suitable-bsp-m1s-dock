#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::audio::{MicChannel, SampleRate};
use suitable_bsp_m1s_dock::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - PDM Audio Record Demo").unwrap();
    writeln!(board.console, "Core: {}", board.core_name()).unwrap();

    board.mic.init(SampleRate::Rate16kHz, MicChannel::Left);
    board.mic.start();
    writeln!(board.console, "Microphone active @ 16kHz").unwrap();

    let mut buf = [0i16; 128];
    loop {
        let n = board.mic.read_buffer(&mut buf);
        if n > 0 {
            board.leds.green.toggle();
            writeln!(board.console, "Captured {} audio samples (sample[0]={})", n, buf[0]).unwrap();
        }
        board.delay_ms(100);
    }
}
