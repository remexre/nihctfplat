use crate::{
    dal::DB,
    logic,
    router::util::{FilterExt, FutureExt},
    schema::User,
};
use failure::{Compat, Error};
use futures::Future;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{
    filters::body::BodyDeserializeError,
    http::{header::LOCATION, Response, StatusCode},
    Filter,
};

/// A wrapper type for a team's member's names.
#[derive(Clone, Debug, Serialize)]
pub struct TeamMembers(pub Vec<String>);

/// The route for creating a team.
pub fn create() -> Resp!() {
    #[derive(Debug, Deserialize)]
    struct Form {
        name: String,
    }

    warp::body::content_length_limit(2 * 1024)
        .and(warp::ext::get::<DB>())
        .and(warp::ext::get::<User>())
        .and(warp::body::form())
        .and_then(|db: DB, user: User, form: Form| {
            logic::create_team(db, user.id, form.name)
                .and_then(|()| {
                    Response::builder()
                        .header(LOCATION, "/team")
                        .status(StatusCode::FOUND)
                        .body("")
                        .map_err(Error::from)
                })
                .err_to_rejection()
        })
        .recover_with_template("create-team.html", |err: &Compat<Error>| {
            let err = err.to_string();
            match coerce!(&err => &str) {
                r#"new row for relation "teams" violates check constraint "name_fmt""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_name"],
                    vec!["Your group name must contain only ASCII letters and digits"],
                )),
                r#"new row for relation "teams" violates check constraint "name_len""# => Some((
                    StatusCode::BAD_REQUEST,
                    vec!["bad_name"],
                    vec!["Your group name must be at least 3 characters"],
                )),
                _ => None,
            }
        })
}

/// The route for joining a team.
pub fn join() -> Resp!() {
    #[derive(Debug, Deserialize)]
    struct Form {
        join_code: Uuid,
    }

    warp::body::content_length_limit(2 * 1024)
        .and(warp::ext::get::<DB>())
        .and(warp::ext::get::<User>())
        .and(warp::body::form())
        .and_then(|db: DB, user: User, form: Form| {
            logic::join_team(db, user.id, form.join_code)
                .and_then(|()| {
                    Response::builder()
                        .header(LOCATION, "/team")
                        .status(StatusCode::FOUND)
                        .body("")
                        .map_err(Error::from)
                })
                .err_to_rejection()
        })
        .recover_with_template("join-team.html", |_: &BodyDeserializeError| {
            Some((
                StatusCode::BAD_REQUEST,
                vec!["bad_join_code"],
                vec!["Your join code was invalid."],
            ))
        })
        .recover_with_template("join-team.html", |err: &Compat<Error>| {
            let err = dbg!(err.to_string());
            match coerce!(&err => &str) {
                _ => None,
            }
        })
}
