use crate::{dal::DB, logic::auth::authed_user, schema::User};
use futures::Future;
use warp::{reject::custom, Filter, Rejection};

/// A filter that parses a user's authentication cookie.
pub fn parse_auth_cookie() -> impl Clone + Filter<Extract = (), Error = Rejection> {
    warp::cookie("auth")
        .and(warp::ext::get::<DB>())
        .and_then(move |token: String, db: DB| {
            authed_user(db.clone(), &token)
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
