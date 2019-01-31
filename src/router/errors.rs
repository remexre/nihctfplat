use crate::router::util::FutureExt;
use failure::Error as FailureError;
use futures::{
    future::{err, result},
    Future,
};
use std::error::Error;
use warp::{
    http::{Response, StatusCode},
    Rejection,
};

/// A handler for unhandled internal errors.
pub fn internal(e: Rejection) -> impl Future<Item = Response<String>, Error = Rejection> {
    err(e)
}

/// A last-chance handler for unhandled errors that pass through the `internal` function. (Probably
/// template-related ones...)
pub fn last_chance(err: Rejection) -> impl Future<Item = Response<String>, Error = Rejection> {
    let mut msg = format!(
        "Internal Server Error; please email ctf@remexre.xyz with the following:\n\n{:?}",
        err
    );
    let mut err = err.cause().map(|c| coerce!(&**c => &dyn Error));
    if err.is_some() {
        msg.push('\n');
    }
    while let Some(cause) = err {
        msg.push('\n');
        msg += &cause.to_string();
        err = cause.cause();
    }
    result(
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(msg)
            .map_err(FailureError::from),
    )
    .err_to_rejection()
}
