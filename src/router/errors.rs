use crate::{router::util::FutureExt, view::render_html};
use failure::Error as FailureError;
use futures::{future::result, Future};
use log::error;
use serde_json::json;
use std::error::Error;
use warp::{
    http::{header::CONTENT_TYPE, Response, StatusCode},
    Rejection,
};

/// A handler for unhandled errors.
pub fn internal(e: Rejection) -> impl Future<Item = Response<String>, Error = Rejection> {
    let (name, data, code) = if e.is_not_found() {
        ("404.html", json!({}), StatusCode::NOT_FOUND)
    } else {
        error!("Unhandled error: {:?}", e);
        let data = json!({
            "causes": error_causes(&e).into_iter().map(|e| e.to_string()).collect::<Vec<_>>(),
            "error": e.cause().map(|e| e.to_string()),
        });
        ("error.html", data, StatusCode::INTERNAL_SERVER_ERROR)
    };
    result(render_html(name, data)).map(move |mut r| {
        *r.status_mut() = code;
        r
    })
}

/// A last-chance handler for unhandled errors that pass through the `internal` filter. (Probably
/// template-related ones...)
pub fn last_chance(err: Rejection) -> impl Future<Item = Response<String>, Error = Rejection> {
    let mut msg = format!(
        "Internal Server Error; please email ctf@remexre.xyz with the following:\n\n{:?}",
        err
    );
    let mut err = err.cause().map(|c| coerce!(&**c => &dyn Error));
    while let Some(cause) = err {
        msg += "\n\n";
        msg += &cause.to_string();
        err = cause.cause();
    }
    result(
        Response::builder()
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(msg)
            .map_err(FailureError::from),
    )
    .err_to_rejection()
}

fn error_causes(err: &Rejection) -> Vec<&dyn Error> {
    let mut err = err.cause().map(|c| coerce!(&**c => &dyn Error));
    let mut errs = Vec::new();
    while let Some(cause) = err {
        errs.push(cause);
        err = cause.cause();
    }
    errs
}
