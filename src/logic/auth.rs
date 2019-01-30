//! Authentication and authorization-related logic.

use crate::{dal::DB, schema::User, view::render};
use failure::Error;
use futures::{
    future::{err, Either},
    Future,
};
use serde_json::json;

/// Returns the user authenticated by the given token, if any.
pub fn authed_user(db: DB, token: &str) -> impl Future<Item = User, Error = Error> {
    match token.parse() {
        Ok(token) => Either::A(db.get_auth_user(token).and_then(move |id| db.get_user(id))),
        Err(e) => Either::B(err(e.into())),
    }
}

/// Creates a new authentication token and mails it to the user.
pub fn login(db: DB, username: String) -> impl Future<Item = (), Error = Error> {
    db.get_user_by_username(username)
        .and_then(move |user| db.create_auth(user.id, None).join(Ok(user)))
        .and_then(|(token, user)| {
            let vars = json!({
                "token": token,
                "user": user,
            });
            let text = render("login-mail.txt", &vars)?;
            let html = render("login-mail.html", vars)?;
            Ok((html, text))
        })
        .and_then(|(_html, _text)| {
            // TODO
            err(failure::format_err!("TODO"))
        })
}
