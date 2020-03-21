use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Local};
use derive_more::{From, Into};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use log::debug;
use serde::{Deserialize, Serialize};
use std::env;

use crate::{errors::ServiceError, models::SlimUser};

const DEFAULT_ALGORITHM: Algorithm = Algorithm::HS256;

pub fn hash_password(plain: &str) -> Result<String, ServiceError> {
    // get the hashing cost from the env variable or use default
    let hashing_cost: u32 = match env::var("HASH_ROUNDS") {
        Ok(cost) => cost.parse().unwrap_or(DEFAULT_COST),
        _ => DEFAULT_COST,
    };
    debug!("{}", &hashing_cost);
    hash(plain, hashing_cost).map_err(|_| ServiceError::InternalServerError)
}

// JWT claim
#[derive(Debug, Serialize, Deserialize)]
pub struct Claim {
    // issuer
    iss: String,
    // subject
    sub: String,
    // issued at
    iat: i64,
    // expiry
    exp: i64,
    // user email
    email: String,
}

// struct to get converted to token and back
impl Claim {
    fn with_email(email: &str) -> Self {
        let domain = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
        Self {
            iss: domain,
            sub: "auth".into(),
            email: email.to_owned(),
            iat: Local::now().timestamp(),
            exp: (Local::now() + Duration::hours(24)).timestamp(),
        }
    }

    pub fn get_email(self) -> String {
        self.email
    }
}

impl From<Claim> for SlimUser {
    fn from(claims: Claim) -> Self {
        Self {
            email: claims.email,
        }
    }
}

#[derive(From, Into)]
pub struct Token(String);

impl Token {
    pub fn create_token(data: &SlimUser) -> Result<Self, ServiceError> {
        let claims = Claim::with_email(data.email.as_str());
        encode(
            &Header::new(DEFAULT_ALGORITHM),
            &claims,
            &EncodingKey::from_secret(get_secret().as_ref()),
        )
        .map(Into::into)
        .map_err(|_err| ServiceError::InternalServerError)
    }

    pub fn decode_token(token: &Self) -> Result<Claim, ServiceError> {
        decode::<Claim>(
            &token.0,
            &DecodingKey::from_secret(get_secret().as_ref()),
            &Validation::new(DEFAULT_ALGORITHM),
        )
        .map(|data| Ok(data.claims))
        .map_err(|_err| ServiceError::Unauthorized)?
    }
}

fn get_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "my secret".into())
}
