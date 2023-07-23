use axum::{routing::get, Router, Server};
use futures::future;
use http::hello;
use log::info;
use nostr::Nostr;
use nostr_sdk::prelude::*;
use std::env;
use std::net::SocketAddr;

mod db;
mod http;
mod nostr;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let pool = db::new_db_pool(&env::var("DATABASE_URL")?).await?;
    let nostr = Nostr::new(pool.clone()).await?;

    let hydrate = nostr.hydrate_messages();

    let app = Router::new().route("/", get(hello)).with_state(pool);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(app.into_make_service());
    info!("running server on {}", addr);

    future::join(hydrate, server).await;

    Ok(())
}
