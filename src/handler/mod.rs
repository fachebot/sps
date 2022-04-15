mod middleware;

use crate::logic;
use crate::service::Context;
use anyhow::Result;
use middleware::{JsonResponseMiddleware, JwtAuthMiddleware};

pub fn register_handlers(app: &mut tide::Server<Context>) -> Result<()> {
    app.with(JsonResponseMiddleware::new());
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
