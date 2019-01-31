use crate::{
    dal::{Mailer, DB},
    logic,
    router::{simple_page, util::FutureExt},
    schema::User,
    view::render_html,
};
use chrono::Duration;
use failure::Error;
use futures::{future::result, Future};
use serde_derive::Deserialize;
use serde_json::json;
use uuid::Uuid;
use warp::{
    http::{
        header::{LOCATION, SET_COOKIE},
        Response, StatusCode,
    },
    path,
    reject::custom,
    Filter, Rejection,
};

/// A filter that parses a user's authentication cookie.
pub fn parse_auth_cookie() -> impl Clone + Filter<Extract = (), Error = Rejection> {
    warp::cookie("auth")
        .and(warp::ext::get::<DB>())
        .and_then(move |token: String, db: DB| {
            logic::auth::authed_user(db.clone(), &token)
                .map(warp::ext::set)
                .map_err(|err| custom(err.compat()))
        })
        .untuple_one()
        .or(warp::any())
        .unify()
}

/*
/// A filter that authenticates the user via a cookie. The `parse_auth_cookie` filter must have
/// already been run.
pub fn auth() -> impl Clone + Filter<Extract = (User,), Error = Rejection> {
    // TODO: Return a different error.
    warp::ext::get::<User>()
}
*/

/// A filter that optionally authenticates the user via a cookie. The `parse_auth_cookie` filter
/// must have already been run.
pub fn auth_opt() -> impl Clone + Filter<Extract = (Option<User>,), Error = Rejection> {
    // TODO: Return a different error.
    warp::ext::get::<User>()
        .map(Some)
        .or(warp::any().map(|| None))
        .unify()
}

pub fn login() -> Resp!() {
    #[derive(Debug, Deserialize)]
    struct Form {
        username: String,
    }

    warp::body::content_length_limit(2 * 1024)
        .and(warp::ext::get::<DB>())
        .and(warp::ext::get::<Mailer>())
        .and(warp::body::form())
        .and_then(|db, mailer, form: Form| {
            logic::auth::login_1(db, mailer, form.username).err_to_rejection()
        })
        .untuple_one()
        .and(simple_page("login-ok.html"))
        .boxed()
}

pub fn login_from_mail_get() -> Resp!() {
    path!(Uuid)
        .and(warp::path::end())
        .and(auth_opt())
        .and_then(move |login, me| {
            render_html("login-from-mail.html", json!({ "login": login, "me": me }))
        })
        .boxed()
}

pub fn login_from_mail_post() -> Resp!() {
    path!(Uuid)
        .and(warp::path::end())
        .and(warp::ext::get::<DB>())
        .and_then(|login, db| logic::auth::login_2(db, login).err_to_rejection())
        .and_then(|auth| {
            let set_cookie = format!(
                "auth={}; Max-Age={}; Path=/",
                auth,
                Duration::weeks(520).num_seconds()
            );
            let r = Response::builder()
                .header(LOCATION, "/")
                .header(SET_COOKIE, set_cookie)
                .status(StatusCode::FOUND)
                .body("")
                .map_err(Error::from);
            result(r).err_to_rejection()
        })
        .boxed()
}

pub fn logout() -> Resp!() {
    warp::path::end()
        .and_then(|| {
            let r = Response::builder()
                .header(LOCATION, "/")
                .header(SET_COOKIE, "auth=; Max-Age=0; Path=/")
                .status(StatusCode::FOUND)
                .body("")
                .map_err(Error::from);
            result(r).err_to_rejection()
        })
        .boxed()
}

pub fn register() -> Resp!() {
    #[derive(Debug, Deserialize)]
    struct Form {
        email: String,
        username: String,
    }

    warp::body::content_length_limit(2 * 1024)
        .and(warp::ext::get::<DB>())
        .and(warp::ext::get::<Mailer>())
        .and(warp::body::form())
        .and_then(|db, mailer, form: Form| {
            logic::auth::register(db, mailer, form.username, form.email).err_to_rejection()
        })
        .untuple_one()
        .and(simple_page("login-ok.html"))
        .boxed()
}
