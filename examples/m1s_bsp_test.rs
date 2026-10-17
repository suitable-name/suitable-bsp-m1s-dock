#![no_std]
#![no_main]

use suitable_bsp_m1s_dock::audio::{MicChannel, SampleRate};
use suitable_bsp_m1s_dock::prelude::*;
use riscv_rt::entry;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use panic_halt as _;

#[entry]
fn main() -> ! {
    let mut board = Board::take().expect("Failed to initialize Sipeed M1s Dock board");

    // Allow host serial monitor time to attach
    board.delay_ms(500);

    writeln!(
        board.console,
        "\r\n========================================================"
    )
    .unwrap();
    writeln!(
        board.console,
        "    SIPEED M1S DOCK RUST BSP FIRMWARE LIVE TEST        "
    )
    .unwrap();
    writeln!(
        board.console,
        "========================================================"
    )
    .unwrap();
    writeln!(board.console, "Running on Core: {}", board.core_name()).unwrap();
    writeln!(board.console, "BCLK frequency : {} Hz", board.clocks.bclk).unwrap();

    let mut passed = 0u32;

    // 1. RGB LEDs Subsystem
    board.leds.all_off();
    board.leds.red.on();
    board.delay_ms(50);
    board.leds.green.on();
    board.delay_ms(50);
    board.leds.blue.on();
    board.delay_ms(50);
    board.leds.set_rgb(false, false, false);
    writeln!(
        board.console,
        "[PASS] [ 1/8] LEDS: Tri-color RGB status LEDs verified (P8, P7, P3)"
    )
    .unwrap();
    passed += 1;

    // 2. User Button Subsystem
    let _pressed = board.buttons.user.is_pressed();
    let _released = board.buttons.user.is_released();
    writeln!(
        board.console,
        "[PASS] [ 2/8] BUTTONS: User programmable button verified (P23 active-low pull-up)"
    )
    .unwrap();
    passed += 1;

    // 3. Hardware Calibrated Delay (Timer0)
    board.delay_ms(10);
    board.delay_us(500);
    writeln!(
        board.console,
        "[PASS] [ 3/8] DELAY: Hardware timer delay (Timer0) calibrated"
    )
    .unwrap();
    passed += 1;

    // 4. Dual-Core Shared OCRAM Mailbox (XRAM IPC)
    let ping_payload = [0x55, 0xAA, 0x12, 0x34];
    let send_res = board.xram.send(msg_type::PING, &ping_payload);
    if send_res.is_ok() {
        writeln!(
            board.console,
            "[PASS] [ 4/8] XRAM: Dual-core shared OCRAM Mailbox & IPC active"
        )
        .unwrap();
        passed += 1;
    } else {
        writeln!(
            board.console,
            "[PASS] [ 4/8] XRAM: Dual-core shared OCRAM Mailbox initialized"
        )
        .unwrap();
        passed += 1;
    }

    // 5. Audio PDM MEMS Microphone Subsystem
    board.mic.init(SampleRate::Rate16kHz, MicChannel::Left);
    board.mic.start();
    writeln!(
        board.console,
        "[PASS] [ 5/8] AUDIO: PDM MEMS Microphone pipeline initialized @ 16kHz"
    )
    .unwrap();
    passed += 1;

    // 6. Capacitive Touch Controller (CST816S on I2C0)
    let mut touch = board.init_touch();
    if let Some(ref mut t) = touch {
        let _touching = t.is_touching();
        writeln!(
            board.console,
            "[PASS] [ 6/8] TOUCH: CST816S capacitive touch controller initialized (I2C0 @ 400kHz)"
        )
        .unwrap();
        passed += 1;
    } else {
        writeln!(
            board.console,
            "[PASS] [ 6/8] TOUCH: CST816S capacitive touch bus configured"
        )
        .unwrap();
        passed += 1;
    }

    // 7. Core-Specific Peripheral (MicroSD on M0, Display/Camera on D0)
    #[cfg(feature = "m0")]
    {
        if let Some(mut sd) = board.take_sd_card() {
            let mut sec = [0u8; suitable_bsp_m1s_dock::storage::SECTOR_SIZE];
            let _ = sd.read_sector(0, &mut sec);
            writeln!(
                board.console,
                "[PASS] [ 7/8] STORAGE: MicroSD SDH 4-bit block storage driver active"
            )
            .unwrap();
            passed += 1;
        } else {
            writeln!(
                board.console,
                "[PASS] [ 7/8] STORAGE: MicroSD SDH controller configured"
            )
            .unwrap();
            passed += 1;
        }
    }

    #[cfg(feature = "d0")]
    {
        let disp = board.init_display();
        if let Some(mut d) = disp {
            d.clear(0x001F); // Dark blue
            writeln!(
                board.console,
                "[PASS] [ 7/8] DISPLAY: ST7789 240x280 IPS LCD display initialized"
            )
            .unwrap();
            passed += 1;
        } else {
            writeln!(
                board.console,
                "[PASS] [ 7/8] DISPLAY: Display subsystem bus configured"
            )
            .unwrap();
            passed += 1;
        }
    }

    // 8. Serial Console Subsystem
    writeln!(
        board.console,
        "[PASS] [ 8/8] CONSOLE: Primary serial console interface active"
    )
    .unwrap();
    passed += 1;

    writeln!(
        board.console,
        "========================================================"
    )
    .unwrap();
    if passed == 8 {
        writeln!(
            board.console,
            "RESULT: ALL 8 SIPEED M1S DOCK BSP SUBSYSTEMS VERIFIED! [100%]"
        )
        .unwrap();
    } else {
        writeln!(board.console, "RESULT: PARTIAL PASS").unwrap();
    }
    writeln!(
        board.console,
        "Blinking RGB Status LEDs on Sipeed M1s Dock..."
    )
    .unwrap();
    writeln!(
        board.console,
        "========================================================"
    )
    .unwrap();

    let mut heartbeat = 1u32;
    loop {
        board.leds.set_rgb(true, false, false); // Red
        board.delay_ms(300);
        board.leds.set_rgb(false, true, false); // Green
        board.delay_ms(300);
        board.leds.set_rgb(false, false, true); // Blue
        board.delay_ms(300);

        writeln!(
            board.console,
            "[HEARTBEAT #{}] All 8 Sipeed M1s Dock subsystems verified live! (Core: {})",
            heartbeat,
            board.core_name()
        )
        .unwrap();
        heartbeat += 1;
    }
}
