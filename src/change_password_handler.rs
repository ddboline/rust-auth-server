use chrono::Local;
use diesel::{PgConnection, RunQueryDsl, ExpressionMethods, QueryDsl};
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, HandleRequest, Invitation, SlimUser, User};
use crate::utils::hash_password;

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

impl HandleRequest<ChangePassword> for DbExecutor {
    type Result = Result<bool, ServiceError>;
    fn handle(&self, msg: ChangePassword) -> Self::Result {
        use crate::schema::invitations::dsl::{id, invitations};
        use crate::schema::users::dsl::{email, password, users};
        let conn: &PgConnection = &self.0.get().unwrap();
        let password_: String = hash_password(&msg.password)?;

        diesel::update(users.filter(email.eq(msg.email)))
            .set(password.eq(password_))
            .execute(conn)
            .map_err(|_db_error| ServiceError::BadRequest("Update failed".into()))
            .map(|changed| changed > 0)
    }
}
