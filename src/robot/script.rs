use std::net::IpAddr;

use tokio::{
	io::{self, AsyncWriteExt},
	net::TcpStream,
};

const EVENT_LOOP_SRC: &'static str = include_str!("event_loop.urscript");
const SCRIPT_PORT: u16 = 30001;

pub struct ScriptClient {
	conn: TcpStream,
}

impl ScriptClient {
	pub async fn new(addr: IpAddr) -> io::Result<Self> {
		let conn = TcpStream::connect((addr, SCRIPT_PORT)).await?;
		Ok(Self { conn })
	}

	pub async fn send_script(&mut self) -> io::Result<()> {
		self.conn.write_all(EVENT_LOOP_SRC.as_bytes()).await
	}
}
