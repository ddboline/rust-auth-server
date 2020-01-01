use actix_identity::Identity;
use actix_web::FromRequest;
use actix_web::{dev::Payload, Error, HttpRequest};
use bcrypt::verify;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use futures::executor::block_on;
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::Poll;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, HandleRequest, SlimUser, User};
use crate::utils::decode_token;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

impl HandleRequest<AuthData> for DbExecutor {
    type Result = Result<SlimUser, ServiceError>;
    fn handle(&self, msg: AuthData) -> Self::Result {
        use crate::schema::users::dsl::{email, users};
        let conn: &PgConnection = &self.0.get().unwrap();

        let mut items = users.filter(email.eq(&msg.email)).load::<User>(conn)?;

        if let Some(user) = items.pop() {
            if let Ok(matching) = verify(&msg.password, &user.password) {
                if matching {
                    return Ok(user.into());
                }
            }
        }
        Err(ServiceError::BadRequest(
            "Username and Password don't match".into(),
        ))
    }
}

// we need the same data
// simple aliasing makes the intentions clear and its more readable
pub type LoggedUser = SlimUser;

fn _from_request(req: &HttpRequest, pl: &mut Payload) -> Result<LoggedUser, Error> {
    if let Some(identity) = block_on(Identity::from_request(req, pl))?.identity() {
        let user: SlimUser = decode_token(&identity)?;
        Ok(user as LoggedUser)
    } else {
        Err(ServiceError::Unauthorized.into())
    }
}

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Ready<Result<LoggedUser, Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        ready(_from_request(req, pl))
    }
}
