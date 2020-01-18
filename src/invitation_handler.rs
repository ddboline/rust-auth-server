use chrono::{Duration, Local};
use diesel::{self, PgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::env::var;
use uuid::Uuid;

use crate::email_service::send_invitation;
use crate::errors::ServiceError;
use crate::models::{DbExecutor, HandleRequest, Invitation};

#[derive(Deserialize)]
pub struct CreateInvitation {
    pub email: String,
}

impl HandleRequest<CreateInvitation> for DbExecutor {
    type Result = Result<Invitation, ServiceError>;

    fn handle(&self, msg: CreateInvitation) -> Self::Result {
        use crate::schema::invitations::dsl::invitations;
        let conn = self.0.get()?;

        // creating a new Invitation object with expired at time that is 24 hours from now
        let new_invitation = Invitation {
            id: Uuid::new_v4(),
            email: msg.email,
            expires_at: Local::now().naive_local() + Duration::hours(24),
        };

        let inserted_invitation = diesel::insert_into(invitations)
            .values(&new_invitation)
            .get_result(&conn)?;

        let callback_url = var("CALLBACK_URL")
            .unwrap_or_else(|_| "http://localhost:3000/register.html".to_string());
        send_invitation(&new_invitation, &callback_url)?;

        Ok(inserted_invitation)
    }
}
