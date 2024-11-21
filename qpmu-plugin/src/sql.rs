use std::sync::OnceLock;

use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

static POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Initialises the sqlite connection and sets the [`POOL`] static.
///
/// This must be called before [`self::pool`] is run, or else it will panic.
pub(crate) async fn init(url: &str) -> Result<()> {
    if Sqlite::database_exists(url).await.unwrap_or(false) {
        eprintln!("creating database {url}");
        Sqlite::create_database(url).await?;
    }

    let init_pool = dbg!(SqlitePool::connect(dbg!(url)).await)?;
    POOL.get_or_init(|| init_pool);

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS activations (
            id INTEGER PRIMARY KEY NOT NULL,
            title TEXT NOT NULL UNIQUE,
            frequency INTEGER NOT NULL,
            last_use DATETIME NOT NULL
        );
        ",
    )
    .execute(pool())
    .await?;

    Ok(())
}

/// Gets access to the sqlite pool.
pub fn pool() -> &'static SqlitePool {
    POOL.get()
        .expect("init must have called first to initialise the pool")
}

// other helper stuff //

pub(crate) async fn increment_frequency_table(title: &str) -> Result<()> {
    sqlx::query(
        "
        INSERT INTO activations (title, frequency, last_use)
        VALUES (?, 1, ?)
        ON CONFLICT (title) DO UPDATE SET
            frequency = frequency + 1,
            last_use = excluded.last_use
        ",
    )
    .bind(title)
    .bind(time::OffsetDateTime::now_utc())
    .execute(pool())
    .await?;
    Ok(())
}
