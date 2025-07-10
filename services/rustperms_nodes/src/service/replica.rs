use std::sync::Arc;

use async_nats::jetstream;
use rustperms::prelude::AsyncManager;
use tonic::{Request, Response, Status};

use crate::proto::{rustperms_replica_proto_server::RustpermsReplicaProto};
use crate::proto::SnapshotResponse;
use crate::proto::CheckPermRequest;
use crate::proto::CheckPermReply;
use rustperms::prelude::*;

#[derive(Debug)]
pub struct ReplicaNode {
    pub manager: Arc<AsyncManager>,
}

#[tonic::async_trait]
impl RustpermsReplicaProto for ReplicaNode {
    async fn check_perm(&self, request: Request<CheckPermRequest>) -> Result<Response<CheckPermReply>, Status> {
        let CheckPermRequest { user_uid, permission, unset_policy } = request.into_inner();
        let result = self.manager.check_perm(&user_uid, &PermissionPath::from_str(&permission)).await;
        Ok(Response::new(CheckPermReply {
            result: result.unwrap_or((unset_policy, MatchType::Exact)).0
        }))
    }
    async fn get_snapshot(
        &self,
        _request: Request<()>,
    ) -> Result<Response<SnapshotResponse>, Status> {
        let reply = SnapshotResponse {
            serialized_groups: self.manager.groups_to_string().await.map_err(|_| Status::internal("Can't encode groups"))?,
            serialized_users: self.manager.users_to_string().await.map_err(|_| Status::internal("Can't encode users"))?,
        };
        Ok(Response::new(reply))
    }
}

use anyhow::{anyhow, Result};
use futures::{StreamExt};
use std::{str::from_utf8};

pub async fn start_nats_event_listener(manager: Arc<AsyncManager>, nats_url: String, event: String) -> Result<(), async_nats::Error> {
    let client = async_nats::connect(nats_url).await?;
    let inbox = client.new_inbox();
    let jetstream = jetstream::new(client);
    let stream = jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: "PERM_WRITE_NATS_EVENT".to_string(),
            subjects: vec![event],
            ..Default::default()
        })
        .await?;

    let consumer_name = shared::uuid::Uuid::new_v4().simple().to_string();
    let consumer = stream.create_consumer(jetstream::consumer::push::Config {
        durable_name: Some(consumer_name.to_string()),
        deliver_subject: inbox.clone(),
        deliver_policy: jetstream::consumer::DeliverPolicy::New,
        ack_policy: jetstream::consumer::AckPolicy::Explicit,
        ..Default::default()
    }).await?;
    let mut messages = consumer.messages().await?;
    while let Some(message) = messages.next().await {
        let message = message?;
        let payload = from_utf8(&message.payload)?;
        tracing::info!("New msg: {payload}");
        match RustpermsDelta::deserialize_from_string(payload) {
            Ok(actions) => manager.apply(actions).await,
            Err(e) => tracing::error!("Can't deserialize delta from string: {}", e),
        }
        message.ack().await?;
    }
    Err(anyhow!("Nats event loop ended!").into())
}