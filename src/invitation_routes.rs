use actix::Addr;
use actix_web::{web, Error, HttpResponse, ResponseError};
use futures::future::Future;
use std::env;

use crate::invitation_handler::CreateInvitation;
use crate::models::DbExecutor;

pub async fn register_email(
    signup_invitation: web::Json<CreateInvitation>,
    db: web::Data<Addr<DbExecutor>>,
) -> Result<HttpResponse, Error> {
    let db_response = db.send(signup_invitation.into_inner()).await?;
    match db_response {
        Ok(_) => Ok(HttpResponse::Ok().into()),
        Err(err) => Ok(err.error_response()),
    }
}
