use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use rusoto_core::Region;
use rusoto_ses::{Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient};
use std::{collections::HashMap, fmt, fs::File, io::Read, path::Path, thread::sleep, time};
use sts_profile_auth::get_client_sts;

#[derive(Clone)]
pub struct SesInstance {
    ses_client: SesClient,
    region: Region,
}

impl fmt::Debug for SesInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SesInstance")
    }
}

impl Default for SesInstance {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SesInstance {
    pub fn new(region: Option<Region>) -> Self {
        let region = region.unwrap_or(Region::UsEast1);
        Self {
            ses_client: get_client_sts!(SesClient, region.clone())
                .expect("Failed to open SesClient"),
            region,
        }
    }

    pub async fn send_email(
        &self,
        src: &str,
        dest: &str,
        sub: &str,
        msg: &str,
    ) -> Result<(), Error> {
        let req = SendEmailRequest {
            source: src.to_string(),
            destination: Destination {
                to_addresses: Some(vec![dest.to_string()]),
                ..Destination::default()
            },
            message: Message {
                subject: Content {
                    data: sub.to_string(),
                    ..Content::default()
                },
                body: Body {
                    html: Some(Content {
                        data: msg.to_string(),
                        ..Content::default()
                    }),
                    ..Body::default()
                },
            },
            ..SendEmailRequest::default()
        };
        self.ses_client
            .send_email(req)
            .await
            .map_err(Into::into)
            .map(|_| ())
    }
}
