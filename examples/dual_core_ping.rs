#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - Dual-Core XRAM Ping-Pong").unwrap();
    writeln!(board.console, "Core: {}", board.core_name()).unwrap();

    let mut ping_count = 0u32;
    let mut rx_buf = [0u8; 64];

    loop {
        // Send a ping message with counter
        let mut msg = [0u8; 8];
        msg[0..4].copy_from_slice(&ping_count.to_le_bytes());
        let _ = board.xram.send(msg_type::PING, &msg[0..4]);
        board.leds.blue.toggle();

        writeln!(board.console, "Sent PING #{}", ping_count).unwrap();

        // Check if peer sent any message
        if board.xram.is_message_pending() {
            if let Ok(Some((hdr, len))) = board.xram.receive(&mut rx_buf) {
                writeln!(
                    board.console,
                    "Received message type: 0x{:02X}, len: {}",
                    hdr.msg_type, len
                )
                .unwrap();
                board.leds.green.toggle();
            }
        }

        ping_count = ping_count.wrapping_add(1);
        board.delay_ms(500);
    }
}
