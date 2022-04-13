use anyhow::Result;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use serde::Serialize;
use serde_json::value::RawValue;
use sha2::Sha256;
use std::collections::BTreeMap;
use tide::{Body, Middleware, Request};

#[derive(Clone)]
pub struct JwtAuthMiddleware {
    key: Hmac<Sha256>,
}

impl JwtAuthMiddleware {
    pub fn new(access_secret: &str) -> Result<Self> {
        let buf = Vec::from(access_secret);
        let key: Hmac<Sha256> = Hmac::new_from_slice(&buf)?;

        Ok(JwtAuthMiddleware { key })
    }
}

fn make_unauthorized_error() -> tide::Error {
    tide::Error::new(401, anyhow::anyhow!("Unauthorized"))
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for JwtAuthMiddleware {
    async fn handle(&self, mut req: Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let auth_header = req
            .header("Authorization")
            .ok_or(make_unauthorized_error())?;

        let values: Vec<_> = auth_header.into_iter().collect();
        if values.is_empty() {
            return Err(make_unauthorized_error());
        }

        const PREFIX: &str = "Bearer ";
        for value in values {
            let value = value.as_str();
            if !value.starts_with(PREFIX) {
                continue;
            }

            let token = &value[PREFIX.len()..];
            let result = VerifyWithKey::verify_with_key(token, &self.key)
                .map_err(|_| make_unauthorized_error())?;

            let claims: BTreeMap<String, String> = result;
            let exp = claims.get("exp").ok_or(make_unauthorized_error())?;
            let exp = exp.parse::<i64>().map_err(|_| make_unauthorized_error())?;
            let username = claims.get("username").ok_or(make_unauthorized_error())?;

            if exp < chrono::Utc::now().timestamp() {
                return Err(tide::Error::new(401, anyhow::anyhow!("Token has expired")));
            }

            req.set_ext(username.clone());

            return Ok(next.run(req).await);
        }

        return Err(make_unauthorized_error());
    }
}

#[derive(Default, Serialize)]
struct JsonResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Box<RawValue>>,
}

#[derive(Clone)]
pub struct JsonResponseMiddleware {}

impl JsonResponseMiddleware {
    pub fn new() -> Self {
        JsonResponseMiddleware {}
    }
}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for JsonResponseMiddleware {
    async fn handle(&self, req: Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let mut res = next.run(req).await;
        let mut payload = JsonResponse::default();

        res.insert_header("Content-Type", "application/json");

        match res.error() {
            None => {
                let json = res.take_body().into_string().await?;
                match RawValue::from_string(json) {
                    Err(err) => {
                        payload.error_code = Some(500);
                        payload.description =
                            Some(format!("JsonResponseMiddleware: {}", err.to_string()));
                    }
                    Ok(result) => {
                        payload.ok = true;
                        payload.result = Some(result);
                    }
                }
            }
            Some(err) => {
                payload.error_code = Some(res.status().into());
                payload.description = Some(err.to_string());
            }
        }

        res.set_body(Body::from_json(&payload)?);

        Ok(res)
    }
}
