use client_api::ApiRequest;
use client_api::commands::LookCommand;

#[derive(Debug)]
pub struct RequestChain(Vec<ApiRequest>);

impl RequestChain {
    pub fn new(requests: Vec<ApiRequest>) -> Self {
        Self(requests)
    }

    pub fn build(request: ApiRequest) -> Self {
        match request {
            ApiRequest::Move(_) => Self(vec![request, ApiRequest::Look(LookCommand)]),
            _ => Self(vec![request]),
        }
    }
}

impl IntoIterator for RequestChain {
    type Item = ApiRequest;
    type IntoIter = std::vec::IntoIter<ApiRequest>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
