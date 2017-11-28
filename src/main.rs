//! This is an example Rust app for OpenShift!
//!
//! It's basically hyper's [`multi_server` example], tweaked to read ports from
//! the environment like [hello-openshift].
//!
//! [`multi_server` example]: https://github.com/hyperium/hyper/blob/master/examples/multi_server.rs
//! [hello-openshift]: https://github.com/openshift/origin/tree/master/examples/hello-openshift

#![deny(warnings)]

extern crate hyper;
extern crate futures;
extern crate tokio_core;

use futures::{Future, Stream};
use futures::future::FutureResult;

use hyper::{Get, StatusCode};
use hyper::header::{ContentLength, ContentType};
use hyper::server::{Http, Service, Request, Response};
use tokio_core::reactor::Core;

use std::env;

static INDEX1: &'static str = "🦀 Hello OpenShift! 🦀\n";
static INDEX2: &'static str = "🦀 This service is powered by Rust! 🦀\n";

struct Srv(&'static str);

impl Service for Srv {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        futures::future::ok(match (req.method(), req.path()) {
            (&Get, "/") => {
                Response::new()
                    .with_header(ContentLength(self.0.len() as u64))
                    .with_header(ContentType::plaintext())
                    .with_body(self.0)
            },
            _ => {
                Response::new()
                    .with_status(StatusCode::NotFound)
            }
        })
    }

}


fn main() {
    let port1 = env::var("PORT").unwrap_or("8080".into()).parse().unwrap();
    let port2 = env::var("SECOND_PORT").unwrap_or("8888".into()).parse().unwrap();

    let addr1 = ([0; 4], port1).into();
    let addr2 = ([0; 4], port2).into();

    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let srv1 = Http::new().serve_addr_handle(&addr1, &handle, || Ok(Srv(INDEX1))).unwrap();
    let srv2 = Http::new().serve_addr_handle(&addr2, &handle, || Ok(Srv(INDEX2))).unwrap();

    println!("Listening on http://{}", srv1.incoming_ref().local_addr());
    println!("Listening on http://{}", srv2.incoming_ref().local_addr());

    let handle1 = handle.clone();
    handle.spawn(srv1.for_each(move |conn| {
        handle1.spawn(conn.map(|_| ()).map_err(|err| println!("srv1 error: {:?}", err)));
        Ok(())
    }).map_err(|_| ()));

    let handle2 = handle.clone();
    handle.spawn(srv2.for_each(move |conn| {
        handle2.spawn(conn.map(|_| ()).map_err(|err| println!("srv2 error: {:?}", err)));
        Ok(())
    }).map_err(|_| ()));

    core.run(futures::future::empty::<(), ()>()).unwrap();
}
