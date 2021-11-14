use std::{
	convert::Infallible,
	net::SocketAddr,
	sync::{Arc, Mutex}
};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

pub struct ProxyServer {
	port: u16,
	rules: Arc<Mutex<String>>
}

impl ProxyServer {
	pub fn new(port: u16, config: String) -> Self {
		ProxyServer {
			port: port,
			rules: Arc::new(Mutex::new(config))
		}
	}

	pub async fn run(&mut self) {

	    // We'll bind to 127.0.0.1:3000
		let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

		// A `Service` is needed for every connection, so this
		// creates one from our `hello_world` function.
		let make_svc = make_service_fn(|_conn| async {
			// service_fn converts our function into a `Service`
			Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
				// async {
					match (req.method(), req.uri().path()) {
						_ => ProxyServer::generate_response("hello world".into())
					}
				// }
			}))
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
}