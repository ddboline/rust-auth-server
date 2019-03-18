// to avoid the warning from diesel macros
#![allow(proc_macro_derive_resolution_fallback)]

extern crate actix;
extern crate actix_web;
extern crate bcrypt;
extern crate chrono;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate jsonwebtoken as jwt;
extern crate r2d2;
extern crate serde;
extern crate sparkpost;
extern crate uuid;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

pub mod app;
pub mod auth_handler;
pub mod auth_routes;
pub mod email_service;
pub mod errors;
pub mod invitation_handler;
pub mod invitation_routes;
pub mod models;
pub mod register_handler;
pub mod register_routes;
pub mod schema;
pub mod static_files;
pub mod utils;

use actix::prelude::*;
use actix_web::server;
use diesel::{r2d2::ConnectionManager, PgConnection};
use std::env;
use std::path::Path;

use models::DbExecutor;

pub fn run_auth_server(port: u32, number_of_connections: usize) {
    let home_dir = env::var("HOME").expect("No HOME directory...");

    let env_file = format!("{}/.config/rust_auth_server/config.env", home_dir);

    if Path::new(&env_file).exists() {
        dotenv::from_path(&env_file).ok();
    } else if Path::new("config.env").exists() {
        dotenv::from_filename("config.env").ok();
    } else {
        dotenv::dotenv().ok();
    }

    env_logger::init();
    let database_url = env::var("AUTHDB").expect("PGURL must be set");
    let sys = actix::System::new("Actix_Tutorial");

    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let address: Addr<DbExecutor> =
        SyncArbiter::start(number_of_connections, move || DbExecutor(pool.clone()));

    server::new(move || app::create_app(address.clone()))
        .bind(&format!("127.0.0.1:{}", port))
        .unwrap_or_else(|_| panic!("Can not bind to '127.0.0.1:{}'", port))
        .start();

    sys.run();
}
