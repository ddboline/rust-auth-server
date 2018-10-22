use actix::prelude::*;
use actix_web::{middleware::Logger, App, http::Method};
use models::DbExecutor;
use invitation_routes::{register_email}   ;

pub struct AppState {
    pub db: Addr<DbExecutor>,
}

/// creates and returns the app after mounting all routes/resources
pub fn create_app(db: Addr<DbExecutor>) -> App<AppState> {
    App::with_state(AppState { db })
        .middleware(Logger::default())

        // routes for authentication
        .resource("/auth", |_r| {
        })
        // routes to invitation
        .resource("/invitation", |r| {
            r.method(Method::POST).with(register_email);
        })
        // routes to register as a user after the
        .resource("/register", |_r| {
        })

}
