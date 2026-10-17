//! Camera Subsystem for Sipeed M1s Dock
//!
//! Provides DVP (Digital Video Port) parallel capture for GC0328 / OV2640 camera sensors
//! with hardware-accelerated DMA frame buffering.

use bl808_hal::dvp::{Config as DvpConfig, Dvp, Polarity};
use bl808_pac::Dvp0;
use embedded_hal::i2c::I2c;

/// Standard I2C / SCCB address for GC0328 camera sensor.
pub const GC0328_I2C_ADDR: u8 = 0x21;

/// Standard I2C / SCCB address for OV2640 camera sensor.
pub const OV2640_I2C_ADDR: u8 = 0x30;

/// Camera capture resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Full VGA resolution (640 × 480).
    Vga,
    /// Quarter VGA resolution (320 × 240).
    Qvga,
    /// Display-matched resolution (240 × 280) for the 1.69" LCD.
    LcdFit,
}

impl Resolution {
    /// Returns the (width, height) in pixels.
    pub const fn dimensions(&self) -> (u16, u16) {
        match self {
            Resolution::Vga => (640, 480),
            Resolution::Qvga => (320, 240),
            Resolution::LcdFit => (240, 280),
        }
    }

    /// Returns the frame size in bytes for 16-bit RGB565 / YUV422 pixels.
    pub const fn frame_bytes(&self) -> usize {
        let (w, h) = self.dimensions();
        (w as usize) * (h as usize) * 2
    }
}

/// Camera pixel format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// 16-bit RGB565 (5 bits red, 6 bits green, 5 bits blue).
    Rgb565,
    /// 16-bit YUV422 (YUYV packed).
    Yuv422,
}

/// Camera subsystem errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraError {
    /// Sensor SCCB / I2C communication error.
    SensorCommError,
    /// DMA transfer timed out waiting for frame sync.
    Timeout,
    /// Invalid buffer configuration.
    InvalidBuffer,
}

/// Camera driver for the Sipeed M1s Dock.
pub struct Camera<I2C> {
    dvp: Dvp,
    sccb: I2C,
    sensor_addr: u8,
    resolution: Resolution,
    format: PixelFormat,
}

impl<I2C: I2c> Camera<I2C> {
    /// Creates a new Camera driver instance.
    pub fn new(dvp0: Dvp0, sccb: I2C, sensor_addr: u8) -> Self {
        Self {
            dvp: Dvp::new(dvp0),
            sccb,
            sensor_addr,
            resolution: Resolution::Qvga,
            format: PixelFormat::Rgb565,
        }
    }

    /// Initializes the DVP pipeline for the requested resolution and pixel format.
    pub fn init(&mut self, res: Resolution, format: PixelFormat) {
        self.resolution = res;
        self.format = format;
        let (w, h) = res.dimensions();

        let config = DvpConfig {
            hsync_polarity: Polarity::ActiveHigh,
            vsync_polarity: Polarity::ActiveLow,
            pclk_polarity: Polarity::ActiveHigh,
            width: w,
            height: h,
        };

        self.dvp.init(config);
    }

    /// Starts streaming frames into memory.
    pub fn start(&mut self) {
        self.dvp.enable();
    }

    /// Stops streaming frames.
    pub fn stop(&mut self) {
        self.dvp.disable();
    }

    /// Configures single-buffer DMA frame destination.
    ///
    /// # Safety
    /// `dest_addr` must point to a physically contiguous buffer large enough
    /// to hold `resolution.frame_bytes()` bytes.
    pub unsafe fn set_frame_buffer(&mut self, dest_addr: u32) {
        self.dvp.set_frame_buffer(dest_addr);
    }

    /// Configures ping-pong double buffers for continuous DMA capture without tearing.
    ///
    /// # Safety
    /// Both `addr0` and `addr1` must point to valid buffers of `resolution.frame_bytes()`.
    pub unsafe fn set_frame_buffers(&mut self, addr0: u32, addr1: u32, burst_bytes: u32) {
        let frame_bytes = self.resolution.frame_bytes() as u32;
        self.dvp.set_frame_buffers(addr0, addr1, frame_bytes, burst_bytes);
    }

    /// Returns `true` if at least one completed frame is available in the DVP FIFO.
    pub fn is_frame_ready(&self) -> bool {
        self.dvp.frame_valid_count() > 0
    }

    /// Returns the memory address of the oldest completed frame in the FIFO.
    pub fn current_frame_addr(&self) -> u32 {
        self.dvp.current_frame_addr()
    }

    /// Advances the DVP hardware FIFO and clears interrupt flags after processing a frame.
    pub fn pop_frame(&mut self) {
        self.dvp.pop_frame();
        self.dvp.clear_frame_interrupt();
    }

    /// Writes an 8-bit register to the camera sensor via SCCB/I2C.
    pub fn write_sensor_reg(&mut self, reg: u8, val: u8) -> Result<(), CameraError> {
        self.sccb
            .write(self.sensor_addr, &[reg, val])
            .map_err(|_| CameraError::SensorCommError)
    }

    /// Reads an 8-bit register from the camera sensor via SCCB/I2C.
    pub fn read_sensor_reg(&mut self, reg: u8) -> Result<u8, CameraError> {
        let mut buf = [0u8; 1];
        self.sccb
            .write_read(self.sensor_addr, &[reg], &mut buf)
            .map_err(|_| CameraError::SensorCommError)?;
        Ok(buf[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolutions() {
        assert_eq!(Resolution::Vga.dimensions(), (640, 480));
        assert_eq!(Resolution::Vga.frame_bytes(), 640 * 480 * 2);

        assert_eq!(Resolution::Qvga.dimensions(), (320, 240));
        assert_eq!(Resolution::Qvga.frame_bytes(), 320 * 240 * 2);

        assert_eq!(Resolution::LcdFit.dimensions(), (240, 280));
        assert_eq!(Resolution::LcdFit.frame_bytes(), 240 * 280 * 2);
    }

    #[test]
    fn test_sensor_addresses() {
        assert_eq!(GC0328_I2C_ADDR, 0x21);
        assert_eq!(OV2640_I2C_ADDR, 0x30);
    }
}
