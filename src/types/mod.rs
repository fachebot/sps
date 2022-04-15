use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub address: String,
    pub timestamp: i64,
    pub signature: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct Transport {
    #[serde(rename = "type")]
    pub transport_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Serialize)]
pub struct GetMeResponse {
    pub id: i64,
    pub open_id: String,
    pub project_id: String,
    pub transports: Vec<Transport>,
}
