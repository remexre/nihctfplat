//! Bindings to the database.
//!
//! > **DAL**, for lack of a better term (borrowing this one from "data access layer" since I don't
//! > want to use "model"), is the only module that does any talking to the database, or any other
//! > IO or interaction with other kinds of externalized state for that matter.

#[allow(proc_macro_derive_resolution_fallback)]
mod schema;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool, PoolError},
};
use failure::Error;
use futures::{
    future::{err, poll_fn, Either},
    Future,
};
use std::sync::Arc;
use tokio_threadpool::blocking;

/// A pool of connections to the database.
#[allow(missing_debug_implementations)]
#[derive(Clone)]
pub struct DB {
    pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}

impl DB {
    /// Connects to the database with at the given URL.
    pub fn connect(database_url: &str) -> Result<DB, PoolError> {
        let pool = Arc::new(Pool::new(ConnectionManager::new(database_url))?);
        Ok(DB { pool })
    }

    /// Performs a query "asynchronously" (but not really). Diesel currently does not support
    /// async/futures, so we use `tokio_threadpool::blocking` so the database operations don't block
    /// the thread. This does, however, require the future to be run inside a threadpool.  
    ///
    /// NOTE(nathan): This isn't really Diesel's fault; libpq exposes a synchronous interface.
    ///
    /// NOTE(nathan): In theory, this is now the bottleneck for most operations -- as I understand
    /// it, we can only have as many concurrent database operations as threads in the tokio thread
    /// pool, and it's not very hard to exhaust the threadpool. If latency problems are noted,
    /// create the thread pool to have `max_blocking < pool_size`.  This should free up a few
    /// threads for non-database operations. (Given that almost everything is done by talking to
    /// the database, this might not actually be an enormous help, though...)
    fn async_query<E, F, I, T>(&self, mut func: F) -> impl Future<Item = T, Error = Error>
    where
        E: Into<Error>,
        F: FnMut(&PgConnection) -> Result<T, E>,
    {
        match self.pool.get() {
            Ok(conn) => Either::A(
                poll_fn(move || {
                    blocking(|| func(&*conn).map_err(|e| e.into())).map_err(|_| {
                        panic!("Database queries must be run inside a Tokio thread pool!")
                    })
                })
                .and_then(|r| r),
            ),
            Err(e) => Either::B(err(e.into())),
        }
    }
}
