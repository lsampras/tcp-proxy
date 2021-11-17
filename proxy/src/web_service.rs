use std::{convert::Infallible, net::SocketAddr, str, sync::{Arc}};
use crate::rules::Configuration;
use crate::proxy_service::ProxyService;
use tokio::{net::TcpListener,sync::{RwLock, Mutex, mpsc::{channel, Sender}}};
use futures::future::{join_all, join};
use hyper::{Body, Request, Response, Server, Method, body::to_bytes};
use hyper::service::{make_service_fn, service_fn};

pub struct ProxyServer {
	ports: Vec<u16>,
	rules: Arc<RwLock<Configuration>>,
	configuration_port: u16,
	listen: Arc<RwLock<bool>>,
}

impl ProxyServer {
	pub fn new(ports: Vec<u16>, config: Configuration, configuration_port: u16) -> Self {
		let config =  Arc::new(RwLock::new(config));
		ProxyServer {
			ports: ports,
			rules: config,
			configuration_port: configuration_port,
			listen: Arc::new(RwLock::new(true))
		}
	}

	pub async fn run(&self) {
	    // We'll bind to 127.0.0.1:3000
		let proxy_handle = join_all(self.ports.iter().cloned().map(|port| {
			let config = self.rules.clone();
			let should_listen = self.listen.clone();
			tokio::spawn(async move {
				let addr = SocketAddr::from(([127, 0, 0, 1], port));
				let mut tcp_connection = None;
				while should_listen.read().await.to_owned()  {
					if let Ok(connection) = TcpListener::bind(addr).await {
						tcp_connection = Some(connection);
						break;
					}
				}
				let mut handles = vec![];
				if let Some(listener) = tcp_connection {
					println!("Listening on: {:?}", &addr);
					while should_listen.read().await.to_owned() {
						let config_clone = config.clone();
						let (stream, addr) = listener.accept().await.unwrap();
						handles.push(tokio::spawn(async move {
							println!("TCP accepted: {:?}, {:?}", &stream, &addr);
							let mut service = ProxyService::new(config_clone, stream);
							service.handle_request().await;
						}));
					}
				}
				join_all(handles.iter_mut()).await;
			})
		}));

		let config_addr = SocketAddr::from(([0, 0 ,0, 0], self.configuration_port));


		let config = Arc::clone(&self.rules);
		let listener_clone = Arc::clone(&self.listen);
		let (trigger, mut receiver) = channel::<bool>(10);
		let trigger = Arc::new(Mutex::new(trigger));
		let make_svc = make_service_fn(|_conn| {
			let config_clone = config.clone();
			let listener_clone2 = listener_clone.clone();
			let sender = trigger.clone();
			async move {
				Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
					ProxyServer::handle_request(req, config_clone.clone(), listener_clone2.clone(), sender.clone())
				}))
			}
		});

		let tcp_connection ;
		loop  {
			if let Ok(connection) = TcpListener::bind(config_addr).await {
				tcp_connection = connection;
				break;
			}
		}
		println!("Config server listening on: {:?}", config_addr);
		let server = Server::from_tcp(tcp_connection.into_std().unwrap()).unwrap().serve(make_svc);
		let graceful = server.with_graceful_shutdown(async {receiver.recv().await;});
		let (_, config) = join(proxy_handle, graceful).await;
		config.unwrap();
	}

	async fn generate_response(data: String) -> Result<Response<Body>, Infallible> {
		Ok(
			Response::builder()
					.status(200)
					.header("Content-Type", "application/json")
					.body(data.into())
					.unwrap()
		)
	}

	async fn handle_request(req: Request<Body>, config: Arc<RwLock<Configuration>>, listener: Arc<RwLock<bool>>, trigger: Arc<Mutex<Sender<bool>>>) -> Result<Response<Body>, Infallible> {
		match (req.method(), req.uri().path()) {
			(&Method::GET, "/routes") => ProxyServer::generate_response(config.read().await.to_json().unwrap()),
			(&Method::POST, "/routes") => {
				// this currently only accepts a list of targets that are assumed to be in priority
				let data: String = str::from_utf8(to_bytes(req.into_body()).await.unwrap().as_ref()).unwrap().into();
				let new_config = Configuration::from_json(data).unwrap();
				{
					// use this block to restrict the limit the write lock reference
					let mut mutex_val = config.write().await;
					*mutex_val = new_config;
				}
				ProxyServer::generate_response(config.read().await.to_json().unwrap())
			},
			(_, "/stop") => {
				println!("server stop triggered");
				{
					// use this block to restrict the write lock reference
					let mut mutex_val = listener.write().await;
					*mutex_val = false;
				}
				trigger.lock().await.send(true).await.unwrap();
				ProxyServer::generate_response("{\"message\": \"Stopping server\"}".into())
			},
			_ => ProxyServer::generate_response("{\"message\": \"Hello world\"}".into())
		}.await
	}

}