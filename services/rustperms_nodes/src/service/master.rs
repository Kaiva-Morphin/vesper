use std::fmt::Display;
use std::sync::Arc;

use async_nats::jetstream::Context;
use rustperms::prelude::{AsyncManager, RustpermsDelta};
use tonic::{Request, Response, Status};
use anyhow::Result;

use crate::db::{PostgreStorage, ReflectedApply, SqlStore};
use crate::proto::rustperms_proto::rustperms_master_proto_server::RustpermsMasterProto;
use crate::proto::rustperms_proto::{SnapshotResponse, WriteRequest};
use crate::ENV;

#[derive(Debug)]
pub struct MasterNode<T : SqlStore> {
    pub manager: AsyncManager,
    pub storage: T,
    pub nats_publisher: Arc<Context>
}


#[tonic::async_trait]
impl RustpermsMasterProto for MasterNode<PostgreStorage> {
    async fn write_changes(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<()>, Status> {
        let WriteRequest{serialized_delta} = request.into_inner();
        let delta = RustpermsDelta::deserialize_from_string(&serialized_delta).map_status(Status::internal(""))?;
        self.manager.reflected_apply(&self.storage, delta).await.map_status(Status::internal(""))?;
        // todo!: revert changes on error
        self.nats_publisher.publish(ENV.PERM_WRITE_NATS_EVENT.clone(), serialized_delta.into()).await.map_status(Status::internal("Can't send nats event! The changes applied to db will not be reflected on replicas!"))?;
        Ok(Response::new(()))
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

pub trait MapStatus<T> {
    fn map_status(self, status: Status) -> Result<T, Status>;
}

impl<T, E : Display> MapStatus<T> for anyhow::Result<T, E> 
{
    fn map_status(self, status: Status) -> Result<T, Status> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                tracing::error!("Error encountered: {}", e);
                Err(status)
            }
        }
    }
}