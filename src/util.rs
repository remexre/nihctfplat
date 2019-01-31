//! Various utilities.

use log::error;

/// Logs an error, including its causes and backtrace (if possible).
pub fn log_err(err: &failure::Error) {
    let mut first = true;
    let num_errs = err.iter_chain().count();
    if num_errs <= 1 {
        error!("{}", err);
    } else {
        for cause in err.iter_chain() {
            if first {
                first = false;
                error!("           {}", cause);
            } else {
                error!("caused by: {}", cause);
            }
        }
    }
    let bt = err.backtrace().to_string();
    if bt != "" {
        error!("{}", bt);
    }
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
