use crate::service::Context;
use anyhow::Result;
use ethers_core::types::{Address, Signature};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde_derive::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
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

fn verify_signature(address: &str, signature: &str, timestamp: i64) -> Result<()> {
    let address = address.parse::<Address>()?;
    let signature = signature.parse::<Signature>()?;

    let message = format!(
        "I agree to connect my wallet to the simple push service. {}",
        timestamp
    );
    signature.verify(message, address)?;

    Ok(())
}

async fn auth_handler(mut req: Request<Context>) -> tide::Result {
    let data: AuthRequest = req.body_json().await?;

    verify_signature(&data.address, &data.signature, data.timestamp)?;

    let now = chrono::Utc::now().timestamp();
    let access_expire = req.state().conf.server.access_expire;

    let mut claims = BTreeMap::new();
    claims.insert("iat", now.to_string());
    claims.insert("exp", (now + access_expire).to_string());
    claims.insert("address", data.address);

    let buf = req.state().conf.server.access_secret.as_bytes();
    let key: Hmac<Sha256> = Hmac::new_from_slice(buf).unwrap();

    let res = AuthResponse {
        access_token: claims.sign_with_key(&key)?,
    };

    Ok(Response::builder(200).body(Body::from_json(&res)?).build())
}
