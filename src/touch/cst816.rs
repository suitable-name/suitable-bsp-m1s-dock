//! CST816S Capacitive Touch Driver for Sipeed M1s Dock
//!
//! Connected via I2C at address `0x15` (SCL: GPIO 0, SDA: GPIO 1, INT: GPIO 2, RST: GPIO 9).

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::i2c::I2c;

/// Standard CST816 7-bit I2C device address.
pub const CST816_I2C_ADDR: u8 = 0x15;

/// Register map for CST816 touch controller.
#[allow(dead_code)]
mod reg {
    pub const GESTURE: u8 = 0x01;
    pub const POINTS: u8 = 0x02;
    pub const XPOS_H: u8 = 0x03;
    pub const CHIP_ID: u8 = 0xA7;
}

/// Gesture recognized by the CST816 touch controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    None,
    SlideDown,
    SlideUp,
    SlideLeft,
    SlideRight,
    SingleClick,
    DoubleClick,
    LongPress,
}

impl From<u8> for Gesture {
    fn from(val: u8) -> Self {
        match val {
            0x01 => Gesture::SlideDown,
            0x02 => Gesture::SlideUp,
            0x03 => Gesture::SlideLeft,
            0x04 => Gesture::SlideRight,
            0x05 => Gesture::SingleClick,
            0x0B => Gesture::DoubleClick,
            0x0C => Gesture::LongPress,
            _ => Gesture::None,
        }
    }
}

/// Touch event state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchEvent {
    Down,
    Up,
    Contact,
}

/// A parsed touch point from the CST816 controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
    pub gesture: Gesture,
    pub event: TouchEvent,
}

/// CST816S capacitive touch screen driver.
pub struct Cst816Touch<I2C, INT, RST> {
    i2c: I2C,
    int_pin: INT,
    rst_pin: RST,
    addr: u8,
}

impl<I2C, INT, RST> Cst816Touch<I2C, INT, RST>
where
    I2C: I2c,
    INT: InputPin,
    RST: OutputPin,
{
    /// Creates a new touch controller handle.
    pub fn new(i2c: I2C, int_pin: INT, rst_pin: RST) -> Self {
        Self {
            i2c,
            int_pin,
            rst_pin,
            addr: CST816_I2C_ADDR,
        }
    }

    /// Resets and initializes the touch controller.
    pub fn init<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I2C::Error> {
        // Hardware Reset: Low 20ms -> High 50ms
        let _ = self.rst_pin.set_low();
        delay.delay_ms(20);
        let _ = self.rst_pin.set_high();
        delay.delay_ms(50);

        Ok(())
    }

    /// Returns `true` if the touch controller is signaling an active touch via the INT pin (Active-Low).
    pub fn is_touching(&mut self) -> bool {
        self.int_pin.is_low().unwrap_or(false)
    }

    /// Reads the current touch point from the controller.
    ///
    /// Returns `Some(TouchPoint)` if a finger is touching, or `None` if released.
    pub fn read_touch(&mut self) -> Result<Option<TouchPoint>, I2C::Error> {
        let mut buf = [0u8; 6];
        // Read 6 registers starting at 0x01 (GESTURE through YPOS_L)
        self.i2c.write_read(self.addr, &[reg::GESTURE], &mut buf)?;

        let gesture = Gesture::from(buf[0]);
        let points = buf[1] & 0x0F;

        if points == 0 {
            return Ok(None);
        }

        let event = match (buf[2] >> 6) & 0x03 {
            0 => TouchEvent::Down,
            1 => TouchEvent::Up,
            _ => TouchEvent::Contact,
        };

        let x = (((buf[2] & 0x0F) as u16) << 8) | (buf[3] as u16);
        let y = (((buf[4] & 0x0F) as u16) << 8) | (buf[5] as u16);

        Ok(Some(TouchPoint {
            x,
            y,
            gesture,
            event,
        }))
    }

    /// Reads the chip ID register (typically `0xB4`, `0xB5`, or `0xB6`).
    pub fn read_chip_id(&mut self) -> Result<u8, I2C::Error> {
        let mut id = [0u8; 1];
        self.i2c.write_read(self.addr, &[reg::CHIP_ID], &mut id)?;
        Ok(id[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::digital::ErrorKind;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::vec;

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

    impl InputPin for MockPin {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(*self.state.borrow())
        }
        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(!*self.state.borrow())
        }
    }

    struct MockDelay;
    impl DelayNs for MockDelay {
        fn delay_ns(&mut self, _ns: u32) {}
    }

    #[derive(Clone, Default)]
    struct MockI2c {
        rx_buf: Rc<RefCell<std::vec::Vec<u8>>>,
        last_addr: Rc<RefCell<u8>>,
    }

    impl embedded_hal::i2c::ErrorType for MockI2c {
        type Error = embedded_hal::i2c::ErrorKind;
    }

    impl I2c for MockI2c {
        fn transaction(
            &mut self,
            address: u8,
            operations: &mut [embedded_hal::i2c::Operation<'_>],
        ) -> Result<(), Self::Error> {
            *self.last_addr.borrow_mut() = address;
            for op in operations {
                if let embedded_hal::i2c::Operation::Read(buf) = op {
                    let rx = self.rx_buf.borrow();
                    let len = buf.len().min(rx.len());
                    buf[..len].copy_from_slice(&rx[..len]);
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_cst816_init() {
        let i2c = MockI2c::default();
        let int_pin = MockPin::default();
        let rst_pin = MockPin::default();

        let mut touch = Cst816Touch::new(i2c, int_pin, rst_pin.clone());
        let mut delay = MockDelay;
        assert!(touch.init(&mut delay).is_ok());

        // RST finishes High after reset
        assert!(*rst_pin.state.borrow());
    }

    #[test]
    fn test_cst816_is_touching() {
        let i2c = MockI2c::default();
        let int_pin = MockPin::default();
        let rst_pin = MockPin::default();

        let mut touch = Cst816Touch::new(i2c, int_pin.clone(), rst_pin);

        // INT pin High -> Not touching
        *int_pin.state.borrow_mut() = true;
        assert!(!touch.is_touching());

        // INT pin Low -> Touching (active-low interrupt)
        *int_pin.state.borrow_mut() = false;
        assert!(touch.is_touching());
    }

    #[test]
    fn test_cst816_read_touch_none_when_released() {
        let i2c = MockI2c::default();
        let int_pin = MockPin::default();
        let rst_pin = MockPin::default();

        // 6 bytes with points = 0
        *i2c.rx_buf.borrow_mut() = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

        let mut touch = Cst816Touch::new(i2c, int_pin, rst_pin);
        let pt = touch.read_touch().unwrap();
        assert_eq!(pt, None);
    }

    #[test]
    fn test_cst816_read_touch_valid_point() {
        let i2c = MockI2c::default();
        let int_pin = MockPin::default();
        let rst_pin = MockPin::default();

        // Gesture: 0x01 (SlideDown)
        // Points: 0x01
        // Event: Down (0 << 6) | X_high (0x01): 0x01, X_low: 0x20 => X = 0x0120 = 288
        // Y_high: 0x00, Y_low: 0x50 => Y = 80
        *i2c.rx_buf.borrow_mut() = vec![0x01, 0x01, 0x01, 0x20, 0x00, 0x50];

        let mut touch = Cst816Touch::new(i2c, int_pin, rst_pin);
        let pt = touch.read_touch().unwrap().expect("Expected touch point");

        assert_eq!(pt.gesture, Gesture::SlideDown);
        assert_eq!(pt.event, TouchEvent::Down);
        assert_eq!(pt.x, 288);
        assert_eq!(pt.y, 80);
    }

    #[test]
    fn test_cst816_read_chip_id() {
        let i2c = MockI2c::default();
        let int_pin = MockPin::default();
        let rst_pin = MockPin::default();

        *i2c.rx_buf.borrow_mut() = vec![0xB5];

        let mut touch = Cst816Touch::new(i2c.clone(), int_pin, rst_pin);
        let id = touch.read_chip_id().unwrap();
        assert_eq!(id, 0xB5);
        assert_eq!(*i2c.last_addr.borrow(), CST816_I2C_ADDR);
    }
}
