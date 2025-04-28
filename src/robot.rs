#![allow(dead_code)]
use std::net::{IpAddr, Ipv4Addr};

use callback::{CallbackClient, CallbackServer};
use color_eyre::eyre::{bail, Context, OptionExt, Result};
use rtde::RtdeClient;
use tokio::net::{lookup_host, ToSocketAddrs};

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
	) -> Result<Self> {
		let addr = lookup_host(addr)
			.await?
			.next()
			.map(|addr| addr.ip())
			.ok_or_eyre("Bad IP Address")?;
		let mut rtde = RtdeClient::new(addr).await?;
		println!("RTDE created");
		let mut script = ScriptClient::new(addr).await?;
		println!("Script created");
		let callback_addr = match callback_addr {
			Some(addr) => addr,
			None => {
				let callback_addr = rtde
					.get_local_addr()
					.context("Failed to get local IP address")?;
				match callback_addr {
					IpAddr::V4(x) => x,
					IpAddr::V6(x) => {
						bail!("Connected via IPV6 addr {x}, but callback requires IPV4 addr!")
					}
				}
			}
		};
		let callback = CallbackServer::new(callback_addr).await?;
		println!("Callback server created");
		rtde.setup(callback_addr).await?;
		println!("RTDE Set up");
		script.send_script().await?;
		println!("Script set up");
		let callback = callback.accept().await?;
		println!("Callback accepted");

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
	) -> Result<()> {
		self.rtde
			.send(recipes::Recipe::JCommand {
				command: 1,
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

	pub async fn move_j(
		&mut self,
		q: [f64; 6],
		speed: f64,
		acceleration: f64,
		time: f64,
		lookahead_time: f64,
		gain: f64,
	) -> Result<()> {
		self.rtde
			.send(recipes::Recipe::JCommand {
				command: 2,
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
