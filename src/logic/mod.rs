//! Business logic.
//!
//! > **Logic** is the "business (or domain) logic" of the application. The router will pull the
//! > necessary information out of the HTTP request, and call into this module as quickly as
//! > possible to do all the actual work.

pub mod auth;

use crate::dal::DB;
use failure::Error;
use futures::Future;
use uuid::Uuid;

/// Creates a team.
pub fn create_team(db: DB, user: i32, name: String) -> impl Future<Item = (), Error = Error> {
    db.create_team(user, name).map(|_| ())
}

/// Joins a team.
pub fn join_team(db: DB, user: i32, team: Uuid) -> impl Future<Item = (), Error = Error> {
    db.join_team(user, team).map(|_| ())
}
