//! Types used throughout.
//!
//! > Schema defines the plain old data types that views operate on. Notably, the schema module has
//! > no knowledge of the database, nor any dependencies on any of the rest of the system.

use serde_derive::Serialize;

/// A user.
#[derive(Clone, Debug, Queryable, Serialize)]
pub struct User {
    /// The user's database ID.
    #[serde(skip)]
    pub id: i32,

    /// The user's name.
    pub name: String,

    /// The user's email address.
    pub email: String,

    /// The database ID of the user's group.
    pub group: Option<i32>,
}
