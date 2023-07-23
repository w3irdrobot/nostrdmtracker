use nostr::Nostr;
use nostr_sdk::prelude::*;
use std::env;

mod db;
mod nostr;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    env_logger::init();

    let pool = db::new_db_pool(&env::var("DATABASE_URL")?).await?;
    let nostr = Nostr::new(pool.clone()).await?;

    let hydrate = nostr.hydrate_messages();

    hydrate.await?;

    Ok(())
}
