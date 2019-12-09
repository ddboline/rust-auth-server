use actix::Addr;
use actix_web::{web, Error, HttpResponse, ResponseError};
use futures::future::Future;
use std::env;

use crate::invitation_handler::CreateInvitation;
use crate::models::DbExecutor;

pub fn register_email(
    signup_invitation: web::Json<CreateInvitation>,
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    db.send(signup_invitation.into_inner())
        .from_err()
        .and_then(|db_response| match db_response {
            Ok(x) => Ok(HttpResponse::Ok().json(x)),
            Err(err) => Ok(err.error_response()),
        })
}
