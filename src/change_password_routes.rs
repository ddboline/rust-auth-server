use actix::Addr;
use actix_web::{
    web,
    web::{Data, Json},
    Error, HttpResponse, ResponseError,
};
use futures::Future;
use maplit::hashmap;

use crate::{
    change_password_handler::{ChangePassword, UserData},
    logged_user::LoggedUser,
    models::{DbExecutor, HandleRequest},
};

pub async fn change_password_user(
    logged_user: LoggedUser,
    user_data: Json<UserData>,
    db: Data<DbExecutor>,
) -> Result<HttpResponse, Error> {
    let msg = ChangePassword {
        // into_inner() returns the inner string value from Path
        email: logged_user.email,
        password: user_data.password.clone(),
    };

    let db_response = db.handle(msg).await;

    match db_response {
        Ok(success) => {
            let status = if success { "success" } else { "failure" };
            let result = hashmap! { "status" => status };
            Ok(HttpResponse::Ok().json(result))
        }
        Err(service_error) => Ok(service_error.error_response()),
    }
}
