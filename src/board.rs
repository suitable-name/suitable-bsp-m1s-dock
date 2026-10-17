//! Board Singleton and Initialization for Sipeed M1s Dock
//!
//! Exposes the [`Board`] struct containing pre-configured on-board peripherals:
//! - RGB Status LEDs ([`Leds`])
//! - User Button ([`Buttons`])
//! - Serial Console ([`Console`])
//! - Calibrated Hardware Delay ([`Timer`])
//! - System Clock Token ([`Clocks`])
//! - PDM MEMS Microphone ([`Microphone`])
//! - Dual-Core XRAM Mailbox ([`XramMailbox`])
//! - Optional 1.69" IPS LCD display (`M1sDisplay` on D0)
//! - Optional CST816S capacitive touch ([`M1sTouch`])

use crate::audio::Microphone;
use crate::button::Buttons;
use crate::console::Console;
#[cfg(feature = "d0")]
use crate::display::St7789Display;
use crate::led::Leds;
use crate::touch::Cst816Touch;
use crate::xram::XramMailbox;
use bl808_hal::glb::{Clocks, GlbExt};
use bl808_hal::gpio::{Alternate, Input, Output, Pin, PullUp, PushPull};
use bl808_hal::timer::Timer;
use bl808_hal::uart::Serial;
use core::sync::atomic::{AtomicBool, Ordering};

static TAKEN: AtomicBool = AtomicBool::new(false);

/// Pre-configured on-board ST7789 display type for the Sipeed M1s Dock.
#[cfg(feature = "d0")]
pub type M1sDisplay = St7789Display<
    bl808_hal::spi::Spi<bl808_pac::Spi0>,
    Pin<12, Output<PushPull>>,
    Pin<11, Output<PushPull>>,
    Pin<10, Output<PushPull>>,
    Pin<21, Output<PushPull>>,
>;

/// Hardware resources reserved for initializing the on-board ST7789 display.
#[cfg(feature = "d0")]
pub struct DisplayParts {
    pub spi0: bl808_pac::Spi0,
    pub cs: Pin<12, Output<PushPull>>,
    pub sck: Pin<13, Alternate<1>>,
    pub mosi: Pin<14, Alternate<1>>,
    pub dc: Pin<11, Output<PushPull>>,
    pub rst: Pin<10, Output<PushPull>>,
    pub bl: Pin<21, Output<PushPull>>,
}

/// Pre-configured on-board CST816S capacitive touch controller type for the Sipeed M1s Dock.
pub type M1sTouch = Cst816Touch<
    bl808_hal::i2c::I2c<bl808_pac::I2c0>,
    Pin<2, Input<PullUp>>,
    Pin<9, Output<PushPull>>,
>;

/// Hardware resources reserved for initializing the on-board CST816S touch controller.
pub struct TouchParts {
    pub i2c0: bl808_pac::I2c0,
    pub scl: Pin<0, Alternate<4>>,
    pub sda: Pin<1, Alternate<4>>,
    pub int_pin: Pin<2, Input<PullUp>>,
    pub rst_pin: Pin<9, Output<PushPull>>,
}

/// Board abstraction for the Sipeed M1s Dock.
pub struct Board {
    /// On-board 3-color RGB status LEDs.
    pub leds: Leds,
    /// On-board user button.
    pub buttons: Buttons,
    /// Primary serial debug console.
    pub console: Console,
    /// Hardware timer for microsecond and millisecond delays.
    pub delay: Timer<bl808_pac::Timer0>,
    /// Clock configuration token.
    pub clocks: Clocks,
    /// On-board PDM MEMS microphone driver.
    pub mic: Microphone,
    /// Dual-core XRAM mailbox for communication with peer core.
    pub xram: XramMailbox,
    /// On-board display hardware parts (takes on demand).
    #[cfg(feature = "d0")]
    pub display_parts: Option<DisplayParts>,
    /// On-board capacitive touch hardware parts (takes on demand).
    pub touch_parts: Option<TouchParts>,
    /// Camera DVP hardware peripheral (D0 only).
    #[cfg(feature = "d0")]
    dvp0: Option<bl808_pac::Dvp0>,
    /// MicroSD SDH hardware peripheral (M0 only).
    #[cfg(feature = "m0")]
    sdh: Option<bl808_pac::Sdh>,
}

impl Board {
    /// Obtains exclusive ownership of the Sipeed M1s Dock board peripherals.
    ///
    /// Returns `Some(Board)` on the first call, or `None` if the board has already been taken.
    pub fn take() -> Option<Self> {
        if TAKEN.swap(true, Ordering::SeqCst) {
            return None;
        }

        let dp = bl808_pac::Peripherals::take()?;
        Some(Self::init_with_peripherals(dp))
    }

    /// Steals the board peripherals without singleton checks.
    ///
    /// # Safety
    /// Calling this function bypasses the singleton guarantee and can lead to
    /// multiple concurrent handles to hardware peripherals.
    pub unsafe fn steal() -> Self {
        let dp = bl808_pac::Peripherals::steal();
        Self::init_with_peripherals(dp)
    }

    /// Internal constructor initializing the hardware peripherals for the target core.
    fn init_with_peripherals(dp: bl808_pac::Peripherals) -> Self {
        let (glb_control, pins) = dp.glb.split();
        let clocks = glb_control.get_clocks();

        // 1. Configure RGB LEDs (Active-Low)
        let leds = Leds::new(
            pins.P8.into_push_pull_output(),
            pins.P7.into_push_pull_output(),
            pins.P3.into_push_pull_output(),
        );

        // 2. Configure User Button (GPIO 23 with pull-up)
        let buttons = Buttons::new(pins.P23.into_pull_up_input());

        // 3. Configure Serial Console
        #[cfg(feature = "d0")]
        let console = {
            let tx = pins.P16.into_alternate::<7>();
            let rx = pins.P17.into_alternate::<7>();
            let serial = Serial::new(dp.uart3, tx, rx, 2_000_000, clocks.bclk);
            Console::new(serial)
        };

        #[cfg(feature = "m0")]
        let console = {
            let tx = pins.P14.into_alternate::<7>();
            let rx = pins.P15.into_alternate::<7>();
            let serial = Serial::new(dp.uart0, tx, rx, 2_000_000, clocks.bclk);
            Console::new(serial)
        };

        // 4. Initialize Delay Timer
        let delay = Timer::new(dp.timer0);

        // 5. Reserve Display Parts (D0 only: SPI0 on P13/P14)
        #[cfg(feature = "d0")]
        let display_parts = Some(DisplayParts {
            spi0: dp.spi0,
            cs: pins.P12.into_push_pull_output(),
            sck: pins.P13.into_alternate::<1>(),
            mosi: pins.P14.into_alternate::<1>(),
            dc: pins.P11.into_push_pull_output(),
            rst: pins.P10.into_push_pull_output(),
            bl: pins.P21.into_push_pull_output(),
        });

        // 6. Reserve Touch Parts (I2C0 on P0/P1)
        let touch_parts = Some(TouchParts {
            i2c0: dp.i2c0,
            scl: pins.P0.into_alternate::<4>(),
            sda: pins.P1.into_alternate::<4>(),
            int_pin: pins.P2.into_pull_up_input(),
            rst_pin: pins.P9.into_push_pull_output(),
        });

        // 7. Initialize Audio Microphone & Dual-Core XRAM Mailbox
        let mic = Microphone::new();
        let xram = XramMailbox::new(dp.ipc);

        Self {
            leds,
            buttons,
            console,
            delay,
            clocks,
            mic,
            xram,
            #[cfg(feature = "d0")]
            display_parts,
            touch_parts,
            #[cfg(feature = "d0")]
            dvp0: Some(dp.dvp0),
            #[cfg(feature = "m0")]
            sdh: Some(dp.sdh),
        }
    }

    /// Takes ownership of the display peripheral and pins, configuring SPI0 at 40 MHz.
    #[cfg(feature = "d0")]
    pub fn take_display(&mut self) -> Option<M1sDisplay> {
        let parts = self.display_parts.take()?;
        let spi = bl808_hal::spi::Spi::new(
            parts.spi0,
            &self.clocks,
            40_000_000,
            bl808_hal::spi::Polarity::IdleLow,
            bl808_hal::spi::Phase::CaptureOnFirstTransition,
            bl808_hal::spi::BitOrder::MsbFirst,
        );
        Some(St7789Display::new(
            spi,
            parts.cs,
            parts.dc,
            parts.rst,
            parts.bl,
        ))
    }

    /// Initializes the on-board ST7789 display (resets the screen, turns on backlight, exits sleep).
    #[cfg(feature = "d0")]
    pub fn init_display(&mut self) -> Option<M1sDisplay> {
        let mut display = self.take_display()?;
        display.init(&mut self.delay);
        Some(display)
    }

    /// Takes ownership of the capacitive touch peripheral and pins, configuring I2C0.
    pub fn take_touch(&mut self) -> Option<M1sTouch> {
        let parts = self.touch_parts.take()?;
        let i2c = bl808_hal::i2c::I2c::new(parts.i2c0, &self.clocks, 400_000);
        Some(Cst816Touch::new(i2c, parts.int_pin, parts.rst_pin))
    }

    /// Initializes the on-board CST816S touch controller (performs hardware reset).
    pub fn init_touch(&mut self) -> Option<M1sTouch> {
        let mut touch = self.take_touch()?;
        let _ = touch.init(&mut self.delay);
        Some(touch)
    }

    /// Takes ownership of the camera DVP0 peripheral (D0 only).
    #[cfg(feature = "d0")]
    pub fn take_camera_dvp(&mut self) -> Option<bl808_pac::Dvp0> {
        self.dvp0.take()
    }

    /// Takes ownership of the MicroSD card block device (M0 only).
    #[cfg(feature = "m0")]
    pub fn take_sd_card(&mut self) -> Option<crate::storage::SdCard<bl808_pac::Sdh>> {
        let sdh = self.sdh.take()?;
        Some(crate::storage::SdCard::new(sdh))
    }

    /// Returns the human-readable core name this firmware was compiled for.
    pub fn core_name(&self) -> &'static str {
        #[cfg(feature = "d0")]
        return "C906 D0 (64-bit Application Core)";

        #[cfg(feature = "m0")]
        return "E907 M0 (32-bit Wireless/Real-time Core)";
    }

    /// Calibrated delay in milliseconds.
    pub fn delay_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms);
    }

    /// Calibrated delay in microseconds.
    pub fn delay_us(&mut self, us: u32) {
        self.delay.delay_us(us);
    }
}
