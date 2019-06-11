use failure::{err_msg, Error};

use rust_auth_server::run_auth_server;

fn main() -> Result<(), Error> {
    run_auth_server(3000, 2).map_err(err_msg)
}
