use crate::service::Context;
use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use tide::{Body, Request, Response};

pub fn register_handlers(app: &mut tide::Server<Context>) -> Result<()> {
    app.at("/api/auth").post(auth_handler);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    address: String,
    timestamp: i64,
    signature: String,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
}

async fn auth_handler(mut req: Request<Context>) -> tide::Result {
    let data: AuthRequest = req.body_json().await?;
    println!(
        "address: {}, timestamp: {}, signature: {}",
        data.address, data.timestamp, data.signature
    );

    let res = AuthResponse {
        access_token: "access_token".into(),
    };

    Ok(Response::builder(200).body(Body::from_json(&res)?).build())
}
