
use crate::{endpoints::login::LoginBody, AppState, ENV};
use message_broker::email::types::Email;
use sea_orm::{prelude::Uuid, *};

use anyhow::Result;

use tracing::info;





impl AppState {
    pub async fn send_register_code(&self, email: String) -> Result<()> {
        info!("Trying to send register code!");
        let message = Email::register_code(email, "123456".to_owned());
        let encoded =  bincode::encode_to_vec(&message, bincode::config::standard())?;
        info!("Encoded");
        self.publisher.publish(ENV.EMAIL_SEND_NATS_EVENT.clone(), encoded.into())
            .await?
            .await?;
        Ok(())
    }
}