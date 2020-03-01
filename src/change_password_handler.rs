use async_trait::async_trait;
use chrono::Local;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::{
    errors::ServiceError,
    models::{DbExecutor, HandleRequest, Invitation, SlimUser, User},
    utils::hash_password,
};

// UserData is used to extract data from a post request by the client
#[derive(Debug, Deserialize)]
pub struct UserData {
    pub password: String,
}

// to be used to send data via the Actix actor system
#[derive(Debug)]
pub struct ChangePassword {
    pub email: String,
    pub password: String,
}

#[async_trait]
impl HandleRequest<ChangePassword> for DbExecutor {
    type Result = Result<bool, ServiceError>;

    async fn handle(&self, msg: ChangePassword) -> Self::Result {
        use crate::schema::{
            invitations::dsl::{id, invitations},
            users::dsl::{email, password, users},
        };

        let dbex = self.clone();
        spawn_blocking(move || {
            let conn = dbex.0.get()?;
            let password_: String = hash_password(&msg.password)?;

            diesel::update(users.filter(email.eq(msg.email)))
                .set(password.eq(password_))
                .execute(&conn)
                .map_err(|_db_error| ServiceError::BadRequest("Update failed".into()))
                .map(|changed| changed > 0)
        })
        .await?
    }
}
