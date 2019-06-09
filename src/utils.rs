use bcrypt::{hash, DEFAULT_COST};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, Header, Validation};

use crate::errors::ServiceError;
use crate::models::SlimUser;

pub fn hash_password(plain: &str) -> Result<String, ServiceError> {
    // get the hashing cost from the env variable or use default
    let hashing_cost: u32 = match std::env::var("HASH_ROUNDS") {
        Ok(cost) => cost.parse().unwrap_or(DEFAULT_COST),
        _ => DEFAULT_COST,
    };
    println!("{}", &hashing_cost);
    hash(plain, hashing_cost).map_err(|_| ServiceError::InternalServerError)
}

// JWT claim
#[derive(Debug, Serialize, Deserialize)]
struct Claim {
    // issuer
    iss: String,
    // subject
    sub: String,
    //issued at
    iat: i64,
    // expiry
    exp: i64,
    // user email
    email: String,
}

// struct to get converted to token and back
impl Claim {
    fn with_email(email: &str) -> Self {
        Claim {
            iss: "localhost".into(),
            sub: "auth".into(),
            email: email.to_owned(),
            iat: Local::now().timestamp(),
            exp: (Local::now() + Duration::hours(24)).timestamp(),
        }
    }
}

impl From<Claim> for SlimUser {
    fn from(claims: Claim) -> Self {
        SlimUser {
            email: claims.email,
        }
    }
}

pub fn create_token(data: &SlimUser) -> Result<String, ServiceError> {
    let claims = Claim::with_email(data.email.as_str());
    encode(&Header::default(), &claims, get_secret().as_ref())
        .map_err(|_err| ServiceError::InternalServerError)
}

pub fn decode_token(token: &str) -> Result<SlimUser, ServiceError> {
    decode::<Claim>(token, get_secret().as_ref(), &Validation::default())
        .map(|data| Ok(data.claims.into()))
        .map_err(|_err| ServiceError::Unauthorized)?
}

fn get_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "my secret".into())
}
