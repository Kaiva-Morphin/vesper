use anyhow::Result;
use async_nats::jetstream::Context;

use crate::ENV;

pub async fn build_publisher() -> Result<Context> {
    let nats_url = format!("nats://{}:{}", ENV.NATS_URL, ENV.NATS_PORT);
    let client = async_nats::connect(nats_url).await?;
    Ok(async_nats::jetstream::new(client))
}
