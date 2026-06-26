use api_client::protocol::command::enums::{ApiRequest, ApiResponse};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RequestEnvelope {
    pub id: Uuid,
    pub request: ApiRequest,
}

impl RequestEnvelope {
    pub fn new(request: ApiRequest) -> Self {
        Self {
            id: Uuid::new_v4(),
            request,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseEnvelope {
    pub id: Uuid,
    pub response: ApiResponse,
    pub original_request: ApiRequest,
}
