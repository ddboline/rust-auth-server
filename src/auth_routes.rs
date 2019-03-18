#![allow(clippy::needless_pass_by_value)]

use actix_web::middleware::identity::RequestIdentity;
use actix_web::{AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json, ResponseError};
use futures::future::Future;

use crate::app::AppState;
use crate::auth_handler::{AuthData, LoggedUser};
use crate::utils::create_token;

pub fn login(
    (auth_data, req): (Json<AuthData>, HttpRequest<AppState>),
) -> FutureResponse<HttpResponse> {
    req.state()
        .db
        .send(auth_data.into_inner())
        .from_err()
        .and_then(move |res| match res {
            Ok(user) => {
                let token = create_token(&user)?;
                req.remember(token);
                Ok(HttpResponse::Ok().into())
            }
            Err(err) => Ok(err.error_response()),
        })
        .responder()
}

pub fn logout(req: HttpRequest<AppState>) -> HttpResponse {
    req.forget();
    HttpResponse::Ok().into()
}

pub fn get_me(logged_user: LoggedUser) -> HttpResponse {
    HttpResponse::Ok().json(logged_user)
}
