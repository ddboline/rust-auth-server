use async_trait::async_trait;
use chrono::{Duration, Local};
use diesel::{self, PgConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::env::var;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::email_service::send_invitation;
use crate::errors::ServiceError;
use crate::models::{DbExecutor, HandleRequest, Invitation};

#[derive(Deserialize)]
pub struct CreateInvitation {
    pub email: String,
}

#[async_trait]
impl HandleRequest<CreateInvitation> for DbExecutor {
    type Result = Result<Invitation, ServiceError>;

    async fn handle(&self, msg: CreateInvitation) -> Self::Result {
        use crate::schema::invitations::dsl::invitations;

        // creating a new Invitation object with expired at time that is 24 hours from now
        let new_invitation = Invitation {
            id: Uuid::new_v4(),
            email: msg.email,
            expires_at: Local::now().naive_local() + Duration::hours(24),
        };

        let dbex = self.clone();

        let new_invitation_ = new_invitation.clone();
        let inserted_invitation = spawn_blocking(move || {
            let conn = dbex.0.get().map_err(ServiceError::R2D2Error)?;
            diesel::insert_into(invitations)
                .values(&new_invitation_)
                .get_result(&conn)
                .map_err(ServiceError::DbError)
        })
        .await??;

        let callback_url = var("CALLBACK_URL")
            .unwrap_or_else(|_| "http://localhost:3000/register.html".to_string());
        send_invitation(&new_invitation, &callback_url).await?;

        Ok(inserted_invitation)
    }
}
