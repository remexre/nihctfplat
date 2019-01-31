//! Bindings to the database.
//!
//! > **DAL**, for lack of a better term (borrowing this one from "data access layer" since I don't
//! > want to use "model"), is the only module that does any talking to the database, or any other
//! > IO or interaction with other kinds of externalized state for that matter.

mod mailer;
#[allow(proc_macro_derive_resolution_fallback, unused_import_braces)]
mod schema;

pub use crate::dal::mailer::Mailer;
use crate::{
    dal::schema::{auths, users},
    schema::User,
    util::blocking,
};
use chrono::{Duration, Utc};
use diesel::{
    dsl::{insert_into, now},
    prelude::*,
    r2d2::{ConnectionManager, Pool, PoolError},
};
use failure::Error;
use futures::{
    future::{err, Either},
    Future,
};
use std::sync::Arc;
use uuid::Uuid;

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

    /// Creates an authentication record for the given user, returning it.
    pub fn create_auth(
        &self,
        user: i32,
        expires: Option<Duration>,
    ) -> impl Future<Item = Uuid, Error = Error> {
        self.async_query(move |conn| {
            let token = Uuid::new_v4();
            let expires = expires.map(|d| Utc::now() + d);
            insert_into(auths::table)
                .values((
                    auths::id.eq(token),
                    auths::userid.eq(user),
                    auths::expires.eq(expires),
                ))
                .execute(conn)
                .map(|_| token)
        })
    }

    /// Looks up an authentication record, returning the ID of the user it corresponds to.
    pub fn get_auth_user(&self, uuid: Uuid) -> impl Future<Item = i32, Error = Error> {
        self.async_query(move |conn| {
            auths::table
                .filter(auths::expires.lt(now))
                .find(uuid)
                .select(auths::userid)
                .get_result(conn)
        })
    }

    /// Gets a user by ID.
    pub fn get_user(&self, user: i32) -> impl Future<Item = User, Error = Error> {
        self.async_query(move |conn| users::table.find(user).get_result(conn))
    }

    /// Gets a user by username.
    pub fn get_user_by_username(
        &self,
        username: String,
    ) -> impl Future<Item = User, Error = Error> {
        self.async_query(move |conn| {
            users::table
                .filter(users::name.eq(&username))
                .get_result(conn)
        })
    }

    /// Performs a query "asynchronously" (but not really). Diesel currently does not support
    /// async/futures, so we use `crate::util::blocking` so the database operations don't block
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
    fn async_query<E, F, T>(&self, mut func: F) -> impl Future<Item = T, Error = Error>
    where
        E: Into<Error>,
        F: FnMut(&PgConnection) -> Result<T, E>,
    {
        match self.pool.get() {
            Ok(conn) => Either::A(blocking(move || func(&*conn).map_err(|e| e.into()))),
            Err(e) => Either::B(err(e.into())),
        }
    }
}
