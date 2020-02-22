use actix::Addr;
use actix_web::web::{Data, Json};
use actix_web::{web, Error, HttpResponse, ResponseError};
use futures::future::Future;
use serde::{Deserialize, Serialize};
use std::env;

use crate::invitation_handler::CreateInvitation;
use crate::models::{DbExecutor, HandleRequest};

pub async fn register_email(
    signup_invitation: Json<CreateInvitation>,
    db: Data<DbExecutor>,
) -> Result<HttpResponse, Error> {
    let db_response = db.handle(signup_invitation.into_inner()).await;
    match db_response {
        Ok(x) => Ok(HttpResponse::Ok().json(x)),
        Err(err) => Ok(err.error_response()),
    }
}
