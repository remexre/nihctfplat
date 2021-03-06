//! The HTTP server.
//!
//! > **Router** is the the only module that knows anything about HTTP. Every other part of the
//! > system has no knowledge of how the request is really being made. The router's responsibility
//! > is to call into the domain logic, and then render that response data with an appropriate view.

#[macro_use]
mod util;

mod auth;
mod errors;
mod team;

use crate::{
    dal::{Mailer, DB},
    router::util::set,
    view::render_html,
};
use futures::{
    future::{loop_fn, ok, Loop},
    Future,
};
use log::{info, warn};
use packer::Packer;
use serde_json::json;
use std::{net::SocketAddr, path::Path};
use warp::{
    http::{header::CONTENT_TYPE, Response},
    path,
    reject::custom,
    Filter, Rejection,
};

/// Starts an HTTP server at the given address. The polymorphism in the return type indicates that
/// the future will never resolve, since it can be trivially used as
/// `impl Future<Item = Void, Error = Void>`.
pub fn serve_on<T, E>(
    addr: SocketAddr,
    db: DB,
    mailer: Mailer,
) -> impl Future<Item = T, Error = E> {
    loop_fn((), move |()| {
        info!("Starting to serve...");
        let server = set(db.clone())
            .and(set(mailer.clone()))
            .and(statics().or(routes()))
            .recover(errors::internal)
            .recover(errors::last_chance)
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

fn routes() -> Resp!() {
    auth::parse_auth_cookie()
        .and(route_any! {
            GET() => simple_page("index.html"),
            GET("humans.txt") => {
                warp::path::end().map(|| env!("CARGO_PKG_AUTHORS").replace(':', "\n"))
            },
            GET("login") => simple_page("login.html"),
            GET("login") => auth::login_from_mail_get(),
            POST("login") => auth::login(),
            POST("login") => auth::login_from_mail_post(),
            POST("logout") => auth::logout(),
            GET("register") => simple_page("register.html"),
            POST("register") => auth::register(),
            GET("sponsoring-ctf3") => simple_page("sponsoring-ctf3.html"),
            GET("team") => simple_page("team.html"),
            GET("team" / "create") => simple_page("create-team.html"),
            POST("team" / "create") => team::create(),
            GET("team" / "join") => simple_page("join-team.html"),
            POST("team" / "join") => team::join(),
        })
        .boxed()
}

fn statics() -> impl Clone + Filter<Extract = (Response<&'static [u8]>,), Error = Rejection> {
    #[derive(Packer)]
    #[folder = "src/static"]
    struct Assets;

    warp::path::tail().and_then(|path: warp::path::Tail| {
        let path = path.as_str();
        Assets::get(path)
            .ok_or_else(warp::reject::not_found)
            .and_then(|body| {
                let ext = coerce!(path.as_ref() => &Path)
                    .extension()
                    .and_then(|s| s.to_str());
                let ct = match ext {
                    Some("css") => "text/css",
                    Some("txt") => "text/plain; charset=utf-8",
                    Some("woff2") => "font/woff2",
                    _ => {
                        warn!("Unknown extension for static file: {:?}", ext);
                        "application/octet-stream"
                    }
                };
                Response::builder()
                    .header(CONTENT_TYPE, ct)
                    .body(body)
                    .map_err(custom)
            })
    })
}

fn simple_page(name: &'static str) -> Resp!() {
    warp::path::end()
        .and(auth::opt_auth())
        .and(auth::opt_team())
        .and(auth::opt_team_members())
        .and_then(move |me, team, team_members| {
            let data = json!({
                "me": me,
                "team": team,
                "team_members": team_members
            });
            render_html(name, data)
        })
        .boxed()
}
