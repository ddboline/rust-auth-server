#![allow(unused_imports)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::similar_names)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::missing_errors_doc)]

#[macro_use]
extern crate diesel;

mod auth_handler;
mod auth_routes;
mod change_password_handler;
mod change_password_routes;
mod email_service;
mod errors;
mod google_openid;
mod invitation_handler;
mod invitation_routes;
pub mod logged_user;
mod models;
mod register_handler;
mod register_routes;
pub mod rust_auth_server;
mod schema;
mod ses_client;
mod static_files;
pub mod utils;
