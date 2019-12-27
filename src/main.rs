use failure::{err_msg, Error};

use rust_auth_server::rust_auth_server::run_auth_server;

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    run_auth_server(3000).await.map_err(err_msg)
}
