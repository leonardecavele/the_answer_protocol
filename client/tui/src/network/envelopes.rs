use api_client::protocol::command::enums::{ApiRequest, ApiResponse};
use uuid::Uuid;

pub struct RequestEnvelope {
    pub id: Uuid,
    pub request: ApiRequest,
}

#[derive(Debug, Clone)]
pub struct ResponseEnvelope {
    pub id: Uuid,
    pub response: ApiResponse,
}
