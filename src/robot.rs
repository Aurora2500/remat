use std::net::SocketAddrV4;

use callback::{CallbackClient, CallbackServer};
use nix::ifaddrs::getifaddrs;
use rtde::RtdeClient;
use tokio::{
	io::{self},
	net::{lookup_host, ToSocketAddrs},
};

use script::ScriptClient;

mod callback;
mod commands;
mod recipes;
mod rtde;
mod script;

pub struct Robot {
	rtde: RtdeClient,
	script: ScriptClient,
	callback: CallbackClient,
}

impl Robot {
	pub async fn start_with_addr<A: ToSocketAddrs>(
		addr: A,
		callback_addr: SocketAddrV4,
	) -> io::Result<Self> {
		let addr = lookup_host(addr)
			.await?
			.next()
			.map(|addr| addr.ip())
			.expect("Bad socket addr");
		let callback = CallbackServer::new(callback_addr).await?;
		let mut rtde = RtdeClient::new(addr).await?;
		let mut script = ScriptClient::new(addr).await?;
		rtde.setup(callback_addr).await?;
		script.send_script().await?;
		let callback = callback.accept().await?;
		Ok(Self {
			rtde,
			script,
			callback,
		})
	}

	pub async fn start<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
		let callback_addr = getifaddrs()
			.expect("Getting own addrs failed")
			.flat_map(|addr| addr.address.and_then(|addr| addr.as_sockaddr_in().cloned()))
			.flat_map(|addr| {
				if !addr.ip().is_loopback() {
					Some(SocketAddrV4::new(addr.ip(), addr.port()))
				} else {
					None
				}
			})
			.next()
			.expect("Didn't find any non loopback ipv4 addresses");
		Self::start_with_addr(addr, callback_addr).await
	}

	pub async fn servo_j(
		&mut self,
		q: [f64; 6],
		speed: f64,
		acceleration: f64,
		time: f64,
		lookahead_time: f64,
		gain: f64,
	) -> io::Result<()> {
		self.rtde
			.send(recipes::Recipe::ServoJ {
				q,
				speed,
				acceleration,
				time,
				lookahead_time,
				gain,
			})
			.await?;
		self.callback.non_awaitable().await;
		Ok(())
	}
}
