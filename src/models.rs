use actix::{Actor, SyncContext};
use anyhow::Error;
use async_trait::async_trait;
use chrono::{Local, NaiveDateTime};
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use serde::{Deserialize, Serialize};
use std::convert::From;
use uuid::Uuid;

use crate::schema::{invitations, users};

/// This is db executor actor. can be run in parallel
#[derive(Clone)]
pub struct DbExecutor(pub Pool<ConnectionManager<PgConnection>>);

#[async_trait]
pub trait HandleRequest<T>
where
    T: 'static,
{
    type Result;
    async fn handle(&self, req: T) -> Self::Result;
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

    pub fn get_by_email(email_: &str, pool: &DbExecutor) -> Result<Self, Error> {
        use crate::schema::users::dsl::{email, users};
        let conn = pool.0.get()?;
        users
            .filter(email.eq(email_))
            .first(&conn)
            .map_err(Into::into)
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
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
