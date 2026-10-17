# `m1s-dock-bsp`

[![Crates.io](https://img.shields.io/crates/v/suitable-bsp-m1s-dock.svg)](https://crates.io/crates/suitable-bsp-m1s-dock)
[![Documentation](https://docs.rs/suitable-bsp-m1s-dock/badge.svg)](https://docs.rs/suitable-bsp-m1s-dock)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

Production-ready `#![no_std]` **Board Support Package (BSP)** for the **Sipeed M1s Dock** development board, powered by the **Bouffalo Lab BL808** dual-core RISC-V SoC.

Builds directly on [`bl808-hal`](https://github.com/bouffalolab/bl808-hal) and [`bl808-pac`](https://github.com/bouffalolab/bl808-pac), providing high-level, safe, and idiomatic Rust drivers for all on-board components.

---

## Features

- **RGB Status LEDs**: High-level control for on-board Red (`GPIO 8`), Green (`GPIO 7`), and Blue (`GPIO 3`) LEDs.
- **User Button**: Debounced state and edge querying for the on-board button (`GPIO 23`).
- **Serial Debug Console**: Automatic console routing over USB-C (UART3 at 2,000,000 baud on D0; UART0 at 115,200 baud on M0) implementing `core::fmt::Write`.
- **1.69" 240×280 IPS LCD Display**: Hardware-accelerated ST7789 SPI driver with row-offset alignment, color fills, fast blitting, and full `embedded-graphics` [`DrawTarget`](https://docs.rs/embedded-graphics) support.
- **CST816S Capacitive Touch Screen**: Multi-gesture recognition (Swipe Up/Down/Left/Right, Single Click, Double Click, Long Press) and coordinate reporting over fast-mode I2C.
- **Audio Subsystem**: On-board PDM MEMS microphone driver with 16-bit PCM streaming and speaker DAC interface.
- **Camera Subsystem**: DVP parallel interface driver for GC0328 / OV2640 VGA camera modules with DMA double-buffering.
- **MicroSD Storage**: 512-byte block device driver for on-board TF card slot via SDH 4-bit bus on M0.
- **Dual-Core XRAM Mailbox**: Pure-Rust ring-buffer IPC protocol over shared OCRAM (`0x4203_0000`) for cooperative D0 (64-bit C906) and M0 (32-bit E907) execution.

---

## Hardware Pinout Matrix

| Subsystem | On-Board Hardware | Target Core | Primary Pins / Bus | Configuration |
| :--- | :--- | :--- | :--- | :--- |
| **RGB LEDs** | 3-Color Status LED | D0 / M0 | Red: `GPIO 8`<br>Green: `GPIO 7`<br>Blue: `GPIO 3` | Active-Low push-pull |
| **User Input** | User Button | D0 / M0 | `GPIO 23` | Active-Low with internal pull-up |
| **Serial Console** | USB-C UART (CH343) | D0 (Default)<br>M0 | TX: `GPIO 16`, RX: `GPIO 17`<br>TX: `GPIO 14`, RX: `GPIO 15` | 2,000,000 baud (D0)<br>115,200 baud (M0) |
| **Display** | 1.69" 240×280 IPS LCD | D0 | CS: `P12`, SCK: `P13`, MOSI: `P14`<br>DC: `P11`, RST: `P10`, BL: `P21` | SPI0 at 40 MHz, PWM backlight |
| **Touch** | CST816S Capacitive Touch | D0 / M0 | SCL: `P0`, SDA: `P1`<br>INT: `P2`, RST: `P9` | I2C0 at 400 kHz |
| **Microphone** | PDM MEMS Microphone | D0 / M0 | Dedicated PDM interface (`0x2000_A800`) | 16 kHz – 48 kHz 16-bit PCM |
| **Camera** | GC0328 / OV2640 DVP | D0 | D0–D7: `P24..31`<br>PCLK, VSYNC, HREF, XCLK | DVP parallel + DMA buffer |
| **MicroSD** | TF Card Slot | M0 | CLK: `P4`, CMD: `P5`<br>DAT0–3: `P0..3` | SDH 4-bit bus |
| **Dual-Core IPC** | Shared OCRAM Mailbox | D0 ↔ M0 | Dedicated Hardware IPC channels | Ring buffer at `0x4203_0000` |

---

## Quickstart

Add `m1s-dock-bsp` to your `Cargo.toml`:

```toml
[dependencies]
m1s-dock-bsp = { version = "0.1.0", features = ["rt", "graphics"] }
```

### Basic Blinky & Serial Console

```rust
#![no_std]
#![no_main]

use m1s_dock_bsp::prelude::*;
use riscv_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize board");

    writeln!(board.console, "Sipeed M1s Dock ready! Core: {}", board.core_name()).unwrap();

    loop {
        board.leds.red.toggle();
        board.delay_ms(500);

        if board.buttons.user.is_pressed() {
            board.leds.blue.toggle();
        }
    }
}
```

### 1.69" ST7789 Display UI

```rust
let mut board = Board::take().unwrap();
let mut display = board.init_display().unwrap();

// Clear screen to blue
display.clear(0x001F);

// Draw color rectangle
display.fill_rect(20, 20, 100, 100, 0xF800); // Red
```

### CST816S Capacitive Touch

```rust
let mut touch = board.init_touch().unwrap();

loop {
    if touch.is_touching() {
        if let Ok(Some(pt)) = touch.read_touch() {
            writeln!(board.console, "Touch at ({}, {}), gesture: {:?}", pt.x, pt.y, pt.gesture).unwrap();
        }
    }
    board.delay_ms(10);
}
```

---

## Flashing & Running

Firmware binaries can be flashed to the M1s Dock using Bouffalo Lab's `bflb-mcu-tool`:

```bash
# 1. Build the release binary
cargo build --example display_ui --target riscv64imac-unknown-none-elf --release

# 2. Convert ELF to binary
rust-objcopy -O binary target/riscv64imac-unknown-none-elf/release/examples/display_ui firmware.bin

# 3. Flash to M1s Dock via USB-C
bflb-mcu-tool --chip=bl808 --port=COM3 --baudrate=2000000 --pt=partition.bin --firmware=firmware.bin
```

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
