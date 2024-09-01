use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::proto::admin_server::Admin;

use crate::proto;

#[derive(Debug, Default)]
pub struct AdminService {
    pub state: Arc<tokio::sync::RwLock<u64>>,
}

#[tonic::async_trait]
impl Admin for AdminService {
    async fn get_request_count(
        &self,
        _request: Request<proto::GetRequestCountRequest>,
    ) -> Result<Response<proto::GetRequestCountResponse>, Status> {
        let count = self.state.read().await;
        let response = proto::GetRequestCountResponse { count: *count };

        Ok(Response::new(response))
    }
}
