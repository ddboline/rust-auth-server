use actix::{Actor, SyncContext};
use anyhow::Error;
use chrono::{Local, NaiveDateTime};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::convert::From;
use uuid::Uuid;

use crate::schema::{invitations, users};

/// This is db executor actor. can be run in parallel
#[derive(Clone)]
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

pub trait HandleRequest<T> {
    type Result;
    fn handle(&self, req: T) -> Self::Result;
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "users"]
pub struct User {
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
}

impl User {
    pub fn get_authorized_users(pool: &DbExecutor) -> Result<Vec<Self>, Error> {
        use crate::schema::users::dsl::users;
        let conn = pool.0.get()?;
        users.load(&conn).map_err(Into::into)
    }

    pub fn from_details(email: String, password: String) -> Self {
        Self {
            email,
            password,
            created_at: Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "invitations"]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
    pub email: String,
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        Self { email: user.email }
    }
}
