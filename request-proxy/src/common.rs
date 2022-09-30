#[derive(Eq, Hash, PartialEq, Debug, Clone)]
pub struct RequestId(pub String);

#[derive(Clone, Debug)]
pub struct Request {
    pub request_id: RequestId,
    pub path: String,
    pub body: String
}

#[derive(Clone, Debug)]
pub struct Response {
    pub request_id: RequestId,
    pub body: Option<String>
}
