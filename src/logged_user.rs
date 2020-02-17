use actix_identity::Identity;
use actix_web::FromRequest;
use actix_web::{dev::Payload, Error, HttpRequest};
use chrono::{DateTime, Utc};
use futures::executor::block_on;
use futures::future::{ready, Ready};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

use crate::errors::ServiceError;
use crate::models::{DbExecutor, User};
use crate::utils::{Claim, Token};

lazy_static! {
    pub static ref AUTHORIZED_USERS: AuthorizedUsers = AuthorizedUsers::new();
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct LoggedUser {
    pub email: String,
}

impl<'a> From<Claim> for LoggedUser {
    fn from(claim: Claim) -> Self {
        Self {
            email: claim.get_email(),
        }
    }
}

fn _from_request(req: &HttpRequest, pl: &mut Payload) -> Result<LoggedUser, actix_web::Error> {
    if let Ok(s) = env::var("TESTENV") {
        if &s == "true" {
            return Ok(LoggedUser {
                email: "user@test".to_string(),
            });
        }
    }
    if let Some(identity) = block_on(Identity::from_request(req, pl))?.identity() {
        let user: LoggedUser = Token::decode_token(&identity.into())?.into();
        if AUTHORIZED_USERS.is_authorized(&user) {
            return Ok(user);
        }
    }
    Err(ServiceError::Unauthorized.into())
}

impl FromRequest for LoggedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        ready(_from_request(req, pl))
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
enum AuthStatus {
    Authorized(DateTime<Utc>),
    NotAuthorized,
}

use evmap::shallow_copy::ShallowCopy;
use evmap::{ReadHandle, ReadHandleFactory, Values, WriteHandle};
use parking_lot::Mutex;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use thread_local::ThreadLocal;

impl ShallowCopy for AuthStatus {
    unsafe fn shallow_copy(&self) -> ManuallyDrop<Self> {
        ManuallyDrop::new(*self)
    }
}

#[derive(Debug)]
pub struct AuthorizedUsers {
    reader: Arc<ThreadLocal<ReadHandle<LoggedUser, AuthStatus>>>,
    factory: ReadHandleFactory<LoggedUser, AuthStatus>,
    writer: Mutex<WriteHandle<LoggedUser, AuthStatus>>,
}

impl AuthorizedUsers {
    pub fn new() -> Self {
        let (reader, writer) = evmap::new::<LoggedUser, AuthStatus>();
        Self {
            reader: Arc::new(ThreadLocal::new()),
            factory: reader.factory(),
            writer: Mutex::new(writer),
        }
    }

    fn get_reader(&self) -> &ReadHandle<LoggedUser, AuthStatus> {
        self.reader.get_or(|| self.factory.handle())
    }

    pub fn is_authorized(&self, user: &LoggedUser) -> bool {
        if let Some(guard) = self.get_reader().get(user) {
            for val in guard.iter() {
                if let AuthStatus::Authorized(last_time) = val {
                    let current_time = Utc::now();
                    if (current_time - *last_time).num_minutes() < 15 {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn store_auth(&self, users: &[LoggedUser], is_auth: bool) -> Result<(), anyhow::Error> {
        let current_time = Utc::now();
        let status = if is_auth {
            AuthStatus::Authorized(current_time)
        } else {
            AuthStatus::NotAuthorized
        };
        let mut writer = self.writer.lock();
        for user in users {
            writer.insert(user.clone(), status);
        }
        writer.refresh();
        Ok(())
    }

    pub fn merge_users(&self, users: &[LoggedUser]) -> Result<(), anyhow::Error> {
        let mut new_users = Vec::new();
        for (user, _) in self.get_reader().read().iter() {
            if !users.contains(user) {
                new_users.push(user.clone());
            }
        }
        self.store_auth(&new_users, false)?;
        self.store_auth(users, true)?;
        Ok(())
    }

    pub fn get_keys(&self) -> Vec<LoggedUser> {
        self.get_reader()
            .read()
            .iter()
            .map(|(k, _)| k.clone())
            .collect()
    }
}

pub fn fill_auth_from_db(pool: &DbExecutor) -> Result<(), anyhow::Error> {
    let users: Vec<_> = User::get_authorized_users(pool)?
        .into_iter()
        .map(|user| LoggedUser { email: user.email })
        .collect();

    AUTHORIZED_USERS.merge_users(&users)
}
