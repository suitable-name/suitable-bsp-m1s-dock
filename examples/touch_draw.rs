#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - Touch Drawing Demo").unwrap();
    writeln!(board.console, "Touch the screen to paint pixels!").unwrap();

    let mut display = board.init_display().expect("Failed to initialize display");
    let mut touch = board.init_touch().expect("Failed to initialize touch");

    // Clear display to black
    display.clear(0x0000);

    let colors: [u16; 5] = [0xF800, 0x07E0, 0x001F, 0xFFE0, 0xFFFF];
    let mut color_idx = 0;

    loop {
        if touch.is_touching() {
            if let Ok(Some(pt)) = touch.read_touch() {
                let color = colors[color_idx % colors.len()];
                // Draw a 6x6 square around the touch coordinate
                let x = pt.x.saturating_sub(3).min(234);
                let y = pt.y.saturating_sub(3).min(274);
                display.fill_rect(x, y, 6, 6, color);

                if pt.gesture == Gesture::SingleClick {
                    color_idx += 1;
                    board.leds.red.toggle();
                } else if pt.gesture == Gesture::LongPress {
                    display.clear(0x0000);
                    board.leds.blue.toggle();
                }
            }
        }

        board.delay_ms(10);
    }
}
