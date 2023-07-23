use anyhow::Result;
use log::{debug, info};
use sqlx::{migrate::Migrator, Connection, PgPool};

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn new_db_pool(url: &str) -> Result<Database> {
    let pool = PgPool::connect(url).await?;
    let db = Database::new(pool).await?;

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
        debug!(target: "db", pubkey = pubkey; "getting pubkey");
        let req = sqlx::query!(r#"SELECT id FROM pubkeys WHERE pubkey = $1"#, pubkey)
            .fetch_one(&self.pool)
            .await?;

        Ok(req.id)
    }

    pub async fn insert_pubkey(&self, pubkey: &str) -> Result<i32> {
        debug!(target: "db", pubkey = pubkey; "inserting pubkey");
        let req = sqlx::query!(
            r#"INSERT INTO pubkeys (pubkey) VALUES ($1) RETURNING id"#,
            pubkey
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(req.id)
    }

    pub async fn address_id(&self, address: &str) -> Result<i32> {
        debug!(target: "db", address = address; "getting address");
        let req = sqlx::query!(r#"SELECT id FROM addresses WHERE address = $1"#, address)
            .fetch_one(&self.pool)
            .await?;

        Ok(req.id)
    }

    pub async fn insert_address(&self, address: &str) -> Result<i32> {
        debug!(target: "db", address = address; "inserting address");
        let req = sqlx::query!(
            r#"INSERT INTO addresses (address) VALUES ($1) RETURNING id"#,
            address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(req.id)
    }

    pub async fn pubkey_has_address(&self, pubkey_id: i32, address_id: i32) -> bool {
        debug!(target: "db", pubkey_id = pubkey_id, address_id = address_id; "checking if pubkey has address");
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
        debug!(target: "db", pubkey_id = pubkey_id, address_id = address_id; "connecting pubkey with address");
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
        debug!(target: "db", to_id = to_id, from_id = from_id, size = size; "saving dm");
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
