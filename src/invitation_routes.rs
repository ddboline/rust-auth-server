use actix_web::{AsyncResponder, FutureResponse, HttpResponse, Json, ResponseError, State};
use futures::future::Future;

use app::AppState;
use invitation_handler::CreateInvitation;

pub fn register_email((signup_invitation, state): (Json<CreateInvitation>, State<AppState>))
    -> FutureResponse<HttpResponse> {
    state
        .db
        .send(signup_invitation.into_inner())
        .from_err()
        .and_then(|db_response| match db_response {
            Ok(invitation) => Ok(HttpResponse::Ok().json(invitation)),
            Err(err) => Ok(err.error_response()),
        }).responder()
}
