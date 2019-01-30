//! The HTTP server.
//!
//! > **Router** is the the only module that knows anything about HTTP. Every other part of the
//! > system has no knowledge of how the request is really being made. The router's responsibility
//! > is to call into the domain logic, and then render that response data with an appropriate view.

use crate::{dal::DB, view::render_html};
use futures::{
    future::{loop_fn, ok, Loop},
    prelude::*,
};
use log::{info, warn};
use maplit::hashmap;
use std::net::SocketAddr;
use warp::{filters::BoxedFilter, http::Response, Filter, Rejection};

/// Starts an HTTP server at the given address. The polymorphism in the return type indicates that
/// the future will never resolve, since it can be trivially used as
/// `impl Future<Item = Void, Error = Void>`.
pub fn serve_on<T, E>(addr: SocketAddr, db: DB) -> impl Future<Item = T, Error = E> {
    loop_fn((), move |()| {
        info!("Starting to serve...");
        let server = statics()
            .or(routes(db.clone()))
            .with(warp::log("nihctfplat::router"));
        warp::serve(server).bind(addr).then(|r| {
            let status = match r {
                Ok(()) => "success",
                Err(()) => "failure",
            };
            warn!("HTTP server exited with {}; restarting...", status);
            ok(Loop::Continue(()))
        })
    })
}

fn routes(db: DB) -> Resp!() {
    route_any!(db => {
        () => simple_page("index.html"),
        ("sponsoring-ctf3") => simple_page("sponsoring-ctf3.html"),
        ("humans.txt") => |_| warp::path::end().map(|| env!("CARGO_PKG_AUTHORS").replace(':', "\n"))
    })
}

fn statics() -> impl Clone + Filter<Extract = (&'static [u8],), Error = Rejection> {
    #[derive(packer::Packer)]
    #[folder = "src/static"]
    struct Assets;

    warp::path::tail().and_then(|path: warp::path::Tail| {
        Assets::get(path.as_str()).ok_or_else(warp::reject::not_found)
    })
}

fn simple_page(name: &'static str) -> impl Fn(DB) -> BoxedFilter<(Response<String>,)> {
    move |_db| {
        warp::path::end()
            .and_then(move || render_html(name, &hashmap! { "foo" => "bar" }))
            .boxed()
    }
}