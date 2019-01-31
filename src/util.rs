//! Various utilities.

use futures::{future::poll_fn, Future};
use log::error;

/// A higher-level version of `tokio_threadpool::blocking`.
pub fn blocking<E, F, T>(func: F) -> impl Future<Item = T, Error = E>
where
    F: FnOnce() -> Result<T, E>,
{
    let mut func = Some(func);
    poll_fn(move || {
        tokio_threadpool::blocking(|| (func.take().unwrap())())
            .map_err(|_| panic!("Blocking operations must be run inside a Tokio thread pool!"))
    })
    .and_then(|r| r)
}

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

/// An explicit trivial cast.
macro_rules! coerce {
    ($e:expr => $t:ty) => {{
        let x: $t = $e;
        x
    }};
}
