use crate::{
    errors::ServiceError,
    models::{DbExecutor, SlimUser, User},
    utils::Token,
};
use actix::Addr;
use actix_identity::Identity;
use actix_web::{
    web,
    web::{Data, Json, Query},
    Error, HttpResponse, ResponseError,
};
use bcrypt::verify;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lazy_static::lazy_static;
use log::debug;
use openidconnect::{
    core::{
        CoreClient, CoreIdTokenClaims, CoreIdTokenVerifier, CoreProviderMetadata, CoreResponseType,
    },
    reqwest::http_client,
    AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env::var};
use tokio::{sync::RwLock, task::spawn_blocking};
use url::Url;

lazy_static! {
    static ref GOOGLE_CLIENT: CoreClient = get_google_client().expect("Failed to load client");
    static ref CSRF_TOKENS: RwLock<HashMap<String, CrsfTokenCache>> = RwLock::new(HashMap::new());
}

struct CrsfTokenCache {
    nonce: Nonce,
    final_url: Url,
    timestamp: DateTime<Utc>,
}

pub async fn cleanup_token_map() {
    let expired_keys: Vec<_> = CSRF_TOKENS
        .read()
        .await
        .iter()
        .filter_map(|(k, t)| {
            if (Utc::now() - t.timestamp).num_seconds() > 3600 {
                Some(k.to_string())
            } else {
                None
            }
        })
        .collect();
    for key in expired_keys {
        CSRF_TOKENS.write().await.remove(&key);
    }
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

#[derive(Serialize, Deserialize)]
pub struct GetAuthUrlData {
    final_url: String,
}

fn get_auth_url() -> (Url, CsrfToken, Nonce) {
    GOOGLE_CLIENT
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .url()
}

pub async fn auth_url(payload: Json<GetAuthUrlData>) -> Result<HttpResponse, ServiceError> {
    let payload = payload.into_inner();
    debug!("{:?}", payload.final_url);
    let final_url: Url = payload
        .final_url
        .parse()
        .map_err(|err| ServiceError::BlockingError(format!("Failed to parse url {:?}", err)))?;
    let (authorize_url, csrf_state, nonce) = spawn_blocking(get_auth_url).await?;
    CSRF_TOKENS.write().await.insert(
        csrf_state.secret().clone(),
        CrsfTokenCache {
            nonce,
            final_url,
            timestamp: Utc::now(),
        },
    );
    Ok(HttpResponse::Ok().body(authorize_url.into_string()))
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

    let value = CSRF_TOKENS.write().await.remove(&query.state);
    if let Some(CrsfTokenCache {
        nonce, final_url, ..
    }) = value
    {
        debug!("Nonce {:?}", nonce);
        let token_response =
            spawn_blocking(move || GOOGLE_CLIENT.exchange_code(code).request(http_client))
                .await?
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
                let body = format!(
                    "{}'{}'{}",
                    r#"<script>!function(){let url = "#,
                    final_url,
                    r#";location.replace(url);}();</script>"#
                );
                return Ok(HttpResponse::Ok().body(body));
            }
        }
        Err(ServiceError::BadRequest("Oauth failed".into()))
    } else {
        Ok(HttpResponse::Ok().body("Csrf Token invalid"))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};
    use std::{env, path::Path};
    use uuid::Uuid;

    use crate::{errors::ServiceError, google_openid::get_auth_url, models::Invitation};

    #[test]
    #[ignore]
    fn test_google_openid() {
        let config_dir = dirs::config_dir().expect("No CONFIG directory");
        let env_file = config_dir.join("rust_auth_server").join("config.env");

        if env_file.exists() {
            dotenv::from_path(&env_file).ok();
        } else if Path::new("config.env").exists() {
            dotenv::from_filename("config.env").ok();
        } else {
            dotenv::dotenv().ok();
        }

        let (url, _, _) = get_auth_url();
        assert_eq!(url.domain(), Some("accounts.google.com"));
        assert!(url
            .as_str()
            .contains("redirect_uri=https%3A%2F%2Fwww.ddboline.net%2Fapi%2Fcallback"));
        assert!(url.as_str().contains("scope=openid+email"));
        assert!(url.as_str().contains("response_type=code"));
    }
}
