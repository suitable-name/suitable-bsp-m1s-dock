//! MicroSD Storage Subsystem for Sipeed M1s Dock
//!
//! Exposes the `SdCard` driver for 512-byte sector read/write on the on-board TF card slot.
//! On the Sipeed M1s Dock, the SD card is controlled by the SDH peripheral on the M0 core
//! using a 4-bit wide high-speed bus (CLK: P4, CMD: P5, DAT0-3: P0-P3).

#[cfg(feature = "m0")]
use bl808_hal::sdh::{SdhDriver, SdhError, SdhInstance};

/// Standard sector size for SD card block storage (512 bytes).
pub const SECTOR_SIZE: usize = 512;

/// Storage errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    /// Controller timeout waiting for card or transfer completion.
    Timeout,
    /// Hardware transmission or CRC error.
    HardwareError,
}

#[cfg(feature = "m0")]
impl From<SdhError> for StorageError {
    fn from(err: SdhError) -> Self {
        match err {
            SdhError::Timeout => StorageError::Timeout,
            SdhError::Error => StorageError::HardwareError,
        }
    }
}

/// Driver for the on-board MicroSD card slot.
#[cfg(feature = "m0")]
pub struct SdCard<SDH> {
    driver: SdhDriver<SDH>,
}

#[cfg(feature = "m0")]
impl<SDH: SdhInstance> SdCard<SDH> {
    /// Constructs a new SD Card driver handle.
    pub fn new(sdh: SDH) -> Self {
        Self {
            driver: SdhDriver::new(sdh),
        }
    }

    /// Reads a 512-byte sector from the given sector index.
    pub fn read_sector(&mut self, sector: u32, buf: &mut [u8; SECTOR_SIZE]) -> Result<(), StorageError> {
        self.driver.read_block(sector, buf)?;
        Ok(())
    }

    /// Writes a 512-byte sector to the given sector index.
    pub fn write_sector(&mut self, sector: u32, buf: &[u8; SECTOR_SIZE]) -> Result<(), StorageError> {
        self.driver.write_block(sector, buf)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_size_is_512() {
        assert_eq!(SECTOR_SIZE, 512);
    }

    #[cfg(feature = "m0")]
    #[test]
    fn test_error_mapping() {
        assert_eq!(StorageError::from(SdhError::Timeout), StorageError::Timeout);
        assert_eq!(StorageError::from(SdhError::Error), StorageError::HardwareError);
    }
}
