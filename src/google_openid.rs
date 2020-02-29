use actix::Addr;
use actix_identity::Identity;
use actix_web::web::{Data, Json, Query};
use actix_web::{web, Error, HttpResponse, ResponseError};
use bcrypt::verify;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lazy_static::lazy_static;
use openidconnect::core::{
    CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
};
use openidconnect::reqwest::http_client;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env::var;
use tokio::task::spawn_blocking;
use url::Url;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, SlimUser, User};
use crate::utils::Token;

lazy_static! {
    static ref GOOGLE_CLIENT: CoreClient = get_google_client().expect("Failed to load client");
    static ref CSRF_TOKENS: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

fn get_google_client() -> Result<CoreClient, ServiceError> {
    let google_client_id = ClientId::new(
        var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");

    // Fetch Google's OpenID Connect discovery document.
    let provider_metadata =
        CoreProviderMetadata::discover(&issuer_url, http_client).map_err(|err| {
            ServiceError::BlockingError(format!("Failed to discover OpenID Provider {:?}", err))
        })?;

    let domain = var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
    let redirect_url = format!("https://{}/api/callback", domain);
    let redirect_url = Url::parse(&redirect_url).expect("failed to parse url");

    // Set up the config for the Google OAuth2 process.
    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url.into_string()).expect("Invalid redirect URL"));
    Ok(client)
}

pub async fn auth_url() -> HttpResponse {
    let (authorize_url, csrf_state, nonce) = GOOGLE_CLIENT
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();
    CSRF_TOKENS
        .write()
        .insert(csrf_state.secret().clone(), nonce.secret().clone());
    HttpResponse::Ok().body(authorize_url.into_string())
}

#[derive(Serialize, Deserialize)]
pub struct CallbackQuery {
    code: String,
    state: String,
}

pub async fn callback(
    query: Query<CallbackQuery>,
    db: Data<DbExecutor>,
    id: Identity,
) -> Result<HttpResponse, ServiceError> {
    let query = query.into_inner();
    let code = AuthorizationCode::new(query.code.clone());

    if let Some(nonce) = CSRF_TOKENS.read().get(&query.state) {
        let nonce = Nonce::new(nonce.clone());
        let token_response = GOOGLE_CLIENT
            .exchange_code(code)
            .request(http_client)
            .map_err(|err| {
                ServiceError::BlockingError(format!("Failed to obtain token {:?}", err))
            })?;
        let id_token_verifier: CoreIdTokenVerifier = GOOGLE_CLIENT.id_token_verifier();
        let id_token_claims: &CoreIdTokenClaims = token_response
            .extra_fields()
            .id_token()
            .expect("Server did not return an ID token")
            .claims(&id_token_verifier, &nonce)
            .map_err(|err| {
                ServiceError::BlockingError(format!("Failed to obtain claims {:?}", err))
            })?;

        if let Some(user_email) = id_token_claims.email() {
            use crate::schema::users::dsl::{email, users};

            let email_ = user_email.as_str().to_string();
            let dbex = db.clone();
            let user: Result<Option<_>, ServiceError> = spawn_blocking(move || {
                let conn = dbex.0.get()?;

                let mut items = users.filter(email.eq(&email_)).load::<User>(&conn)?;

                if let Some(user) = items.pop() {
                    let user: SlimUser = user.into();
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            })
            .await?;
            if let Some(user) = user? {
                let token = Token::create_token(&user)?;
                id.remember(token.into());
                return Ok(HttpResponse::Ok().json(user));
            }
        }
        Err(ServiceError::BadRequest("Oauth failed".into()))
    } else {
        Ok(HttpResponse::Ok().body("Csrf Token invalid"))
    }
}
