//! Audio Subsystem for Sipeed M1s Dock
//!
//! Provides support for the on-board PDM MEMS microphone and audio output.

use bl808_hal::pdm::{PdmChannel, PdmDriver};

/// Audio sample rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleRate {
    /// 16 kHz (standard for speech recognition and voice commands).
    Rate16kHz,
    /// 32 kHz.
    Rate32kHz,
    /// 44.1 kHz (CD quality).
    Rate44_1kHz,
    /// 48 kHz (standard audio).
    Rate48kHz,
}

/// Microphone capture channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicChannel {
    Left,
    Right,
    Stereo,
}

impl From<MicChannel> for PdmChannel {
    fn from(ch: MicChannel) -> Self {
        match ch {
            MicChannel::Left => PdmChannel::Left,
            MicChannel::Right => PdmChannel::Right,
            MicChannel::Stereo => PdmChannel::Both,
        }
    }
}

/// On-board PDM MEMS Microphone driver.
pub struct Microphone {
    pdm: PdmDriver,
    sample_rate: SampleRate,
    channel: MicChannel,
}

impl Microphone {
    /// Creates a new Microphone driver.
    pub fn new() -> Self {
        Self {
            pdm: PdmDriver::new(),
            sample_rate: SampleRate::Rate16kHz,
            channel: MicChannel::Left,
        }
    }

    /// Initializes and enables the microphone at the requested sample rate and channel.
    pub fn init(&mut self, rate: SampleRate, channel: MicChannel) {
        self.sample_rate = rate;
        self.channel = channel;
        self.pdm.enable(channel.into(), 0);
    }

    /// Enables sample capture.
    pub fn start(&mut self) {
        self.pdm.enable(self.channel.into(), 0);
    }

    /// Disables sample capture.
    pub fn stop(&mut self) {
        self.pdm.disable();
    }

    /// Reads a single 16-bit signed PCM sample from the hardware FIFO if available.
    pub fn read_sample(&mut self) -> Option<i16> {
        self.pdm.read_sample()
    }

    /// Fills a buffer with PCM samples from the FIFO.
    ///
    /// Returns the number of samples actually read into `buf`.
    pub fn read_buffer(&mut self, buf: &mut [i16]) -> usize {
        let mut count = 0;
        for sample in buf.iter_mut() {
            if let Some(s) = self.pdm.read_sample() {
                *sample = s;
                count += 1;
            } else {
                break;
            }
        }
        count
    }
}

impl Default for Microphone {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microphone_channel_mapping() {
        assert_eq!(PdmChannel::from(MicChannel::Left), PdmChannel::Left);
        assert_eq!(PdmChannel::from(MicChannel::Right), PdmChannel::Right);
        assert_eq!(PdmChannel::from(MicChannel::Stereo), PdmChannel::Both);
    }

    #[test]
    fn test_sample_rates() {
        assert_eq!(SampleRate::Rate16kHz as u32, 0);
    }
}
