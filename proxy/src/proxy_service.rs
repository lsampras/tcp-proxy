use std::{net::{SocketAddr}, sync::{Arc}};
use crate::rules::{Configuration};
use tokio::{io::AsyncWriteExt, join, net::TcpStream, sync::{Mutex,RwLock}};
use std::io::ErrorKind::WouldBlock;



pub struct ProxyService {
	config: Arc<RwLock<Configuration>>,
	incoming: Arc<Mutex<TcpStream>>,
	incoming_addr: SocketAddr
}


impl ProxyService {

	pub fn new(config: Arc<RwLock<Configuration>>, tcp:TcpStream) -> Self {
		ProxyService {
			config: config,
			incoming_addr: tcp.peer_addr().unwrap(),
			incoming: Arc::new(Mutex::new(tcp))
		}
	}

	async fn get_outgoing(&mut self) -> Option<TcpStream> {
		let rules = self.config.read().await.rules.clone();
		for rule in rules {
			if rule.source == self.incoming_addr.ip() {
				for target in rule.targets {
					// using input order as priority order here 
					// i.e the closest/fastest target is the first in the list
					// use tokio select incase no priority is given
					// Could also add pre-processing when the rules are updated to determine targets
					if let Ok(stream) = TcpStream::connect(target).await {
						return Some(stream)
					}
				}
			}
		}
		None
	}

	async fn pipe_streams(&self, from:Arc<Mutex<TcpStream>>, to:Arc<Mutex<TcpStream>>) {
		let mut read_pending = true;
		let mut write_pending = false;
		while read_pending || write_pending {

			let mut read_msg = vec![0; 1024];
			// read data first
			while read_pending {
				let locked_stream = from.lock().await;
				match locked_stream.try_read(&mut read_msg) {
					Ok(0) => {
						read_pending = false;
						break;
					},
					Ok(n) => {
						read_msg.truncate(n);
						read_pending = false;
						write_pending = true;
						break;
					}
					Err(ref e) if e.kind() == WouldBlock => {
						break;
					}
					Err(e) => {
						println!("Other read errors: {:?}", e);
						break;
					}
				}
			}
			// write data that's been read
			while write_pending {
				let locked_stream = to.lock().await;
				match locked_stream.try_write(&mut read_msg) {
					Ok(n) => {
						println!("Data written {:?} bytes to {:?}", n, locked_stream.peer_addr().unwrap());
						read_pending = true;
						write_pending = false;
						break;
					}
					Err(ref e) if e.kind() == WouldBlock => {
						break;
					}
					Err(e) => {
						println!("Other write errors: {:?}", e);
						break;
					}
				}
			}
		}
		println!("closing stream {:?}", to.lock().await.peer_addr().unwrap());
		// this only closes the write portions of a stream
		to.lock().await.shutdown().await.unwrap();
	}

	pub async fn handle_request(&mut self) {
		if let Some(outgoing) = self.get_outgoing().await {
			let arc_outgoing = Arc::new(Mutex::new(outgoing));
			join!(
				self.pipe_streams(self.incoming.clone(), arc_outgoing.clone()),
				self.pipe_streams(arc_outgoing.clone(), self.incoming.clone())
			);
		} else {
			self.incoming.lock().await.shutdown().await.unwrap();
		}
	}
}