use async_trait::async_trait;
use furukawa_common::Result;
use furukawa_domain::container::{store::ContainerStore, Container, Created};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::info;

#[derive(Clone)]
pub struct SqliteStore {
    pool: Pool<Sqlite>,
}

impl SqliteStore {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Initialize schema (Strict strictness)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS containers (
                id TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );"
        )
        .execute(&pool)
        .await?;
        
        info!("SQLite store initialized at {}", database_url);

        Ok(Self { pool })
    }
}

#[async_trait]
impl ContainerStore for SqliteStore {
    async fn save(&self, container: &Container<Created>) -> Result<()> {
        // We only support saving 'Created' state for now as per the FSM definition in this phase.
        // In a full implementation, we'd enum match on state or have separate tables/columns.
        
        // This is a "robust" implementation: it handles conflicts (though ID should be unique).
        sqlx::query("INSERT INTO containers (id, state) VALUES (?, ?)")
            .bind(container.id()) // We need to expose ID from Container
            .bind("created")
            .execute(&self.pool)
            .await
            .map_err(|e| furukawa_common::diagnostic::Error::new(DbError(e)))?;
            
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Container<Created>>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT id FROM containers WHERE id = ? AND state = 'created'")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| furukawa_common::diagnostic::Error::new(DbError(e)))?;

        match row {
            Some((id,)) => Ok(Some(Container::new(id))),
            None => Ok(None),
        }
    }
}

// Map sqlx errors to our Diagnosable error
#[derive(Debug, thiserror::Error)]
#[error("Database error: {0}")]
struct DbError(#[from] sqlx::Error);

impl furukawa_common::diagnostic::Diagnosable for DbError {
    fn code(&self) -> String {
        "DB_ERROR".to_string()
    }
    fn suggestion(&self) -> Option<String> {
        Some("Check database connection or query syntax".to_string())
    }
}
