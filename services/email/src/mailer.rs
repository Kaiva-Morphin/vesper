use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;
use anyhow::Result;
use tracing::{error, info};
use crate::ENV;


pub struct Mailer(SmtpTransport);

pub fn build_mailer() -> Mailer {
    info!("Building mailer");
    Mailer(SmtpTransport::relay(&ENV.EMAIL_RELAY)
        .unwrap()
        .credentials(Credentials::new(ENV.EMAIL_USERNAME.clone(), ENV.EMAIL_PASSWORD.clone()))
        .build())
}


impl Mailer {
    pub async fn send_email(&self, msg: &Message) -> Result<()> {
        info!("Sending email");
        let r = self.0.send(msg)?;
        info!("Email sent!");
        if !r.is_positive() {
            error!("Email not sent: {:?}", r);
        }
        Ok(())
    }
}

