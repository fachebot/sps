mod middleware;
use crate::logic;
use crate::service::Context;
use anyhow::Result;

pub fn register_handlers(app: &mut tide::Server<Context>) -> Result<()> {
    app.with(middleware::JsonResponseMiddleware {});

    app.at("/api/auth").post(logic::auth);

    Ok(())
}
