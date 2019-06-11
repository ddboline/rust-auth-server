use actix::Addr;
use actix_web::{web, Error, HttpResponse, ResponseError};
use futures::future::Future;
use std::env;

use crate::email_service::send_invitation;
use crate::invitation_handler::CreateInvitation;
use crate::models::DbExecutor;

pub fn register_email(
    signup_invitation: web::Json<CreateInvitation>,
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(signup_invitation.into_inner())
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
}
