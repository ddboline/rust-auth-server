use actix::Addr;
use actix_web::{web, Error, HttpResponse, ResponseError};
use futures::Future;

use crate::auth_handler::LoggedUser;
use crate::change_password_handler::{ChangePassword, UserData};
use crate::models::DbExecutor;

pub fn change_password_user(
    logged_user: LoggedUser,
    user_data: web::Json<UserData>,
    db: web::Data<Addr<DbExecutor>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let msg = ChangePassword {
        // into_inner() returns the inner string value from Path
        email: logged_user.email,
        password: user_data.password.clone(),
    };

    db.send(msg)
        .from_err()
        .and_then(|db_response| match db_response {
            Ok(slim_user) => Ok(HttpResponse::Ok().json(slim_user)),
            Err(service_error) => Ok(service_error.error_response()),
        })
}
