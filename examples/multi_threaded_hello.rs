#![deny(warnings)]
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate pretty_env_logger;

use std::net::{SocketAddr, TcpListener};
use futures::future::FutureResult;
use futures::Stream;

use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Service, Request, Response};

static PHRASE: &'static [u8] = b"Hello World!";

#[derive(Clone, Copy)]
struct Hello;

impl Service for Hello {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;
    fn call(&self, _req: Request) -> Self::Future {
        futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_header(ContentType::plaintext())
                .with_body(PHRASE)
        )
    }

}

fn main() {
    pretty_env_logger::init().unwrap();

    // First, we check the command line args for a num-of-threads
    let num_threads = match ::std::env::args().nth(1) {
        Some(s) => {
            match s.parse::<usize>() {
                Ok(num) => num,
                Err(_) => {
                    println!("Error parsing {:?} as integer", s);
                    return;
                }
            }
        },
        None => {
            println!("Usage: multi_threaded_hello <num-of-threads>");
            return;
        }
    };


    let addr = "127.0.0.1:3000".parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(addr).unwrap();
    let addr = listener.local_addr().unwrap();

    // Spawn a server in (num_threads - 1) threads, because we will also use
    // the main thread aftewards.
    for _ in 0 .. num_threads - 1 {
        let lst = listener.try_clone().unwrap();
        ::std::thread::spawn(move || {
            run_server(lst, addr);
        });
    }
    println!("Listening on http://{} with {} threads.", addr, num_threads);
    run_server(listener, addr);
}

fn run_server(listener: TcpListener, addr: SocketAddr) {
    println!("running server");
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let listener = tokio_core::net::TcpListener::from_listener(listener, &addr, &handle).unwrap();

    let http = Http::new();
    core.run(listener.incoming().for_each(move |(sock, addr)| {
        http.bind_connection(&handle, sock, addr, Hello);
        Ok(())
    })).unwrap();
}
