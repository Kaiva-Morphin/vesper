use async_nats::jetstream::{self, consumer::PullConsumer};
use bincode::decode_from_slice;
use futures::StreamExt;
use message_broker::email::types::Email;
use tracing::error;

use crate::{mailer::build_mailer, ENV};
use anyhow::Result;

use crate::email_templates::ToMessage;


async fn build_consumer() -> Result<PullConsumer> {
    let nats_url = format!("nats://{}:{}", ENV.NATS_URL, ENV.NATS_PORT).to_string();
    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);
    let consumer: PullConsumer = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "EMAIL_SEND_NATS_EVENT".to_string(),
            subjects: vec![ENV.EMAIL_SEND_NATS_EVENT.clone()],
            ..Default::default()
        })
        .await?
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("consumer".into()),
            ..Default::default()
        })
        .await?;
    Ok(consumer)
}



#[macro_export]
macro_rules! ok_or {
    ($expr:expr ; $msg:expr ; $act:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                error!("{}: {:?}", $msg, e);
                $act
            }
        }
    };
}

pub async fn run_subscriber() -> Result<(), async_nats::Error> {
    let mailer = build_mailer();
    let consumer = build_consumer().await?;
    loop {
        let mut messages = consumer.fetch().max_messages(15).messages().await?;
        while let Some(message) = messages.next().await {
            let message = message?;
            'b : {
                let (email, _) : (Email, usize)= ok_or!(decode_from_slice(&message.payload, bincode::config::standard()) ; "Can't deserialize message!" ; break 'b);
                let msg = ok_or!(email.to_message() ; "Can't convert to message!" ; break 'b);
                ok_or!(mailer.send_email(&msg).await ; "Can't send email!" ;  break 'b);
            }
            message.ack().await?;
        }
    }
}