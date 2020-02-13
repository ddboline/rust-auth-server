use actix::Addr;
use actix_identity::Identity;
use actix_web::web::{Data, Json};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, ResponseError};
use futures::Future;

use crate::auth_handler::AuthData;
use crate::logged_user::LoggedUser;
use crate::models::{DbExecutor, HandleRequest};
use crate::utils::Token;

pub async fn login(
    auth_data: Json<AuthData>,
    id: Identity,
    db: Data<DbExecutor>,
) -> Result<HttpResponse, Error> {
    db.handle(auth_data.into_inner())
        .await
        .map(|(user, token)| {
            id.remember(token.into());
            HttpResponse::Ok().json(user)
        })
        .map_err(Into::into)
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Ok().finish()
}

pub fn get_me(logged_user: LoggedUser) -> HttpResponse {
    HttpResponse::Ok().json(logged_user)
}
