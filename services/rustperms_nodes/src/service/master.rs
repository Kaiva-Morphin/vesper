use std::marker::PhantomData;

use rustperms::prelude::{AsyncManager, RustpermsDelta};
use tonic::{Request, Response, Status};
use anyhow::Result;
pub mod rustperms_master {
    tonic::include_proto!("rustperms_manager");
}
use crate::db::{PostgreStorage, ReflectedApply, SqlStore};
use crate::rustperms_master::rustperms_master_proto_server::RustpermsMasterProto;
use crate::service::master::rustperms_master::{SnapshotResponse, WriteRequest};

#[derive(Debug)]
pub struct MasterNode<T> {
    pub manager: AsyncManager,
    pub storage: T,
}


#[tonic::async_trait]
impl RustpermsMasterProto for MasterNode<PostgreStorage> {
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
    async fn write_changes(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<()>, Status> {
        let WriteRequest{serialized_delta} = request.into_inner();
        let delta = RustpermsDelta::deserialize_from_string(&serialized_delta).map_status(Status::internal(""))?;
        self.manager.reflected_apply(&self.storage, delta).await.map_status(Status::internal(""))?;
        Ok(Response::new(()))
    }
}
pub trait MapStatus<T> {
    fn map_status(self, status: Status) -> Result<T, Status>;
}

impl<T> MapStatus<T> for anyhow::Result<T, anyhow::Error> 
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