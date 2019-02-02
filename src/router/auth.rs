use crate::{
    dal::{Mailer, DB},
    logic,
    router::{
        simple_page,
        team::TeamMembers,
        util::{FilterExt, FutureExt},
    },
    schema::{Team, User},
    view::render_html,
};
use chrono::Duration;
use failure::{Compat, Error};
use futures::{
    future::{ok, result, Either},
    Future,
};
use serde_derive::Deserialize;
use serde_json::json;
use uuid::Uuid;
use warp::{
    http::{
        header::{LOCATION, SET_COOKIE},
        Response, StatusCode,
    },
    path, Filter, Rejection,
};

/// A filter that parses a user's authentication cookie.
pub fn parse_auth_cookie() -> impl Clone + Filter<Extract = (), Error = Rejection> {
    warp::cookie("auth")
        .and(warp::ext::get::<DB>())
        .and_then(move |token: String, db: DB| {
            logic::auth::authed_user(db.clone(), &token)
                .and_then(move |user| {
                    let team = user.team;
                    warp::ext::set(user);
                    if let Some(team) = team {
                        Either::A(
                            db.get_team(team)
                                .map(warp::ext::set)
                                .join(
                                    db.get_team_members(team)
                                        .map(TeamMembers)
                                        .map(warp::ext::set),
                                )
                                .map(|((), ())| ()),
                        )
                    } else {
                        Either::B(ok(()))
                    }
                })
                .err_to_rejection()
        })
        .untuple_one()
        .or(warp::any())
        .unify()
}

/// A filter that optionally authenticates the user via a cookie. The `parse_auth_cookie` filter
/// must have already been run.
pub fn opt_auth() -> impl Clone + Filter<Extract = (Option<User>,), Error = Rejection> {
    warp::ext::get::<User>()
        .map(Some)
        .or(warp::any().map(|| None))
        .unify()
}

/// A filter that retrieves the user's team from their authentication cookie. The
/// `parse_auth_cookie` filter must have already been run.
pub fn opt_team() -> impl Clone + Filter<Extract = (Option<Team>,), Error = Rejection> {
    warp::ext::get::<Team>()
        .map(Some)
        .or(warp::any().map(|| None))
        .unify()
}

/// A filter that retrieves the user's team's members from their authentication cookie. The
/// `parse_auth_cookie` filter must have already been run.
pub fn opt_team_members() -> impl Clone + Filter<Extract = (Option<Vec<String>>,), Error = Rejection>
{
    warp::ext::get::<TeamMembers>()
        .map(|TeamMembers(tm)| Some(tm))
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
        .recover_with_template("login.html", |err: &Compat<Error>| {
            let err = err.to_string();
            match coerce!(&err => &str) {
                r#"NotFound"# => Some((
                    StatusCode::NOT_FOUND,
                    vec!["bad_username"],
                    vec!["That user doesn't exist..."],
                )),
                _ => None,
            }
        })
        .boxed()
}

pub fn login_from_mail_get() -> Resp!() {
    path!(Uuid)
        .and(warp::path::end())
        .and(opt_auth())
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
                .body(String::new())
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
                .body(String::new())
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
        .recover_with_template("register.html", |err: &Compat<Error>| {
            let err = err.to_string();
            match coerce!(&err => &str) {
                r#"new row for relation "users" violates check constraint "name_fmt""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_username"],
                    vec!["Your username must contain only ASCII letters and digits"],
                )),
                r#"new row for relation "users" violates check constraint "name_len""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_username"],
                    vec!["Your username must be at least 3 characters"],
                )),
                r#"duplicate key value violates unique constraint "users_name_key""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_username"],
                    vec!["This username is already taken"],
                )),
                r#"new row for relation "users" violates check constraint "email_fmt""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_email"],
                    vec!["That doesn't look like an email address..."],
                )),
                r#"new row for relation "users" violates check constraint "email_len""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_email"],
                    vec!["That doesn't look like an email address..."],
                )),
                r#"duplicate key value violates unique constraint "users_email_key""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_email"],
                    vec!["This email is already registered"],
                )),
                _ => None,
            }
        })
}
