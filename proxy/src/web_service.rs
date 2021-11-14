use std::{
	convert::Infallible,
	net::SocketAddr,
	sync::{Arc, Mutex},
	str
};
use hyper::{Body, Method, Request, Response, Server, StatusCode, body::to_bytes};
use hyper::service::{make_service_fn, service_fn};
use futures::StreamExt;
use crate::rules::Configuration;

pub struct ProxyServer {
	port: u16,
	rules: Arc<Mutex<Configuration>>
}

impl ProxyServer {
	pub fn new(port: u16, config: Configuration) -> Self {
		ProxyServer {
			port: port,
			rules: Arc::new(Mutex::new(config))
		}
	}

	pub async fn run(&mut self) {

	    // We'll bind to 127.0.0.1:3000
		let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

		// A `Service` is needed for every connection, so this
		// creates one from our `hello_world` fun
		let config = Arc::clone(&self.rules);
		let make_svc = make_service_fn(|_conn| {
			let config_clone = config.clone();
			async move {
				// let config_clone2 = Arc::clone(&config_clone);
				// service_fn converts our function into a `Service`
				Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
					// match (req.method(), req.uri().path()) {
					// 	(&Method::GET, "/routes") => ProxyServer::generate_response(config_clone.lock().unwrap().to_json().unwrap()),
					// 	// (&Method::POST, "/routes") => {
					// 	// 	let data: String = req.body().
					// 	// 	ProxyServer::generate_response(config_clone.lock().unwrap().to_json().unwrap())
					// 	// },
					// 	_ => ProxyServer::generate_response("hello world".into())
					// }
					ProxyServer::handle_request(req, config_clone.clone())
				}))
			}
		});

		let server = Server::bind(&addr).serve(make_svc);

		// Run this server for... forever!
		if let Err(e) = server.await {
			eprintln!("server error: {}", e);
		}
	}

	async fn generate_response(data: String) -> Result<Response<Body>, Infallible> {
		Ok(Response::new(data.into()))
	}

	async fn handle_request(req: Request<Body>, config: Arc<Mutex<Configuration>>) -> Result<Response<Body>, Infallible> {
		match (req.method(), req.uri().path()) {
			(&Method::GET, "/routes") => ProxyServer::generate_response(config.lock().unwrap().to_json().unwrap()),
			(&Method::POST, "/routes") => {
				let data: String = str::from_utf8(to_bytes(req.into_body()).await.unwrap().as_ref()).unwrap().into();
				println!("{}", &data);
				let new_config = Configuration::from_json(data).unwrap();
				if let Ok(mut mutex_val) = config.lock() {
					*mutex_val = new_config;
				}
				ProxyServer::generate_response(config.lock().unwrap().to_json().unwrap())
			},
			_ => ProxyServer::generate_response("hello world".into())
		}.await
	}
}