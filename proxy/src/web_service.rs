use std::{
	convert::Infallible,
	net::SocketAddr,
	sync::{Arc},
	str,
	// net::TcpListener
};
use crate::rules::Configuration;
use crate::proxy_service::ProxyService;
use tokio::{net::TcpListener,sync::Mutex};
use futures::future::{join_all, join};
use hyper::{Body, Request, Response, Server, Method, body::to_bytes};
use hyper::service::{make_service_fn, service_fn};

pub struct ProxyServer {
	ports: Vec<u16>,
	rules: Arc<Mutex<Configuration>>,
	configuration_port: u16
}

impl ProxyServer {
	pub fn new(ports: Vec<u16>, config: Configuration, configuration_port: u16) -> Self {
		let config =  Arc::new(Mutex::new(config));
		ProxyServer {
			ports: ports,
			rules: config,
			configuration_port: configuration_port
		}
	}

	pub async fn run(&self) {
	    // We'll bind to 127.0.0.1:3000
		let proxy_handle = join_all(self.ports.iter().cloned().map(|port| {
			let config = self.rules.clone();
			tokio::spawn(async move {
				let addr = SocketAddr::from(([127, 0, 0, 1], port));
				let listener = TcpListener::bind(addr).await.unwrap();
				loop {
					let config_clone = config.clone();
					let (stream, addr) = listener.accept().await.unwrap();
					tokio::spawn(async move {
						println!("TCP accepted: {:?}, {:?}", &stream, &addr);
						let mut service = ProxyService::new(config_clone, stream);
						service.handle_request().await;
					});
				}
			})
		}));

		let config_addr = SocketAddr::from(([0, 0, 0, 0], self.configuration_port));

		let config = Arc::clone(&self.rules);
		let make_svc = make_service_fn(|_conn| {
			let config_clone = config.clone();
			async move {
				Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
					ProxyServer::handle_request(req, config_clone.clone())
				}))
			}
		});
		let server = Server::bind(&config_addr).serve(make_svc);

		let (_, config) = join(proxy_handle, server).await;
		config.unwrap();
	}

	async fn generate_response(data: String) -> Result<Response<Body>, Infallible> {
		Ok(Response::new(data.into()))
	}

	async fn handle_request(req: Request<Body>, config: Arc<Mutex<Configuration>>) -> Result<Response<Body>, Infallible> {
		match (req.method(), req.uri().path()) {
			(&Method::GET, "/routes") => ProxyServer::generate_response(config.lock().await.to_json().unwrap()),
			(&Method::POST, "/routes") => {
				let data: String = str::from_utf8(to_bytes(req.into_body()).await.unwrap().as_ref()).unwrap().into();
				println!("{}", &data);
				let new_config = Configuration::from_json(data).unwrap();
				let mut mutex_val = config.lock().await;
				*mutex_val = new_config;
				ProxyServer::generate_response(mutex_val.to_json().unwrap())
			},
			_ => ProxyServer::generate_response("not available".into())
		}.await
	}

}