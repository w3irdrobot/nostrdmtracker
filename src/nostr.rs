use crate::db::DbPool;
use anyhow::{anyhow, Result};
use base64::prelude::*;
use futures::{future, stream::StreamExt};
use log::{debug, info};
use nostr_sdk::prelude::*;
use tokio_stream::wrappers::BroadcastStream;

pub struct Nostr {
    db: DbPool,
    nostr_client: Client,
}

impl Nostr {
    pub async fn new(db: DbPool) -> Result<Self> {
        let keys = Keys::generate();
        let nostr_client = Client::new(&keys);

        for url in vec![
            "wss://nos.lol",
            "wss://no.str.cr",
            "wss://nostr.bitcoiner.social",
            "wss://relay.snort.social",
            "wss://relay.damus.io",
        ] {
            info!("connecting to relay {}", url);
            nostr_client
                .add_relay(url, None)
                .await
                .expect(&format!("{} connects", url));
        }
        nostr_client.connect().await;

        Ok(Nostr { db, nostr_client })
    }

    pub async fn hydrate_messages(&self) -> Result<()> {
        let filter = Filter::new().kind(Kind::Metadata);
        self.nostr_client.subscribe(vec![filter]).await;

        Ok(BroadcastStream::new(self.nostr_client.notifications())
            .filter_map(|x| future::ready(x.ok()))
            .filter_map(|x| async {
                match x {
                    RelayPoolNotification::Event(_, e) => Some(e),
                    _ => None,
                }
            })
            .for_each_concurrent(15, |e| async {
                debug!("handling event {:?}", e);
                match e.kind {
                    Kind::Metadata => self.save_metadata(&self.db, e).await.unwrap_or(()),
                    Kind::EncryptedDirectMessage => self.save_dm(&self.db, e).await.unwrap_or(()),
                    _ => (),
                }
            })
            .await)
    }

    async fn save_metadata(&self, pool: &DbPool, event: Event) -> Result<()> {
        let pubkey = event.pubkey.to_string();
        let metadata = Metadata::from_json(event.content)?;
        let pubkey_id = self.get_or_create_pubkey(&pool, &pubkey).await?;

        if metadata.nip05.is_none() {
            return Ok(());
        }

        let nip05 = metadata.nip05.unwrap();
        let address_id = match pool.address_id(&nip05).await {
            Ok(id) => id,
            Err(_) => pool.insert_address(&nip05).await?,
        };

        if !pool.pubkey_has_address(pubkey_id, address_id).await {
            pool.connect_pubkey_address(pubkey_id, address_id).await?;
        }

        Ok(())
    }

    async fn save_dm(&self, pool: &DbPool, event: Event) -> Result<()> {
        let from_pubkey = event.pubkey.to_string();
        let from_pubkey_id = self.get_or_create_pubkey(&pool, &from_pubkey).await?;

        let to_pubkey = event
            .tags
            .into_iter()
            .find_map(|t| match t {
                Tag::PubKey(pubkey, _) => Some(pubkey),
                _ => None,
            })
            .ok_or(anyhow!("no to pubkey"))?
            .to_string();
        let to_pubkey_id = self.get_or_create_pubkey(&pool, &to_pubkey).await?;

        let content_len = BASE64_STANDARD.decode(event.content)?.len();

        pool.save_dm(to_pubkey_id, from_pubkey_id, content_len.try_into()?)
            .await?;

        Ok(())
    }

    async fn get_or_create_pubkey(&self, pool: &DbPool, pubkey: &str) -> Result<i32> {
        let id = match pool.pubkey_id(&pubkey).await {
            Ok(id) => id,
            Err(_) => pool.insert_pubkey(&pubkey).await?,
        };

        Ok(id)
    }
}
