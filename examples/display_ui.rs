#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - ST7789 Display UI Demo").unwrap();
    writeln!(board.console, "Core: {}", board.core_name()).unwrap();

    // Initialize display with hardware reset and default configuration
    let mut display = board.init_display().expect("Failed to take display");

    writeln!(board.console, "Display initialized: 240x280 IPS LCD").unwrap();

    // Fill screen with dark blue
    display.clear(0x001F);
    board.delay_ms(500);

    // Color bars: Red, Green, Blue, Yellow, Cyan, Magenta, White
    let colors: [u16; 7] = [
        0xF800, // Red
        0x07E0, // Green
        0x001F, // Blue
        0xFFE0, // Yellow
        0x07FF, // Cyan
        0xF81F, // Magenta
        0xFFFF, // White
    ];

    let bar_height = 280 / colors.len() as u16;
    for (i, &color) in colors.iter().enumerate() {
        let y = i as u16 * bar_height;
        display.fill_rect(0, y, 240, bar_height, color);
    }

    board.delay_ms(1000);

    // Interactive loop: Blink LED and animate a bouncing rectangle
    let mut rect_x: u16 = 20;
    let mut rect_y: u16 = 20;
    let mut dx: i16 = 2;
    let mut dy: i16 = 3;
    let box_size: u16 = 40;

    display.clear(0x0000); // Black

    loop {
        board.leds.blue.toggle();

        // Clear old box position
        display.fill_rect(rect_x, rect_y, box_size, box_size, 0x0000);

        // Update box position
        if (rect_x as i16 + dx) < 0 || (rect_x + box_size + dx as u16) >= 240 {
            dx = -dx;
            board.leds.red.toggle();
        }
        if (rect_y as i16 + dy) < 0 || (rect_y + box_size + dy as u16) >= 280 {
            dy = -dy;
            board.leds.green.toggle();
        }

        rect_x = (rect_x as i16 + dx) as u16;
        rect_y = (rect_y as i16 + dy) as u16;

        // Draw new box in bright yellow
        display.fill_rect(rect_x, rect_y, box_size, box_size, 0xFFE0);

        board.delay_ms(20);
    }
}
