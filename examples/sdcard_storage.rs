#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::prelude::*;
use suitable_bsp_m1s_dock::storage::SECTOR_SIZE;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - MicroSD Storage Demo").unwrap();
    writeln!(board.console, "Core: {}", board.core_name()).unwrap();

    #[cfg(feature = "m0")]
    if let Some(mut sd) = board.take_sd_card() {
        let mut sector = [0u8; SECTOR_SIZE];
        match sd.read_sector(0, &mut sector) {
            Ok(()) => {
                writeln!(
                    board.console,
                    "Read sector 0 successfully! Boot signature: 0x{:02X}{:02X}",
                    sector[510], sector[511]
                )
                .unwrap();
                board.leds.green.on();
            }
            Err(e) => {
                writeln!(board.console, "SD read error: {:?}", e).unwrap();
                board.leds.red.on();
            }
        }
    }

    loop {
        board.leds.blue.toggle();
        board.delay_ms(1000);
    }
}
