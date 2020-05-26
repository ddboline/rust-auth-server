use anyhow::Error;
use std::env::var;

use rust_auth_server::rust_auth_server::run_auth_server;

#[actix_rt::main]
async fn main() -> Result<(), Error> {
    let port = var("PORT").ok().and_then(|port| port.parse::<u32>().ok()).unwrap_or(3000);
    run_auth_server(port).await.map_err(Into::into)
}
