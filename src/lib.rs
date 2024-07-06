mod model;
mod adapter;
mod driver;

use worker::*;
use crate::driver::routes::{lookup, webhook};

#[event(fetch, respond_with_errors)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .post_async("/lookup", lookup)
        .post_async("/webhook", webhook)
        .run(req, env)
        .await
}

