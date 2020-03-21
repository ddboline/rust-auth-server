use async_trait::async_trait;
use chrono::Local;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::{
    errors::ServiceError,
    logged_user::TRIGGER_DB_UPDATE,
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
pub struct RegisterUser {
    pub invitation_id: String,
    pub password: String,
}

#[async_trait]
impl HandleRequest<RegisterUser> for DbExecutor {
    type Result = Result<SlimUser, ServiceError>;
    async fn handle(&self, msg: RegisterUser) -> Self::Result {
        use crate::schema::{
            invitations::dsl::{id, invitations},
            users::dsl::users,
        };

        // try parsing the string provided by the user as url parameter
        // return early with error that will be converted to ServiceError
        let invitation_id = Uuid::parse_str(&msg.invitation_id)?;

        let dbex = self.clone();
        spawn_blocking(move || {
            let conn = dbex.0.get()?;

            invitations
                .filter(id.eq(invitation_id))
                .load::<Invitation>(&conn)
                .map_err(|_db_error| ServiceError::BadRequest("Invalid Invitation".into()))
                .and_then(|mut result| {
                    if let Some(invitation) = result.pop() {
                        // if invitation is not expired
                        if invitation.expires_at > Local::now().naive_local() {
                            // try hashing the password, else return the error that will be
                            // converted to ServiceError
                            let password: String = hash_password(&msg.password)?;
                            let user = User::from_details(invitation.email, password);
                            let inserted_user: User =
                                diesel::insert_into(users).values(&user).get_result(&conn)?;
                            TRIGGER_DB_UPDATE.set();
                            return Ok(inserted_user.into());
                        }
                    }
                    Err(ServiceError::BadRequest("Invalid Invitation".into()))
                })
        })
        .await?
    }
}
