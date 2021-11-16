use std::{
	convert::Infallible,
	net::SocketAddr,
	sync::{Arc},
	str,
	// net::TcpListener
};
use crate::rules::Configuration;
use crate::proxy_service::ProxyService;
use tokio::{io::AsyncWriteExt, net::TcpListener,sync::Mutex};
use tokio::time::{Duration, sleep};

pub struct ProxyServer {
	port: u16,
	rules: Arc<Mutex<Configuration>>,
	configuration_port: u16
}

impl ProxyServer {
	pub fn new(port: u16, config: Configuration) -> Self {
		let config =  Arc::new(Mutex::new(config));
		ProxyServer {
			port: port,
			rules: config,
			configuration_port: 8001
		}
	}

	pub async fn run(&mut self) {

	    // We'll bind to 127.0.0.1:3000
		let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
		let listener = TcpListener::bind(addr).await.unwrap();
		loop {
			let (stream, addr) = listener.accept().await.unwrap();
			let config_clone = self.rules.clone();
			tokio::spawn(async move {
				println!("TCP accepted: {:?}, {:?}", &stream, &addr);
				let mut service = ProxyService::new(config_clone, stream);
				service.handle_request().await;
			});
		}
	}
}