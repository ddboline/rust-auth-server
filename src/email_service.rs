use anyhow::{format_err, Error};
use log::debug;
use std::env;

use crate::errors::ServiceError;
use crate::models::Invitation;
use crate::ses_client::SesInstance;

pub fn send_invitation(invitation: &Invitation, callback_url: &str) -> Result<(), ServiceError> {
    let ses = SesInstance::new(None);

    let sending_email =
        env::var("SENDING_EMAIL_ADDRESS").expect("SENDING_EMAIL_ADDRESS must be set");

    let email_body = format!(
        "Please click on the link below to complete registration. <br/>
         <a href=\"{}?id={}&email={}\">
         {}</a> <br>
         your Invitation expires on <strong>{}</strong>",
        callback_url,
        invitation.id,
        invitation.email,
        invitation
            .expires_at
            .format("%I:%M %p %A, %-d %B, %C%y")
            .to_string(),
        callback_url,
    );

    ses.send_email(
        &sending_email,
        &invitation.email,
        "You have been invited to join Simple-Auth-Server Rust",
        &email_body,
    )
    .map(|_| debug!("Success"))
    .map_err(|e| ServiceError::BadRequest(format!("Bad request {:?}", e)))
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};
    use std::env;
    use std::path::Path;
    use uuid::Uuid;

    use crate::email_service::send_invitation;
    use crate::models::Invitation;

    #[test]
    #[ignore]
    fn test_send_invitation() {
        let home_dir = env::var("HOME").expect("No HOME directory...");

        let env_file = format!("{}/.config/rust_auth_server/config.env", home_dir);

        if Path::new(&env_file).exists() {
            dotenv::from_path(&env_file).ok();
        } else if Path::new("config.env").exists() {
            dotenv::from_filename("config.env").ok();
        } else {
            dotenv::dotenv().ok();
        }

        let new_invitation = Invitation {
            id: Uuid::new_v4(),
            email: "dboline@mediamath.com".to_string(),
            expires_at: Local::now().naive_local() + Duration::hours(24),
        };

        send_invitation(&new_invitation, "test_url").unwrap();

        assert!(false);
    }
}
