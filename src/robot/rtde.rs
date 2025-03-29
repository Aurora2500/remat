use core::str;
use std::net::{IpAddr, Ipv4Addr};

use bytes::{BufMut, BytesMut};
use color_eyre::eyre::{bail, Ok, Result};
use strum::IntoEnumIterator;
use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};

use crate::robot::callback::CALLBACK_PORT;

use super::{
	commands::RDTECommand,
	recipes::{Recipe, RecipeId},
};

const PROTOCOL_VERSION: u16 = 2;
const RTDE_PORT: u16 = 30004;

pub struct RtdeClient {
	conn: TcpStream,
}

impl RtdeClient {
	pub async fn new(addr: IpAddr) -> Result<Self> {
		let conn = TcpStream::connect((addr, RTDE_PORT)).await?;

		Ok(Self { conn })
	}

	pub fn get_local_addr(&self) -> Result<IpAddr> {
		let local_addr = self.conn.local_addr()?;
		Ok(local_addr.ip())
	}

	pub async fn setup(&mut self, callback_addr: Ipv4Addr) -> Result<()> {
		self.request_protocol().await?;
		self.setup_recipes().await?;
		// probably only need it for output stuff? probably don't want it anyways
		// self.start().await?;
		self.send(Recipe::Connection {
			ip: callback_addr.to_bits(),
			port: CALLBACK_PORT,
		})
		.await?;
		Ok(())
	}

	async fn request_protocol(&mut self) -> Result<()> {
		let mut bytes = BytesMut::with_capacity(5);
		bytes.put_u16(5);
		bytes.put_u8(RDTECommand::RequestProtocolVersion as u8);
		bytes.put_u16(PROTOCOL_VERSION);
		self.conn.write_all(&bytes).await?;
		let mut res_buff = [0u8; 4];
		self.conn.read_exact(&mut res_buff).await?;
		if res_buff[3] != 1 {
			bail!("UR RTDE protocol didn't accept version {PROTOCOL_VERSION}");
		}
		Ok(())
	}

	async fn setup_recipes(&mut self) -> Result<()> {
		for recipe in RecipeId::iter() {
			let mut bytes = BytesMut::new();
			bytes.put_u16(0);
			bytes.put_u8(RDTECommand::ControlPackageSetupInputs as u8);
			recipe.setup(&mut bytes)?;
			let payload_len = bytes.len() as u16;
			bytes[..2].copy_from_slice(&payload_len.to_be_bytes());
			self.conn.write_all(&bytes).await?;
			let mut res_buf = [0u8; 4];
			self.conn.read(&mut res_buf).await?;
			let id = res_buf[3];
			if id != recipe as u8 {
				bail!(
					"Recipe id mismatch! Found {id} for {recipe:?} (expected {})",
					recipe as u8
				)
			}
			let res_len = u16::from_be_bytes([res_buf[0], res_buf[1]]) as usize - 4;
			let mut res_buf = vec![0; res_len];
			self.conn.read_exact(&mut res_buf).await?;
			let res_str = str::from_utf8(&res_buf).expect("Bad recipe response string!");
			for res in res_str.split(',') {
				match res {
					"IN_USE" => bail!("Came across IN_USE when setting up recipe for {recipe:?}!"),
					"NOT_FOUND" => {
						bail!("Came across NOT_FOUND when setting up recipe for {recipe:?}!")
					}
					_ => {}
				}
			}
		}
		Ok(())
	}

	async fn start(&mut self) -> Result<()> {
		let mut bytes = BytesMut::with_capacity(5);
		bytes.put_u16(3);
		bytes.put_u8(RDTECommand::ControlPackageStart as u8);
		self.conn.write_all(&bytes).await?;
		let mut res_buff = [0u8; 4];
		self.conn.read_exact(&mut res_buff).await?;
		if res_buff[1] != 1 {
			bail!("UR RTDE protocol didn't accept starting");
		}
		Ok(())
	}

	pub async fn send(&mut self, recipe: Recipe) -> Result<()> {
		let mut bytes = BytesMut::new();
		bytes.put_u16(0);
		bytes.put_u8(RDTECommand::DataPackage as u8);
		recipe.serialize(&mut bytes);
		let payload_len = bytes.len() as u16;
		bytes[..2].copy_from_slice(&payload_len.to_be_bytes());
		self.conn.write_all(&bytes).await?;
		Ok(())
	}
}
