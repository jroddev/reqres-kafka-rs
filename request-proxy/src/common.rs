#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct RequestId(pub String);

#[derive(Clone, Debug)]
pub struct Request {
    pub data: String
}

#[derive(Clone, Debug)]
pub struct Response {
    pub request_id: RequestId,
    pub data: Option<String>
}
