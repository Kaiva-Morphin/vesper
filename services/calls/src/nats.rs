use std::str::from_utf8;

use async_nats::jetstream;
use futures_util::StreamExt;
use tracing::info;

use crate::{state::AppState, types::{InnerSignal, Receiver}, ENV};






pub async fn run_consumer(
    state: &AppState
) -> Result<(), async_nats::Error> {
    let consumer_name = shared::uuid::Uuid::new_v4().simple().to_string();
    let consumer = state.jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "CALLS_NATS_EVENT".to_string(),
            subjects: vec![ENV.CALLS_NATS_EVENT.clone()],
            ..Default::default()
        })
        .await.expect("Can't create stream!").create_consumer(jetstream::consumer::push::Config {
            durable_name: Some(consumer_name),
            deliver_subject: state.inbox.clone(),
            deliver_policy: jetstream::consumer::DeliverPolicy::New,
            ack_policy: jetstream::consumer::AckPolicy::Explicit,
            ..Default::default()
        })
        .await.expect("Can't create consumer!");
    let mut messages = consumer.messages().await?;
    while let Some(msg) = messages.next().await {
        let msg = msg?;
        info!("Received message: {}", from_utf8(&msg.payload)?);
        let signal: InnerSignal = serde_json::from_slice(&msg.payload)?;
        let clients = state.signal_clients.read().await;
        for sender in clients.values() {
            let _ = sender.send(signal.clone());
        }
        msg.ack().await?;
    }
    Err(anyhow::anyhow!("Nats event loop ended!").into())
}


