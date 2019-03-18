use crate::auth_routes::{get_me, login, logout};
use crate::invitation_routes::register_email;
use crate::models::DbExecutor;
use crate::register_routes::register_user;
use crate::static_files::{index_html, login_html, main_css, main_js, register_html};

use ::actix::prelude::*;
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{http::Method, middleware::Logger, App};
use chrono::Duration;

pub struct AppState {
    pub db: Addr<DbExecutor>,
}

/// creates and returns the app after mounting all routes/resources
pub fn create_app(db: Addr<DbExecutor>) -> App<AppState> {
    // secret is a random minimum 32 bytes long base 64 string
    let secret: String = std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(8));
    let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    App::with_state(AppState { db })
        .middleware(Logger::default())
        .middleware(IdentityService::new(
            CookieIdentityPolicy::new(secret.as_bytes())
                .name("auth")
                .path("/")
                .domain(domain.as_str())
                .max_age(Duration::days(1))
                .secure(false), // this can only be true if you have https
        ))
        // everything under '/api/' route
        .scope("/api", |api| {
            // routes for authentication
            api.resource("/auth", |r| {
                r.method(Method::POST).with(login);
                r.method(Method::DELETE).with(logout);
                r.method(Method::GET).with(get_me);
            })
            // routes to invitation
            .resource("/invitation", |r| {
                r.method(Method::POST).with(register_email);
            })
            // routes to register as a user after the
            .resource("/register/{invitation_id}", |r| {
                r.method(Method::POST).with(register_user);
            })
        })
        .scope("/auth", |static_| {
            static_
                .resource("/index.html", |r| {
                    r.method(Method::GET).with(index_html);
                })
                .resource("/main.css", |r| {
                    r.method(Method::GET).with(main_css);
                })
                .resource("/main.js", |r| {
                    r.method(Method::GET).with(main_js);
                })
                .resource("/register.html", |r| {
                    r.method(Method::GET).with(register_html);
                })
                .resource("/login.html", |r| {
                    r.method(Method::GET).with(login_html);
                })
        })
}
