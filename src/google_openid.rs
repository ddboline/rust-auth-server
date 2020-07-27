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
use base64::{encode_config, URL_SAFE_NO_PAD};
use bcrypt::verify;
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lazy_static::lazy_static;
use log::debug;
use openid::{DiscoveredClient, Options, Token as OpenIdToken, Userinfo};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env::var, sync::Arc};
use tokio::{sync::RwLock, task::spawn_blocking};
use url::Url;

lazy_static! {
    static ref CSRF_TOKENS: RwLock<HashMap<String, CrsfTokenCache>> = RwLock::new(HashMap::new());
}

pub type GoogleClient = RwLock<Arc<DiscoveredClient>>;

struct CrsfTokenCache {
    nonce: String,
    final_url: Url,
    timestamp: DateTime<Utc>,
}

fn get_random_string() -> String {
    let random_bytes: Vec<u8> = (0..16).map(|_| thread_rng().gen::<u8>()).collect();
    encode_config(&random_bytes, URL_SAFE_NO_PAD)
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

pub async fn get_google_client() -> Result<DiscoveredClient, ServiceError> {
    let google_client_id =
        var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable.");
    let google_client_secret = var("GOOGLE_CLIENT_SECRET")
        .expect("Missing the GOOGLE_CLIENT_SECRET environment variable.");
    let issuer_url = Url::parse("https://accounts.google.com").expect("Invalid issuer URL");

    let domain = var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());
    let redirect_url = format!("https://{}/api/callback", domain);

    let client = DiscoveredClient::discover(
        google_client_id,
        google_client_secret,
        Some(redirect_url),
        issuer_url,
    )
    .await?;

    Ok(client)
}

#[derive(Serialize, Deserialize)]
pub struct GetAuthUrlData {
    final_url: String,
}

fn get_auth_url(client: &DiscoveredClient) -> (Url, Options) {
    let options = Options {
        scope: Some("email".into()),
        state: Some(get_random_string()),
        nonce: Some(get_random_string()),
        ..Options::default()
    };
    let url = client.auth_url(&options);
    (url, options)
}

pub async fn auth_url(
    payload: Json<GetAuthUrlData>,
    client: Data<GoogleClient>,
) -> Result<HttpResponse, ServiceError> {
    let payload = payload.into_inner();
    debug!("{:?}", payload.final_url);
    let final_url: Url = payload
        .final_url
        .parse()
        .map_err(|err| ServiceError::BlockingError(format!("Failed to parse url {:?}", err)))?;
    let client = client.read().await.clone();
    let (authorize_url, options) = get_auth_url(&client);

    let csrf_state = options.state.clone().expect("No CSRF state");
    let nonce = options.nonce.clone().expect("No nonce");

    CSRF_TOKENS.write().await.insert(
        csrf_state,
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

async fn request_userinfo(
    client: &DiscoveredClient,
    code: &str,
    nonce: &str,
) -> Result<Userinfo, ServiceError> {
    let token = client.authenticate(code, Some(nonce), None).await?;
    let userinfo = client.request_userinfo(&token).await?;
    Ok(userinfo)
}

pub async fn callback(
    query: Query<CallbackQuery>,
    db: Data<DbExecutor>,
    client: Data<GoogleClient>,
    id: Identity,
) -> Result<HttpResponse, ServiceError> {
    let query = query.into_inner();
    let code = query.code.clone();

    let value = CSRF_TOKENS.write().await.remove(&query.state);
    if let Some(CrsfTokenCache {
        nonce, final_url, ..
    }) = value
    {
        debug!("Nonce {:?}", nonce);

        let userinfo = match request_userinfo(&client.read().await.clone(), &code, &nonce).await {
            Ok(userinfo) => userinfo,
            Err(e) => {
                let new_client = get_google_client().await?;
                *client.write().await = Arc::new(new_client);
                return Err(e);
            }
        };

        if let Some(user_email) = &userinfo.email {
            let email_ = user_email.as_str().to_string();

            let user = {
                let dbex = db.clone();
                spawn_blocking(move || {
                    if let Ok(user) = User::get_by_email(&email_, &dbex) {
                        let user: SlimUser = user.into();
                        Some(user)
                    } else {
                        None
                    }
                })
                .await?
            };

            if let Some(user) = user {
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
    use anyhow::Error;
    use chrono::{Duration, Local};
    use std::{env, path::Path};
    use uuid::Uuid;

    use crate::{
        errors::ServiceError,
        google_openid::{get_auth_url, get_google_client},
        models::Invitation,
    };

    #[tokio::test]
    #[ignore]
    async fn test_google_openid() -> Result<(), Error> {
        let config_dir = dirs::config_dir().expect("No CONFIG directory");
        let env_file = config_dir.join("rust_auth_server").join("config.env");

        if env_file.exists() {
            dotenv::from_path(&env_file).ok();
        } else if Path::new("config.env").exists() {
            dotenv::from_filename("config.env").ok();
        } else {
            dotenv::dotenv().ok();
        }

        let client = get_google_client().await?;
        let (url, _) = get_auth_url(&client);
        assert_eq!(url.domain(), Some("accounts.google.com"));
        assert!(url
            .as_str()
            .contains("redirect_uri=https%3A%2F%2Fwww.ddboline.net%2Fapi%2Fcallback"));
        assert!(url.as_str().contains("scope=openid+email"));
        assert!(url.as_str().contains("response_type=code"));
        Ok(())
    }
}
