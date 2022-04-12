use serde::Serialize;
use serde_json::value::RawValue;
use tide::Body;

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

#[derive(Default, Clone)]
pub struct JsonResponseMiddleware {}

#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> tide::Middleware<State> for JsonResponseMiddleware {
    async fn handle(&self, req: tide::Request<State>, next: tide::Next<'_, State>) -> tide::Result {
        let mut res = next.run(req).await;
        let mut payload = JsonResponse::default();

        if let Some(err) = res.error() {
            payload.error_code = Some(res.status().into());
            payload.description = Some(err.to_string());
        } else {
            payload.ok = true;
            let json = res.take_body().into_string().await?;
            payload.result = Some(RawValue::from_string(json)?);
        }

        res.set_body(Body::from_json(&payload)?);

        Ok(res)
    }
}
