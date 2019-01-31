use failure::Error;
use futures::{Async, Future};
use warp::{Filter, Rejection};

/// An extension trait for Futures.
pub trait FutureExt: Sized {
    /// Converts an error to a `warp::Rejection`.
    fn err_to_rejection(self) -> ErrToRejection<Self>;
}

impl<F: Future<Error = Error>> FutureExt for F {
    fn err_to_rejection(self) -> ErrToRejection<Self> {
        ErrToRejection(self)
    }
}

/// A wrapper that converts errors to Rejections.
pub struct ErrToRejection<F>(F);

impl<F: Future<Error = Error>> Future for ErrToRejection<F> {
    type Item = F::Item;
    type Error = Rejection;

    fn poll(&mut self) -> Result<Async<F::Item>, Rejection> {
        match self.0.poll() {
            Ok(x) => Ok(x),
            Err(e) => Err(warp::reject::custom(e.compat())),
        }
    }
}

/// Inserts a value into the request extensions.
pub fn set<T: 'static + Clone + Send + Sync>(
    t: T,
) -> impl Clone + Filter<Extract = (), Error = Rejection> {
    warp::any()
        .map(move || warp::ext::set(t.clone()))
        .and_then(|()| -> Result<(), Rejection> { Ok(()) })
        .untuple_one()
}
