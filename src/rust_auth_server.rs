use std::env;
use std::path::Path;
// use actix::prelude::*;
use actix::{Addr, SyncArbiter};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use chrono::Duration;
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;

use crate::auth_routes;
use crate::change_password_routes;
use crate::invitation_routes;
use crate::models::DbExecutor;
use crate::register_routes;
use crate::static_files::{index_html, login_html, main_css, main_js, register_html};

pub fn run_auth_server(port: u32, number_of_connections: usize) -> std::io::Result<()> {
    let home_dir = env::var("HOME").expect("No HOME directory...");

    let env_file = format!("{}/.config/rust_auth_server/config.env", home_dir);

    if Path::new(&env_file).exists() {
        dotenv::from_path(&env_file).ok();
    } else if Path::new("config.env").exists() {
        dotenv::from_filename("config.env").ok();
    } else {
        dotenv::dotenv().ok();
    }

    std::env::set_var(
        "RUST_LOG",
        "simple-auth-server=debug,actix_web=info,actix_server=info",
    );
    env_logger::init();
    let sys = actix_rt::System::new("example");

    let database_url = std::env::var("AUTHDB").expect("DATABASE_URL must be set");

    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let address: Addr<DbExecutor> =
        SyncArbiter::start(number_of_connections, move || DbExecutor(pool.clone()));

    HttpServer::new(move || {
        // secret is a random minimum 32 bytes long base 64 string
        let secret: String = std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(8));
        let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

        App::new()
            .data(address.clone())
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(secret.as_bytes())
                    .name("auth")
                    .path("/")
                    .domain(domain.as_str())
                    .max_age_time(Duration::days(1))
                    .secure(false), // this can only be true if you have https
            ))
            // everything under '/api/' route
            .service(
                web::scope("/api")
                    // routes for authentication
                    .service(
                        web::resource("/auth")
                            .route(web::post().to_async(auth_routes::login))
                            .route(web::delete().to(auth_routes::logout))
                            .route(web::get().to_async(auth_routes::get_me)),
                    )
                    // routes to invitation
                    .service(
                        web::resource("/invitation")
                            .route(web::post().to_async(invitation_routes::register_email)),
                    )
                    // routes to register as a user after the
                    .service(
                        web::resource("/register/{invitation_id}")
                            .route(web::post().to_async(register_routes::register_user)),
                    )
                    .service(
                        web::resource("/password_change").route(
                            web::post().to_async(change_password_routes::change_password_user),
                        ),
                    ),
            )
            // serve static files
            .service(
                web::scope("/auth")
                    .service(web::resource("/index.html").route(web::get().to(index_html)))
                    .service(web::resource("/main.css").route(web::get().to(main_css)))
                    .service(web::resource("/main.js").route(web::get().to(main_js)))
                    .service(web::resource("/register.html").route(web::get().to(register_html)))
                    .service(web::resource("/login.html").route(web::get().to(login_html))),
            )
    })
    .bind(&format!("127.0.0.1:{}", port))?
    .start();

    sys.run()
}
