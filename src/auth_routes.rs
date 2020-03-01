use actix::Addr;
use actix_identity::Identity;
use actix_web::{
    web,
    web::{Data, Json},
    Error, HttpRequest, HttpResponse, Responder, ResponseError,
};
use futures::Future;

use crate::{
    auth_handler::AuthData,
    logged_user::LoggedUser,
    models::{DbExecutor, HandleRequest},
    utils::Token,
};

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
