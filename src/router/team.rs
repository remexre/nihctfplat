use crate::{
    dal::DB,
    logic,
    router::util::{FilterExt, FutureExt},
    schema::User,
};
use failure::Error;
use futures::Future;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use warp::{
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
        .recover_with_template("team.html", |err| match err.to_string() {
            _ => None,
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
        .boxed()
}
