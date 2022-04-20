use crate::service::Context;
use crate::types::*;
use anyhow::Result;
use async_std::sync::Arc;
use ethers_core::types::{Address, Signature};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use std::collections::BTreeMap;
use tide::{Body, Request, Response};

fn verify_signature(address: &str, signature: &str, timestamp: i64) -> Result<()> {
    let address = address.parse::<Address>()?;
    let signature = signature.parse::<Signature>()?;

    let message = format!(
        "I agree to connect my wallet to the simple push service. Timestamp: {}",
        timestamp
    );

    signature.verify(message, address)?;

    Ok(())
}

pub async fn auth(mut req: Request<Arc<Context>>) -> tide::Result {
    let data: AuthRequest = req.body_json().await?;

    // Verify wallet signature
    verify_signature(&data.address, &data.signature, data.timestamp)?;

    // Ensure that user record exist
    let user_model = &req.state().user_model;
    let result = user_model.find_one_by_wallet_address(&data.address).await;
    if let Err(err) = result {
        if !crate::model::is_not_found_record_err(&err) {
            return Err(err.into());
        }

        let user = crate::model::User::new(&data.address);
        user_model.insert(&user).await?;
    }

    // Generate JWT access token for user
    let now = chrono::Utc::now().timestamp();
    let access_expire = req.state().conf.server.access_expire;

    let mut claims = BTreeMap::new();
    claims.insert("iat", now.to_string());
    claims.insert("exp", (now + access_expire).to_string());
    claims.insert("username", data.address);

    let buf = req.state().conf.server.access_secret.as_bytes();
    let key: Hmac<Sha256> = Hmac::new_from_slice(buf).unwrap();

    let res = AuthResponse {
        access_token: claims.sign_with_key(&key)?,
    };

    Ok(Response::builder(200).body(Body::from_json(&res)?).build())
}
