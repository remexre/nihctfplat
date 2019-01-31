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
#[derive(Debug)]
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

/// The type of a responder. Since `impl Trait` can't be used in `type` items, this magics one up.
macro_rules! Resp {
    () => { warp::filters::BoxedFilter<(impl warp::Reply,)> };
}

/// Inserts `.or(...)` between the given filters.
macro_rules! route_any {
    ($hm:ident $hp:tt => $h:expr $(, $tm:ident $tp:tt => $t:expr)* $(,)*) => {
        route_any!(@internal @path $hm $hp).and($h)
            $(.or(route_any!(@internal @path $tm $tp).and($t)))*
    };

    (@internal @path GET ()) => {{ warp::get2() }};
    (@internal @path POST ()) => {{ warp::post2() }};
    (@internal @path $m:ident $p:tt) => {{
        use warp::path;
        route_any!(@internal @path $m ()).and(path! $p)
    }};
}
