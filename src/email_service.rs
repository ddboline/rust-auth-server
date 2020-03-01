use anyhow::{format_err, Error};
use log::debug;
use std::env;

use crate::{errors::ServiceError, models::Invitation, ses_client::SesInstance};

pub async fn send_invitation(
    invitation: &Invitation,
    callback_url: &str,
) -> Result<(), ServiceError> {
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
    .await
    .map(|_| debug!("Success"))
    .map_err(|e| ServiceError::BadRequest(format!("Bad request {:?}", e)))
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Local};
    use std::{env, path::Path};
    use uuid::Uuid;

    use crate::{email_service::send_invitation, errors::ServiceError, models::Invitation};

    #[tokio::test]
    #[ignore]
    async fn test_send_invitation() -> Result<(), ServiceError> {
        let config_dir = dirs::config_dir().expect("No CONFIG directory");
        let env_file = config_dir.join("rust_auth_server").join("config.env");

        if env_file.exists() {
            dotenv::from_path(&env_file).ok();
        } else if Path::new("config.env").exists() {
            dotenv::from_filename("config.env").ok();
        } else {
            dotenv::dotenv().ok();
        }

        let new_invitation = Invitation {
            id: Uuid::new_v4(),
            email: "ddboline.im@gmail.com".to_string(),
            expires_at: Local::now().naive_local() + Duration::hours(24),
        };

        send_invitation(&new_invitation, "test_url").await?;
        Ok(())
    }
}
