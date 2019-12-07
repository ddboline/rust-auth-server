use chrono::{DateTime, Duration, Utc};
use failure::{err_msg, Error};
use rusoto_core::Region;
use rusoto_ses::{Body, Content, Destination, Message, SendEmailRequest, Ses, SesClient};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread::sleep;
use std::time;

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
            ses_client: SesClient::new(region.clone()),
            region,
        }
    }

    pub fn send_email(&self, src: &str, dest: &str, sub: &str, msg: &str) -> Result<(), Error> {
        self.ses_client
            .send_email(SendEmailRequest {
                source: src.to_string(),
                destination: Destination {
                    to_addresses: Some(vec![dest.to_string()]),
                    ..Default::default()
                },
                message: Message {
                    subject: Content {
                        data: sub.to_string(),
                        ..Default::default()
                    },
                    body: Body {
                        html: Some(Content {
                            data: msg.to_string(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                },
                ..Default::default()
            })
            .sync()
            .map_err(err_msg)
            .map(|_| ())
    }
}
