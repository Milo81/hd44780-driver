use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::i2c::Write;

use crate::{bus::DataBus, error::Result};

pub struct I2CBusMcp2300<I2C: Write> {
	i2c_bus: I2C,
	address: u8,
}

const BACKLIGHT: u8 = 0b1000_0000;
const ENABLE: u8 = 0b0000_0100;
const REGISTER_SELECT: u8 = 0b0000_0010;

const MCP23008_IODIR: u8 = 0x00;
const MCP23008_GPIO: u8 = 0x09;

impl<I2C: Write> I2CBusMcp2300<I2C> {
	pub fn new(mut i2c_bus: I2C, address: u8) -> I2CBusMcp2300<I2C> {
		// initialization of MCP2300x expander
		let _ = i2c_bus.write(address, &[MCP23008_IODIR, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
		let _ = i2c_bus.write(address, &[MCP23008_IODIR, 0x00]);

		I2CBusMcp2300 { i2c_bus, address }
	}

	// bit pattern for the burstBits function is
	//
	//  7   6   5   4   3   2   1   0
	// LT  D7  D6  D5  D4  EN  RS  n/c
	//-----
	fn burst_bits<D: DelayUs<u16> + DelayMs<u8>>(&mut self, bits: u8, delay: &mut D) {
		let _ = self.i2c_bus.write(self.address, &[MCP23008_GPIO, bits]);
		delay.delay_us(30);
	}

	/// Write a nibble to the lcd
	/// The nibble should be in the upper part of the byte
	fn write_nibble<D: DelayUs<u16> + DelayMs<u8>>(&mut self, nibble: u8, data: bool, delay: &mut D) {
		let rs = match data {
			false => 0u8,
			true => REGISTER_SELECT,
		};
		let byte = nibble | rs | BACKLIGHT;

		self.burst_bits(byte | ENABLE, delay);
		self.burst_bits(byte, delay);
	}
}

impl<I2C: Write> DataBus for I2CBusMcp2300<I2C> {
	fn write<D: DelayUs<u16> + DelayMs<u8>>(&mut self, byte: u8, data: bool, delay: &mut D) -> Result<()> {
		// isolate high 4 bits, shift over to data pins (bits 6-3: x1111xxx)
		let upper_nibble = (byte & 0xF0) >> 1;
		self.write_nibble(upper_nibble, data, delay);

		// isolate low 4 bits, shift over to data pins (bits 6-3: x1111xxx)
		let lower_nibble = (byte & 0x0F) << 3;
		self.write_nibble(lower_nibble, data, delay);

		Ok(())
	}
}
