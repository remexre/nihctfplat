//! Authentication and authorization-related logic.
//!
//! This whole thing is fairly inefficient, wrt refetching things from the DB we "already know."
//! Stuff here only runs on login/register though, so it shouldn't be a hot path.

use crate::{
    dal::{Mailer, DB},
    schema::User,
    view::render,
};
use chrono::{Duration, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use failure::Error;
use futures::{
    future::{err, Either},
    Future,
};
use serde_json::json;
use uuid::Uuid;

/// Returns the user authenticated by the given token, if any.
pub fn authed_user(db: DB, token: &str) -> impl Future<Item = User, Error = Error> {
    match token.parse() {
        Ok(token) => Either::A(db.get_auth_user(token).and_then(move |id| db.get_user(id))),
        Err(e) => Either::B(err(e.into())),
    }
}

/// Creates a new login token and mails it to the user.
pub fn login_1(db: DB, mailer: Mailer, username: String) -> impl Future<Item = (), Error = Error> {
    db.get_user_by_username(username)
        .and_then(|user| send_login_mail(db, mailer, false, user.id))
}

/// Converts a login token to an authentication token.
pub fn login_2(db: DB, login: Uuid) -> impl Future<Item = Uuid, Error = Error> {
    db.consume_login_link(login)
}

/// Creates a new user and mails them a login link.
pub fn register(
    db: DB,
    mailer: Mailer,
    username: String,
    email: String,
) -> impl Future<Item = (), Error = Error> {
    db.create_user(username, email)
        .and_then(move |id| send_login_mail(db, mailer, true, id))
}

fn send_login_mail(
    db: DB,
    mailer: Mailer,
    register: bool,
    id: i32,
) -> impl Future<Item = (), Error = Error> {
    // TODO: Make this time configurable.
    let expire_duration = Duration::hours(1);

    let expires = Utc::now() + expire_duration;
    db.get_user(id)
        .and_then(move |user| db.create_login_link(id, expires).join(Ok(user)))
        .and_then(move |(token, user)| {
            let vars = json!({
                "duration": HumanTime::from(expire_duration).to_text_en(Accuracy::Rough, Tense::Future),
                "expires": expires.to_rfc2822(),
                "register": register,
                "token": token
            });
            let text = render("login-mail.txt", vars)?;
            Ok((user, text))
        })
        .and_then(move |(user, text)| {
            mailer.send(&user.email, "Log in to ACM CTF 2", &text)
        })
}
