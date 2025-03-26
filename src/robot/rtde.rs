use core::str;
use std::net::{IpAddr, SocketAddrV4};

use bytes::{BufMut, Bytes, BytesMut};
use strum::IntoEnumIterator;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
};

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
	pub async fn new(addr: IpAddr) -> io::Result<Self> {
		let conn = TcpStream::connect((addr, RTDE_PORT)).await?;

		Ok(Self { conn })
	}

	pub fn get_local_addr(&self) -> io::Result<IpAddr> {
		let local_addr = self.conn.local_addr()?;
		Ok(local_addr.ip())
	}

	pub async fn setup(&mut self) -> io::Result<()> {
		self.request_protocol().await?;
		self.setup_recipes().await?;
		// probably only need it for output stuff? probably don't want it anyways
		// self.start().await?;
		todo!()
	}

	async fn request_protocol(&mut self) -> io::Result<()> {
		let mut bytes = BytesMut::with_capacity(5);
		bytes.put_u16(5);
		bytes.put_u8(RDTECommand::RequestProtocolVersion as u8);
		bytes.put_u16(PROTOCOL_VERSION);
		self.conn.write_all(&bytes).await?;
		let mut res_buff = [0u8; 4];
		self.conn.read_exact(&mut res_buff).await?;
		if res_buff[1] != 1 {
			panic!("UR RTDE protocol didn't accept version {PROTOCOL_VERSION}");
		}
		Ok(())
	}

	async fn setup_recipes(&mut self) -> io::Result<()> {
		for recipe in RecipeId::iter() {
			let mut bytes = BytesMut::new();
			bytes.put_u16(0);
			bytes.put_u8(RDTECommand::ControlPackageSetupInputs as u8);
			recipe.setup(&mut bytes);
			let payload_len = bytes.len() as u16;
			bytes[..2].copy_from_slice(&payload_len.to_be_bytes());
			self.conn.write_all(&bytes).await?;
			let mut res_buf = [0u8; 4096];
			self.conn.read(&mut res_buf[..4]).await?;
			let id = res_buf[3];
			if id != recipe as u8 {
				panic!("Recipe id mismatch! Found {id} for {recipe:?}")
			}
			let res_len = u16::from_be_bytes([res_buf[0], res_buf[1]]) as usize - 4;
			if res_len > 4096 {
				panic!("Didn't expect recipe setup response to be this big!");
			}
			self.conn.read_exact(&mut res_buf[..res_len]).await?;
			let res_str = str::from_utf8(&res_buf[..res_len]).expect("Bad recipe response string!");
			for res in res_str.split(',') {
				match res {
					"IN_USE" => panic!("Came across IN_USE when setting up recipe for {recipe:?}!"),
					"NOT_FOUND" => {
						panic!("Came across NOT_FOUND when setting up recipe for {recipe:?}!")
					}
					_ => {}
				}
			}
		}
		Ok(())
	}

	async fn start(&mut self) -> io::Result<()> {
		let mut bytes = BytesMut::with_capacity(5);
		bytes.put_u16(3);
		bytes.put_u8(RDTECommand::ControlPackageStart as u8);
		self.conn.write_all(&bytes).await?;
		let mut res_buff = [0u8; 4];
		self.conn.read_exact(&mut res_buff).await?;
		if res_buff[1] != 1 {
			panic!("UR RTDE protocol didn't accept starting");
		}
		Ok(())
	}

	pub async fn send(&mut self, recipe: Recipe) -> io::Result<()> {
		let mut bytes = BytesMut::new();
		bytes.put_u16(0);
		bytes.put_u8(RDTECommand::DataPackage as u8);
		recipe.serialize(&mut bytes);
		let payload_len = bytes.len() as u16;
		bytes[..2].copy_from_slice(&payload_len.to_be_bytes());
		self.conn.write_all(&bytes).await
	}
}
