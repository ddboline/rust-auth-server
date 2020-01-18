use actix_identity::Identity;
use actix_web::FromRequest;
use actix_web::{dev::Payload, Error, HttpRequest};
use bcrypt::verify;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use futures::executor::block_on;
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, HandleRequest, SlimUser, User};
use crate::utils::Token;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

impl HandleRequest<AuthData> for DbExecutor {
    type Result = Result<(SlimUser, Token), ServiceError>;
    fn handle(&self, msg: AuthData) -> Self::Result {
        use crate::schema::users::dsl::{email, users};
        let conn = self.0.get()?;

        let mut items = users.filter(email.eq(&msg.email)).load::<User>(&conn)?;

        if let Some(user) = items.pop() {
            if let Ok(matching) = verify(&msg.password, &user.password) {
                if matching {
                    let user: SlimUser = user.into();
                    let token = Token::create_token(&user)?;
                    return Ok((user, token));
                }
            }
        }
        Err(ServiceError::BadRequest(
            "Username and Password don't match".into(),
        ))
    }
}
