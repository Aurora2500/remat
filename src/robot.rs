use std::net::{IpAddr, Ipv4Addr};

use callback::{CallbackClient, CallbackServer};
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
		callback_addr: Option<Ipv4Addr>,
	) -> io::Result<Self> {
		let addr = lookup_host(addr)
			.await?
			.next()
			.map(|addr| addr.ip())
			.expect("Bad socket addr");
		let mut rtde = RtdeClient::new(addr).await?;
		let mut script = ScriptClient::new(addr).await?;
		let callback_addr = match callback_addr {
			Some(addr) => addr,
			None => {
				let callback_addr = rtde
					.get_local_addr()
					.expect("Failed getting rtde local addr");
				match callback_addr {
					IpAddr::V4(x) => x,
					IpAddr::V6(x) => {
						panic!("Connected via IPV6 addr {x}, but callback requires IPV4 addr!")
					}
				}
			}
		};
		let callback = CallbackServer::new(callback_addr).await?;
		rtde.setup().await?;
		script.send_script().await?;
		let callback = callback.accept().await?;
		Ok(Self {
			rtde,
			script,
			callback,
		})
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
