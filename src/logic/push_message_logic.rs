use crate::model;
use crate::service::Context;
use crate::types::{PushMessageRequest, PushMessageResponse};
use anyhow::anyhow;
use async_std::sync::Arc;
use redis::AsyncCommands;
use std::ops::DerefMut;
use tide::{Body, Request, Response};

pub async fn push_message(mut req: Request<Arc<Context>>) -> tide::Result {
    let data = match req.method() {
        http_types::Method::Get => req.query::<PushMessageRequest>()?,
        http_types::Method::Post => req.body_json::<PushMessageRequest>().await?,
        _ => return Err(tide::Error::new(400, anyhow!("Bad request"))),
    };
    let project_id = req.param("project_id").unwrap();

    // Save message & task
    let user = req
        .state()
        .user_model
        .find_one_by_project_id(project_id)
        .await?;
    let transports = req
        .state()
        .transport_model
        .find_all_by_user_id(user.id)
        .await?;

    let ids = transports.into_iter().map(|e| e.id).collect::<Vec<i64>>();
    let task_ids =
        model::insert_message(&req.state().pool, user.id, &data.title, &data.content, &ids).await?;

    // Adding to redis task queue
    let ts = chrono::Utc::now().timestamp();
    let items = task_ids
        .into_iter()
        .map(|task_id| (ts, task_id))
        .collect::<Vec<(i64, i64)>>();
    let mut guard = req.state().redis_connection.lock().await;
    guard
        .deref_mut()
        .zadd_multiple(req.state().conf.redis.queue_name.as_str(), items.as_slice())
        .await?;

    let res = PushMessageResponse {
        status: "queued".to_string(),
    };

    Ok(Response::builder(200).body(Body::from_json(&res)?).build())
}
