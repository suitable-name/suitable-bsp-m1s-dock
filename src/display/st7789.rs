//! ST7789V 1.69-inch 240×280 IPS LCD Driver for Sipeed M1s Dock
//!
//! Provides hardware-accelerated SPI drawing and optional `DrawTarget` support.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

/// Screen width in pixels.
pub const SCREEN_WIDTH: u16 = 240;
/// Screen height in pixels.
pub const SCREEN_HEIGHT: u16 = 280;

/// Default row offset (the 280-row panel is centered inside ST7789's 320-row controller buffer).
pub const DEFAULT_ROW_OFFSET: u16 = 20;

/// ST7789 Controller Commands
#[allow(dead_code)]
mod cmd {
    pub const SWRESET: u8 = 0x01;
    pub const SLPOUT: u8 = 0x11;
    pub const NORON: u8 = 0x13;
    pub const INVOFF: u8 = 0x20;
    pub const INVON: u8 = 0x21;
    pub const DISPOFF: u8 = 0x28;
    pub const DISPON: u8 = 0x29;
    pub const CASET: u8 = 0x2A;
    pub const RASET: u8 = 0x2B;
    pub const RAMWR: u8 = 0x2C;
    pub const MADCTL: u8 = 0x36;
    pub const COLMOD: u8 = 0x3A;
}

/// Display orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Portrait,
    Landscape,
    PortraitInverted,
    LandscapeInverted,
}

/// Driver for the on-board ST7789 LCD display.
pub struct St7789Display<SPI, CS, DC, RST, BL> {
    spi: SPI,
    cs: CS,
    dc: DC,
    rst: RST,
    bl: BL,
    row_offset: u16,
    col_offset: u16,
    width: u16,
    height: u16,
}

impl<SPI, CS, DC, RST, BL> St7789Display<SPI, CS, DC, RST, BL>
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
    BL: OutputPin,
{
    /// Creates a new ST7789 display driver without initializing the screen.
    pub fn new(spi: SPI, cs: CS, dc: DC, rst: RST, bl: BL) -> Self {
        Self {
            spi,
            cs,
            dc,
            rst,
            bl,
            row_offset: DEFAULT_ROW_OFFSET,
            col_offset: 0,
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
        }
    }

    /// Performs the hardware reset and ST7789 initialization sequence.
    pub fn init<D: DelayNs>(&mut self, delay: &mut D) {
        // Hardware Reset sequence: Low 20ms -> High 120ms
        let _ = self.rst.set_high();
        delay.delay_ms(10);
        let _ = self.rst.set_low();
        delay.delay_ms(20);
        let _ = self.rst.set_high();
        delay.delay_ms(120);

        // Turn on backlight
        let _ = self.bl.set_high();

        // Software reset & exit sleep
        self.write_command(cmd::SWRESET);
        delay.delay_ms(150);

        self.write_command(cmd::SLPOUT);
        delay.delay_ms(120);

        // 16-bit RGB565 color format (0x55)
        self.write_command(cmd::COLMOD);
        self.write_data(&[0x55]);

        // Memory access control (Orientation: Portrait)
        self.write_command(cmd::MADCTL);
        self.write_data(&[0x00]);

        // Display Inversion ON (required for IPS panels on M1s Dock)
        self.write_command(cmd::INVON);
        delay.delay_ms(10);

        // Normal display mode ON
        self.write_command(cmd::NORON);
        delay.delay_ms(10);

        // Display ON
        self.write_command(cmd::DISPON);
        delay.delay_ms(120);
    }

    /// Turns the LCD backlight ON or OFF.
    pub fn set_backlight(&mut self, on: bool) {
        if on {
            let _ = self.bl.set_high();
        } else {
            let _ = self.bl.set_low();
        }
    }

    /// Sets the drawing window address.
    pub fn set_window(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) {
        let x_start = x0 + self.col_offset;
        let x_end = x1 + self.col_offset;
        let y_start = y0 + self.row_offset;
        let y_end = y1 + self.row_offset;

        self.write_command(cmd::CASET);
        self.write_data(&[
            (x_start >> 8) as u8,
            (x_start & 0xFF) as u8,
            (x_end >> 8) as u8,
            (x_end & 0xFF) as u8,
        ]);

        self.write_command(cmd::RASET);
        self.write_data(&[
            (y_start >> 8) as u8,
            (y_start & 0xFF) as u8,
            (y_end >> 8) as u8,
            (y_end & 0xFF) as u8,
        ]);

        self.write_command(cmd::RAMWR);
    }

    /// Draws a single pixel at `(x, y)` with RGB565 color.
    pub fn draw_pixel(&mut self, x: u16, y: u16, color: u16) {
        if x >= self.width || y >= self.height {
            return;
        }
        self.set_window(x, y, x, y);
        let bytes = [(color >> 8) as u8, (color & 0xFF) as u8];
        self.write_data(&bytes);
    }

    /// Fills the entire screen with a solid RGB565 color.
    pub fn clear(&mut self, color: u16) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    /// Fills a rectangular area with a solid RGB565 color.
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u16) {
        if w == 0 || h == 0 || x >= self.width || y >= self.height {
            return;
        }
        let x1 = (x + w - 1).min(self.width - 1);
        let y1 = (y + h - 1).min(self.height - 1);
        self.set_window(x, y, x1, y1);

        let pixel_bytes = [(color >> 8) as u8, (color & 0xFF) as u8];
        let total_pixels = ((x1 - x + 1) as usize) * ((y1 - y + 1) as usize);

        let _ = self.cs.set_low();
        let _ = self.dc.set_high(); // Data mode
        for _ in 0..total_pixels {
            let _ = self.spi.write(&pixel_bytes);
        }
        let _ = self.cs.set_high();
    }

    /// Blits a buffer of 16-bit RGB565 pixels to a specified window.
    pub fn blit(&mut self, x: u16, y: u16, w: u16, h: u16, pixels: &[u16]) {
        if w == 0 || h == 0 || x >= self.width || y >= self.height {
            return;
        }
        let x1 = (x + w - 1).min(self.width - 1);
        let y1 = (y + h - 1).min(self.height - 1);
        self.set_window(x, y, x1, y1);

        let _ = self.cs.set_low();
        let _ = self.dc.set_high();
        for &pixel in pixels {
            let bytes = [(pixel >> 8) as u8, (pixel & 0xFF) as u8];
            let _ = self.spi.write(&bytes);
        }
        let _ = self.cs.set_high();
    }

    fn write_command(&mut self, cmd: u8) {
        let _ = self.cs.set_low();
        let _ = self.dc.set_low(); // Command mode
        let _ = self.spi.write(&[cmd]);
        let _ = self.cs.set_high();
    }

    fn write_data(&mut self, data: &[u8]) {
        let _ = self.cs.set_low();
        let _ = self.dc.set_high(); // Data mode
        let _ = self.spi.write(data);
        let _ = self.cs.set_high();
    }
}

#[cfg(feature = "graphics")]
impl<SPI, CS, DC, RST, BL> embedded_graphics_core::draw_target::DrawTarget
    for St7789Display<SPI, CS, DC, RST, BL>
where
    SPI: SpiBus<u8>,
    CS: OutputPin,
    DC: OutputPin,
    RST: OutputPin,
    BL: OutputPin,
{
    type Color = embedded_graphics_core::pixelcolor::Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics_core::Pixel<Self::Color>>,
    {
        for embedded_graphics_core::Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.x < self.width as i32 && coord.y >= 0 && coord.y < self.height as i32 {
                use embedded_graphics_core::pixelcolor::IntoStorage;
                let raw_color = color.into_storage();
                self.draw_pixel(coord.x as u16, coord.y as u16, raw_color);
            }
        }
        Ok(())
    }

    fn fill_solid(
        &mut self,
        area: &embedded_graphics_core::primitives::Rectangle,
        color: Self::Color,
    ) -> Result<(), Self::Error> {
        use embedded_graphics_core::geometry::Dimensions;
        use embedded_graphics_core::pixelcolor::IntoStorage;
        let intersection = area.intersection(&self.bounding_box());
        if !intersection.is_zero_sized() {
            let raw_color = color.into_storage();
            self.fill_rect(
                intersection.top_left.x as u16,
                intersection.top_left.y as u16,
                intersection.size.width as u16,
                intersection.size.height as u16,
                raw_color,
            );
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        use embedded_graphics_core::pixelcolor::IntoStorage;
        let raw_color = color.into_storage();
        self.clear(raw_color);
        Ok(())
    }
}

#[cfg(feature = "graphics")]
impl<SPI, CS, DC, RST, BL> embedded_graphics_core::geometry::OriginDimensions
    for St7789Display<SPI, CS, DC, RST, BL>
{
    fn size(&self) -> embedded_graphics_core::geometry::Size {
        embedded_graphics_core::geometry::Size::new(self.width as u32, self.height as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::digital::ErrorKind;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::vec;
    use std::vec::Vec;

    #[derive(Clone, Default)]
    struct MockPin {
        state: Rc<RefCell<bool>>,
    }

    impl embedded_hal::digital::ErrorType for MockPin {
        type Error = ErrorKind;
    }

    impl OutputPin for MockPin {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            *self.state.borrow_mut() = false;
            Ok(())
        }
        fn set_high(&mut self) -> Result<(), Self::Error> {
            *self.state.borrow_mut() = true;
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockSpi {
        transfers: Rc<RefCell<Vec<Vec<u8>>>>,
    }

    impl embedded_hal::spi::ErrorType for MockSpi {
        type Error = embedded_hal::spi::ErrorKind;
    }

    impl SpiBus<u8> for MockSpi {
        fn read(&mut self, _words: &mut [u8]) -> Result<(), Self::Error> {
            Ok(())
        }
        fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
            self.transfers.borrow_mut().push(words.to_vec());
            Ok(())
        }
        fn transfer(&mut self, _read: &mut [u8], _write: &[u8]) -> Result<(), Self::Error> {
            Ok(())
        }
        fn transfer_in_place(&mut self, _words: &mut [u8]) -> Result<(), Self::Error> {
            Ok(())
        }
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct MockDelay;
    impl DelayNs for MockDelay {
        fn delay_ns(&mut self, _ns: u32) {}
    }

    #[test]
    fn test_st7789_init_sequence() {
        let spi = MockSpi::default();
        let cs = MockPin::default();
        let dc = MockPin::default();
        let rst = MockPin::default();
        let bl = MockPin::default();

        let mut display = St7789Display::new(spi.clone(), cs, dc, rst, bl.clone());
        let mut delay = MockDelay;
        display.init(&mut delay);

        // Backlight turned on
        assert!(*bl.state.borrow());

        // Verify key commands sent
        let transfers = spi.transfers.borrow();
        assert!(transfers.contains(&vec![cmd::SWRESET]));
        assert!(transfers.contains(&vec![cmd::SLPOUT]));
        assert!(transfers.contains(&vec![cmd::COLMOD]));
        assert!(transfers.contains(&vec![0x55])); // 16-bit RGB565
        assert!(transfers.contains(&vec![cmd::INVON]));
        assert!(transfers.contains(&vec![cmd::NORON]));
        assert!(transfers.contains(&vec![cmd::DISPON]));
    }

    #[test]
    fn test_st7789_set_window_row_offset() {
        let spi = MockSpi::default();
        let cs = MockPin::default();
        let dc = MockPin::default();
        let rst = MockPin::default();
        let bl = MockPin::default();

        let mut display = St7789Display::new(spi.clone(), cs, dc, rst, bl);
        display.set_window(10, 20, 30, 40);

        let transfers = spi.transfers.borrow();
        // CASET: x0=10, x1=30
        assert!(transfers.contains(&vec![cmd::CASET]));
        assert!(transfers.contains(&vec![0, 10, 0, 30]));
        // RASET: y0=20 + 20 offset = 40, y1=40 + 20 offset = 60
        assert!(transfers.contains(&vec![cmd::RASET]));
        assert!(transfers.contains(&vec![0, 40, 0, 60]));
        assert!(transfers.contains(&vec![cmd::RAMWR]));
    }

    #[test]
    fn test_st7789_draw_pixel() {
        let spi = MockSpi::default();
        let cs = MockPin::default();
        let dc = MockPin::default();
        let rst = MockPin::default();
        let bl = MockPin::default();

        let mut display = St7789Display::new(spi.clone(), cs, dc, rst, bl);
        display.draw_pixel(5, 5, 0xF800); // Red (high byte 0xF8, low byte 0x00)

        let transfers = spi.transfers.borrow();
        assert!(transfers.contains(&vec![0xF8, 0x00]));
    }
}
