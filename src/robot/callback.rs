use std::net::SocketAddrV4;

use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
	sync::{
		mpsc::{channel, Receiver, Sender},
		oneshot::{channel as oneshot_channel, Sender as OneSender},
	},
	task::{self, JoinHandle},
};

pub struct CallbackServer {
	listener: TcpListener,
}

impl CallbackServer {
	pub async fn new(addr: SocketAddrV4) -> io::Result<Self> {
		let listener = TcpListener::bind(addr).await?;
		Ok(Self { listener })
	}

	pub async fn accept(self) -> io::Result<CallbackClient> {
		let (conn, _) = self.listener.accept().await?;
		Ok(CallbackClient::new(conn))
	}
}

pub struct CallbackClient {
	event_loop_handle: JoinHandle<()>,
	tx: Sender<Option<OneSender<()>>>,
}

async fn callback_event_loop(mut conn: TcpStream, mut rx: Receiver<Option<OneSender<()>>>) {
	while let Some(req) = rx.recv().await {
		conn.write(&[0])
			.await
			.expect("Callback conn unexpectedly closed");
		if let Some(req) = req {
			conn.read_u8().await.expect("Callback unexpectedly closed");
			if let Err(_) = req.send(()) {}
		}
	}
}

impl CallbackClient {
	fn new(conn: TcpStream) -> Self {
		let (tx, rx) = channel(1);
		let event_loop_handle = task::spawn(callback_event_loop(conn, rx));
		Self {
			event_loop_handle,
			tx,
		}
	}

	pub async fn awaitable(&self) {
		let (tx, rx) = oneshot_channel();
		self.tx.send(Some(tx)).await;
		if let Err(_) = rx.await {}
	}

	pub async fn non_awaitable(&self) {
		self.tx.send(None).await;
	}
}
