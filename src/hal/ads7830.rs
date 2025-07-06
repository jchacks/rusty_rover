use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x48;
/// Command-byte template: SD = 1 (single-ended), PD = 01 (powered, no refs off)
const CMD_TEMPLATE: u8 = 0b1000_0100;
const VREF: f32 = 5.0; // external 5 V reference
const DIVIDER: f32 = 3.0; // v1.0 to v1.9 = 3; v2+ = 2

const MEDIAN_SAMPLES: usize = 9;
const MEDIAN_INDEX: usize = MEDIAN_SAMPLES / 2;

pub struct Ads7830<I2C> {
    i2c: I2C,
}

impl<I2C> Ads7830<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C) -> Self {
        Ads7830 { i2c }
    }

    pub fn read_adc(&mut self, channel: u8) -> Result<u8, I2C::Error> {
        let channel_bits = ((channel << 2) | (channel >> 1)) & 0x07;
        let command = CMD_TEMPLATE | (channel_bits << 4);

        let mut read_buffer = [0u8];

        self.i2c
            .write_read(ADDR, &[command], &mut read_buffer)
            .and(Ok(read_buffer[0]))
    }

    pub fn battery_voltage(&mut self) -> Result<f32, I2C::Error> {
        let mut samples = [0u8; MEDIAN_SAMPLES];
        for s in samples.iter_mut() {
            *s = self.read_adc(0)?;
        }
        samples.sort_unstable();
        let code = samples[MEDIAN_INDEX] as f32;
        Ok(code / 255.0 * VREF * DIVIDER)
    }
}

/// Convert natural 0-7 to the ADS7830’s (SD=1) table:
/// 0→0b000 , 1→0b100 , 2→0b001 , … , 7→0b111
fn map_channel(ch: u8) -> Result<u8, ()> {
    if ch > 7 {
        return Err(());
    }
    // Rotate the three LSBs left by two positions
    //  (abc) → (cab)
    Ok(((ch << 2) | (ch >> 1)) & 0x07)
}
