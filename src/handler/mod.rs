mod middleware;

use crate::logic;
use crate::service::Context;
use anyhow::Result;
use async_std::sync::Arc;
use http_types::headers::HeaderValue;
use middleware::{JsonResponseMiddleware, JwtAuthMiddleware};
use tide::security::{CorsMiddleware, Origin};

pub fn register_handlers(app: &mut tide::Server<Arc<Context>>) -> Result<()> {
    app.with(JsonResponseMiddleware::new());
    app.with(
        CorsMiddleware::new()
            .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
            .allow_headers(
                "Authorization, Content-Type"
                    .parse::<HeaderValue>()
                    .unwrap(),
            )
            .allow_origin(Origin::from("*"))
            .allow_credentials(false),
    );

    let access_secret = &app.state().conf.server.access_secret;
    let jwt_middleware = JwtAuthMiddleware::new(access_secret)?;

    // Authentication required
    app.at("/api/get_me")
        .with(jwt_middleware)
        .get(logic::get_me);

    // No authentication required
    app.at("/api/auth").post(logic::auth);
    app.at("/api/push/:project_id")
        .get(logic::push_message)
        .post(logic::push_message);

    Ok(())
}
