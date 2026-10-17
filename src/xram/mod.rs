//! Dual-Core Inter-Processor Communication (IPC) & XRAM Mailbox
//!
//! Implements the shared-memory ring buffer protocol between the 64-bit application core (D0)
//! and the 32-bit wireless/real-time core (M0) on the Sipeed M1s Dock.
//!
//! # Shared Memory Layout
//! - OCRAM Base Address: `0x4203_0000`
//! - D0 to M0 Channel (Tx on D0, Rx on M0): Offset `0x0000`, 4 KB ring
//! - M0 to D0 Channel (Tx on M0, Rx on D0): Offset `0x1000`, 4 KB ring

use bl808_hal::ipc::Ipc;
use bl808_pac::Ipc as PacIpc;
use core::sync::atomic::{compiler_fence, Ordering};

/// Base address of the shared XRAM mailbox in BL808 OCRAM.
pub const XRAM_BASE_ADDR: usize = 0x4203_0000;

/// Channel buffer capacity in bytes (4 KB per direction).
pub const CHANNEL_BUFFER_SIZE: usize = 4096;

/// Standard Sipeed XRAM message types.
pub mod msg_type {
    /// Inter-core virtual UART / serial console messages.
    pub const UARTP: u8 = 0x80;
    /// Inter-core audio sample streaming.
    pub const AUDIO: u8 = 0x81;
    /// Wireless WiFi control commands (Scan, Connect, Status, IP).
    pub const WIFI: u8 = 0x82;
    /// Raw Ethernet network packet exchange.
    pub const NET: u8 = 0x83;
    /// Ping / keepalive test message.
    pub const PING: u8 = 0x8F;
}

/// XRAM packet header preceding every message in the ring buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct PacketHeader {
    /// Message type identifier (e.g. [`msg_type::WIFI`]).
    pub msg_type: u8,
    /// Message flags / error status.
    pub flags: u8,
    /// Length of payload following this header in bytes.
    pub len: u16,
}

/// XRAM communication error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XramError {
    /// The ring buffer does not have enough free space for the packet.
    BufferFull,
    /// The received packet header is corrupt or truncated.
    InvalidPacket,
    /// The requested payload exceeds channel capacity.
    PayloadTooLarge,
}

/// A unidirectional ring buffer residing in shared OCRAM.
pub struct XramRingBuffer {
    base: usize,
    capacity: usize,
}

impl XramRingBuffer {
    /// Creates a ring buffer handle mapped to a specific physical address.
    ///
    /// # Safety
    /// `base` must point to a valid, exclusive or shared memory region of `capacity` bytes.
    pub const unsafe fn new(base: usize, capacity: usize) -> Self {
        Self { base, capacity }
    }

    /// Writes a packet with the given header and payload into the ring buffer.
    pub fn push(&mut self, header: PacketHeader, payload: &[u8]) -> Result<(), XramError> {
        let total_size = core::mem::size_of::<PacketHeader>() + payload.len();
        if total_size > self.capacity {
            return Err(XramError::PayloadTooLarge);
        }

        let hdr_bytes = unsafe {
            core::slice::from_raw_parts(
                &header as *const PacketHeader as *const u8,
                core::mem::size_of::<PacketHeader>(),
            )
        };

        // Write header
        let dest = self.base as *mut u8;
        unsafe {
            core::ptr::copy_nonoverlapping(hdr_bytes.as_ptr(), dest, hdr_bytes.len());
            core::ptr::copy_nonoverlapping(payload.as_ptr(), dest.add(hdr_bytes.len()), payload.len());
        }

        // Memory barrier ensuring data is visible to other core before IPC interrupt
        compiler_fence(Ordering::SeqCst);
        Ok(())
    }

    /// Reads a packet header and payload out of the ring buffer.
    pub fn pop(&mut self, buf: &mut [u8]) -> Result<Option<(PacketHeader, usize)>, XramError> {
        let dest = self.base as *const u8;
        let mut header = PacketHeader {
            msg_type: 0,
            flags: 0,
            len: 0,
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                dest,
                &mut header as *mut PacketHeader as *mut u8,
                core::mem::size_of::<PacketHeader>(),
            );
        }

        if header.msg_type == 0 {
            return Ok(None);
        }

        let payload_len = header.len as usize;
        if payload_len > buf.len() {
            return Err(XramError::PayloadTooLarge);
        }

        unsafe {
            core::ptr::copy_nonoverlapping(
                dest.add(core::mem::size_of::<PacketHeader>()),
                buf.as_mut_ptr(),
                payload_len,
            );
        }

        Ok(Some((header, payload_len)))
    }
}

/// Dual-Core Mailbox for the Sipeed M1s Dock.
pub struct XramMailbox {
    ipc: Ipc,
    tx_ring: XramRingBuffer,
    rx_ring: XramRingBuffer,
}

impl XramMailbox {
    /// Initializes the mailbox channels using default shared OCRAM offsets.
    pub fn new(ipc: PacIpc) -> Self {
        #[cfg(feature = "d0")]
        unsafe {
            Self {
                ipc: Ipc::new(ipc),
                // D0 Tx is at 0x4203_0000, Rx is at 0x4203_1000
                tx_ring: XramRingBuffer::new(XRAM_BASE_ADDR, CHANNEL_BUFFER_SIZE),
                rx_ring: XramRingBuffer::new(XRAM_BASE_ADDR + CHANNEL_BUFFER_SIZE, CHANNEL_BUFFER_SIZE),
            }
        }

        #[cfg(feature = "m0")]
        unsafe {
            Self {
                ipc: Ipc::new(ipc),
                // M0 Tx is at 0x4203_1000, Rx is at 0x4203_0000
                tx_ring: XramRingBuffer::new(XRAM_BASE_ADDR + CHANNEL_BUFFER_SIZE, CHANNEL_BUFFER_SIZE),
                rx_ring: XramRingBuffer::new(XRAM_BASE_ADDR, CHANNEL_BUFFER_SIZE),
            }
        }
    }

    /// Sends a message packet to the peer core and triggers an IPC interrupt.
    pub fn send(&mut self, msg_type: u8, payload: &[u8]) -> Result<(), XramError> {
        let header = PacketHeader {
            msg_type,
            flags: 0,
            len: payload.len() as u16,
        };

        self.tx_ring.push(header, payload)?;

        // Signal peer core via hardware IPC channel 0
        #[cfg(feature = "d0")]
        self.ipc.trigger_m0(1 << 0);

        #[cfg(feature = "m0")]
        self.ipc.trigger_d0(1 << 0);

        Ok(())
    }

    /// Checks if a message is pending from the peer core on channel 0.
    pub fn is_message_pending(&self) -> bool {
        #[cfg(feature = "d0")]
        return (self.ipc.pending_d0() & (1 << 0)) != 0;

        #[cfg(feature = "m0")]
        return (self.ipc.pending_m0() & (1 << 0)) != 0;
    }

    /// Receives a message packet if one is pending, clearing the IPC interrupt flag.
    pub fn receive(&mut self, buf: &mut [u8]) -> Result<Option<(PacketHeader, usize)>, XramError> {
        if !self.is_message_pending() {
            return Ok(None);
        }

        let result = self.rx_ring.pop(buf);

        #[cfg(feature = "d0")]
        self.ipc.clear_d0(1 << 0);

        #[cfg(feature = "m0")]
        self.ipc.clear_m0(1 << 0);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header_size() {
        assert_eq!(core::mem::size_of::<PacketHeader>(), 4);
    }

    #[test]
    fn test_ring_buffer_push_pop() {
        let mut mem = [0u8; 1024];
        let mut ring = unsafe { XramRingBuffer::new(mem.as_mut_ptr() as usize, 1024) };

        let payload = b"Hello from D0";
        let header = PacketHeader {
            msg_type: msg_type::PING,
            flags: 0,
            len: payload.len() as u16,
        };

        assert!(ring.push(header, payload).is_ok());

        let mut rx_buf = [0u8; 64];
        let (hdr, len) = ring.pop(&mut rx_buf).unwrap().expect("Expected packet");

        assert_eq!(hdr.msg_type, msg_type::PING);
        assert_eq!(len, payload.len());
        assert_eq!(&rx_buf[..len], payload);
    }
}
