use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, ResponseError, State};
use futures::future::Future;
use std::env;

use crate::app::AppState;
use crate::email_service::send_invitation;
use crate::invitation_handler::CreateInvitation;

pub fn register_email(
    (signup_invitation, state): (Json<CreateInvitation>, State<AppState>),
) -> FutureResponse<HttpResponse> {
    state
        .db
        .send(signup_invitation.into_inner())
        .from_err()
        .and_then(|db_response| match db_response {
            Ok(invitation) => {
                let callback_url = env::var("CALLBACK_URL")
                    .unwrap_or_else(|_| "http://localhost:3000/register.html".to_string());
                send_invitation(&invitation, &callback_url);
                Ok(HttpResponse::Ok().into())
            }
            Err(err) => Ok(err.error_response()),
        })
        .responder()
}
