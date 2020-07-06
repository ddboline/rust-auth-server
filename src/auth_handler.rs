use actix_identity::Identity;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use async_trait::async_trait;
use bcrypt::verify;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use futures::{
    executor::block_on,
    future::{ready, Ready},
};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin, task::Poll};
use tokio::task::spawn_blocking;

use crate::{
    errors::ServiceError,
    models::{DbExecutor, HandleRequest, SlimUser, User},
    utils::Token,
};

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

#[async_trait]
impl HandleRequest<AuthData> for DbExecutor {
    type Result = Result<(SlimUser, Token), ServiceError>;

    async fn handle(&self, msg: AuthData) -> Self::Result {
        use crate::schema::users::dsl::{email, users};

        let dbex = self.clone();
        spawn_blocking(move || {
            let conn = dbex.0.get()?;
            let email_ = msg.email.clone();
            let user = users.filter(email.eq(&email_)).first::<User>(&conn)?;
            match verify(&msg.password, &user.password) {
                Ok(true) => {
                    let user: SlimUser = user.into();
                    let token = Token::create_token(&user)?;
                    Ok((user, token))
                }
                _ => Err(ServiceError::BadRequest(
                    "Username and Password don't match".into(),
                )),
            }
        })
        .await?
    }
}
