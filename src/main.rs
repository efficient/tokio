#![deny(warnings)]

mod black_box;

use black_box::black_box;
use hyper::rt::Future;
use hyper::rt::run;
use hyper::service::service_fn_ok;
use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper::StatusCode;

fn main() {
	let addr = "127.0.0.1:1337".parse().unwrap();
	let server = Server::bind(&addr)
		.serve(|| service_fn_ok(callback))
		.map_err(|e| eprintln!("server error: {}", e))
	;

	println!("Listening on http://{}", addr);
	run(server);
}

fn callback(req: Request<Body>) -> Response<Body> {
	let path = &req.uri().path()[1..];
	if let Ok(iters) = path.parse() {
		for _ in 0..iters {
			black_box();
		}
		Response::default()
	} else {
		Response::builder()
			.status(StatusCode::NOT_FOUND)
			.body("404 not found".into())
			.unwrap()
	}
}
