#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::camera::{Camera, PixelFormat, Resolution, GC0328_I2C_ADDR};
use suitable_bsp_m1s_dock::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock - Camera Frame Capture Demo").unwrap();
    writeln!(board.console, "Core: {}", board.core_name()).unwrap();

    #[cfg(feature = "d0")]
    if let Some(dvp0) = board.take_camera_dvp() {
        if let Some(parts) = board.touch_parts.take() {
            let i2c = bl808_hal::i2c::I2c::new(parts.i2c0, &board.clocks, 100_000);
            let mut camera = Camera::new(dvp0, i2c, GC0328_I2C_ADDR);
            camera.init(Resolution::Qvga, PixelFormat::Rgb565);
            camera.start();
            writeln!(board.console, "Camera DVP pipeline started: QVGA 320x240 RGB565").unwrap();
            board.leds.blue.on();
        }
    }

    loop {
        board.leds.red.toggle();
        board.delay_ms(500);
    }
}
