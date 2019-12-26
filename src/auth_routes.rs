use actix::Addr;
use actix_identity::Identity;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, ResponseError};
use futures::Future;

use crate::auth_handler::{AuthData, LoggedUser};
use crate::models::DbExecutor;
use crate::utils::create_token;

pub async fn login(
    auth_data: web::Json<AuthData>,
    id: Identity,
    db: web::Data<Addr<DbExecutor>>,
) -> Result<HttpResponse, Error> {
    let res = db.send(auth_data.into_inner()).await?;

    match res {
        Ok(user) => {
            let token = create_token(&user)?;
            id.remember(token);
            Ok(HttpResponse::Ok().into())
        }
        Err(err) => Ok(err.error_response()),
    }
}

pub fn logout(id: Identity) -> HttpResponse {
    id.forget();
    HttpResponse::Ok().finish()
}

pub fn get_me(logged_user: LoggedUser) -> HttpResponse {
    HttpResponse::Ok().json(logged_user)
}
