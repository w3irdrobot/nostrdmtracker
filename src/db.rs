use anyhow::Result;
use log::{debug, info};
use sqlx::{migrate::Migrator, Connection, PgPool};
use std::sync::Arc;

static MIGRATOR: Migrator = sqlx::migrate!();

pub type DbPool = Arc<Database>;

pub async fn new_db_pool(url: &str) -> Result<DbPool> {
    let pool = PgPool::connect(url).await?;
    let db = Arc::new(Database::new(pool).await?);

    Ok(db)
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(pool: PgPool) -> Result<Self> {
        pool.acquire().await?.ping().await?;
        MIGRATOR.run(&pool).await?;

        info!("connected to database");
        Ok(Self { pool })
    }

    pub async fn pubkey_id(&self, pubkey: &str) -> Result<i32> {
        debug!("getting pubkey {}", pubkey);
        let req = sqlx::query!(r#"SELECT id FROM pubkeys WHERE pubkey = $1"#, pubkey)
            .fetch_one(&self.pool)
            .await?;

        Ok(req.id)
    }

    pub async fn insert_pubkey(&self, pubkey: &str) -> Result<i32> {
        debug!("inserting pubkey {}", pubkey);
        let req = sqlx::query!(
            r#"INSERT INTO pubkeys (pubkey) VALUES ($1) RETURNING id"#,
            pubkey
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(req.id)
    }

    pub async fn address_id(&self, address: &str) -> Result<i32> {
        debug!("getting address {}", address);
        let req = sqlx::query!(r#"SELECT id FROM addresses WHERE address = $1"#, address)
            .fetch_one(&self.pool)
            .await?;

        Ok(req.id)
    }

    pub async fn insert_address(&self, address: &str) -> Result<i32> {
        debug!("inserting address {}", address);
        let req = sqlx::query!(
            r#"INSERT INTO addresses (address) VALUES ($1) RETURNING id"#,
            address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(req.id)
    }

    pub async fn pubkey_has_address(&self, pubkey_id: i32, address_id: i32) -> bool {
        debug!(
            "checking if pubkey {} has address {}",
            pubkey_id, address_id
        );
        match sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!" FROM addresses_pubkeys WHERE address_id = $1 AND pubkey_id = $2"#,
            address_id,
            pubkey_id
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(0) => false,
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn connect_pubkey_address(&self, pubkey_id: i32, address_id: i32) -> Result<()> {
        debug!(
            "connecting pubkey {} with address {}",
            pubkey_id, address_id
        );
        sqlx::query!(
            r#"INSERT INTO addresses_pubkeys (address_id, pubkey_id) VALUES ($1, $2)"#,
            address_id,
            pubkey_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_dm(&self, to_id: i32, from_id: i32, size: i64) -> Result<()> {
        debug!(
            "saving dm; to_id: {}, from_id: {}, size: {}",
            to_id, from_id, size
        );
        sqlx::query!(
            r#"INSERT INTO dms (to_pubkey_id, from_pubkey_id, size) VALUES ($1, $2, $3)"#,
            to_id,
            from_id,
            size
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
