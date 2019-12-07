use log::debug;
use std::env;

use crate::models::Invitation;
use crate::ses_client::SesInstance;

pub fn send_invitation(invitation: &Invitation, callback_url: &str) {
    let ses = SesInstance::new();

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

    match ses.send_email(
        &sending_email,
        &invitation.email,
        "You have been invited to join Simple-Auth-Server Rust",
        &email_body,
    ) {
        Ok(_) => debug!("Success"),
        Err(e) => debug!("Failure {}", e),
    }
}
