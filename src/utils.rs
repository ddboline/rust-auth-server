use bcrypt::{hash, DEFAULT_COST};
use errors::ServiceError;
use std::env;
use models::SlimUser;


pub fn hash_password(plain: &str) -> Result<String, ServiceError> {
    // get the hashing cost from the env variable or use default
    let hashing_cost: u32 = match env::var("HASH_ROUNDS") {
        Ok(cost) => cost.parse().unwrap_or(DEFAULT_COST),
        _ => DEFAULT_COST,
    };
    hash(plain, hashing_cost).map_err(|_| ServiceError::InternalServerError)
}

// we are simply serializing the data to a string
// I leave it to you to use a JWT lib here 
pub fn create_token(data: &SlimUser) -> Result<String, ServiceError> {
    serde_json::to_string(&data)
        .map_err(|_err| ServiceError::InternalServerError)
}

pub fn verify_token(token: &str) -> Result<SlimUser, ServiceError> {
    serde_json::from_str(&token)
        .map_err(|_err| ServiceError::Unauthorized)
}
