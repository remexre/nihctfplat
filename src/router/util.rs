use crate::{router::auth, view::render_html};
use either::Either;
use failure::{Compat, Error};
use futures::{Async, Future};
use serde_json::Value;
use std::collections::HashMap;
use warp::{filters::BoxedFilter, http::Response, Filter, Rejection, Reply};

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

/// An extension trait for Filters.
pub trait FilterExt<T>: Sized {
    /// An error-handling function. The argument function should return keys to set to true.
    fn recover_with_template<F>(
        self,
        template: &'static str,
        func: F,
    ) -> BoxedFilter<(Either<T, Response<String>>,)>
    where
        F: 'static + Clone + Fn(&Error) -> Option<Vec<&'static str>> + Send + Sync;
}

impl<Fi, T> FilterExt<T> for Fi
where
    Fi: 'static + Filter<Extract = (T,), Error = Rejection> + Send + Sync,
    T: 'static + Reply + Send + Sync,
{
    fn recover_with_template<F>(
        self,
        template: &'static str,
        func: F,
    ) -> BoxedFilter<(Either<T, Response<String>>,)>
    where
        F: 'static + Clone + Fn(&Error) -> Option<Vec<&'static str>> + Send + Sync,
    {
        self.map(Ok)
            .recover(|e| Ok(Err(e)))
            .unify()
            .and(auth::opt_auth())
            .and_then(move |res: Result<T, Rejection>, me| match res {
                Ok(r) => Ok(Either::Left(r)),
                Err(r) => match r.find_cause::<Compat<Error>>() {
                    Some(err) => match func(err.get_ref()) {
                        Some(codes) => {
                            let mut hm = HashMap::new();
                            let _ = hm.insert("me", serde_json::to_value(me).unwrap());
                            for code in codes {
                                let _ = hm.insert(code, Value::Bool(true));
                            }
                            render_html(template, hm).map(Either::Right)
                        }
                        None => Err(r),
                    },
                    None => Err(r),
                },
            })
            .boxed()
    }
}

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
        .and_then(|()| -> Result<(), Rejection> { Ok(()) }) // Since Never is private
        .untuple_one()
}
