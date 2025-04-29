use bytes::{BufMut, BytesMut};
use color_eyre::eyre::{Ok, Result};
use std::fmt::{Display, Write};
use strum_macros::EnumIter;

const INT_REGISTER_OFFSET: u8 = 24;
const DOUBLE_REGISTER_OFFSET: u8 = 24;

#[derive(Debug, Clone, Copy)]
pub enum Recipe {
	Connection {
		ip: u32,
		port: u16,
	},
	JCommand {
		command: i32,
		q: [f64; 6],
		speed: f64,
		acceleration: f64,
		time: f64,
		lookahead_time: f64,
		gain: f64,
	},
}

impl Recipe {
	pub fn serialize(self, bytes: &mut BytesMut) {
		match self {
			Recipe::Connection { ip, port } => {
				bytes.put_u8(RecipeId::Connection as u8);
				bytes.put_u32(ip);
				bytes.put_u32(port as u32);
			}
			Recipe::JCommand {
				command,
				q,
				speed,
				acceleration,
				time,
				lookahead_time,
				gain,
			} => {
				bytes.put_u8(RecipeId::JCommand as u8);
				bytes.put_i32(command);
				for p in q {
					bytes.put_f64(p);
				}
				bytes.put_f64(speed);
				bytes.put_f64(acceleration);
				bytes.put_f64(time);
				bytes.put_f64(lookahead_time);
				bytes.put_f64(gain);
			}
		}
	}
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, EnumIter)]
pub enum RecipeId {
	Connection = 1,
	JCommand = 2,
}

macro_rules! write_regs_fmt {
	($last:expr) => {
		"{}"
	};
	($head:expr, $($tail:expr),+) => {
		concat!("{},", write_regs_fmt!($($tail),+))
	};
}

macro_rules! write_regs {
	($w:expr, $($regs:expr),+ $(,)?) => {
		write!($w, write_regs_fmt!($($regs),+), $($regs),+)
	};
}

impl RecipeId {
	pub fn setup(self, bytes: &mut BytesMut) -> Result<()> {
		match self {
			Self::Connection => {
				write_regs!(bytes, IntReg(0), IntReg(1))
			}
			Self::JCommand => {
				write_regs!(
					bytes,
					IntReg(0),
					Vec6D(0),
					DoubleReg(6),
					DoubleReg(7),
					DoubleReg(8),
					DoubleReg(9),
					DoubleReg(10),
				)
			}
		}?;
		Ok(())
	}
}

struct IntReg(u8);

impl Display for IntReg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "input_int_register_{}", INT_REGISTER_OFFSET + self.0)
	}
}

struct DoubleReg(u8);

impl Display for DoubleReg {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"input_double_register_{}",
			DOUBLE_REGISTER_OFFSET + self.0
		)
	}
}

struct Vec6D(u8);

impl Display for Vec6D {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write_regs!(
			f,
			DoubleReg(self.0 + 0),
			DoubleReg(self.0 + 1),
			DoubleReg(self.0 + 2),
			DoubleReg(self.0 + 3),
			DoubleReg(self.0 + 4),
			DoubleReg(self.0 + 5)
		)
	}
}
