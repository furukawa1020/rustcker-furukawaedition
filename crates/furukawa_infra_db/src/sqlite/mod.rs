use async_trait::async_trait;
use furukawa_common::Result;
use furukawa_domain::container::{store::ContainerStore, Container, Created, Config};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Row};
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
        // We drop table for now to ensure schema match during dev phase refactor
        // In real prod, we'd use migrations
        sqlx::query("DROP TABLE IF EXISTS containers").execute(&pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS containers (
                id TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                config JSON NOT NULL,
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
        let config_json = serde_json::to_string(container.config())
            .map_err(|e| furukawa_common::diagnostic::Error::new(SerializationError(e)))?;

        sqlx::query("INSERT INTO containers (id, state, config) VALUES (?, ?, ?)")
            .bind(container.id())
            .bind("created")
            .bind(config_json)
            .execute(&self.pool)
            .await
            .map_err(|e| furukawa_common::diagnostic::Error::new(DbError(e)))?;
            
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Container<Created>>> {
        let rows = sqlx::query("SELECT id, config FROM containers WHERE state = 'created'")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| furukawa_common::diagnostic::Error::new(DbError(e)))?;

        let mut containers = Vec::new();
        for row in rows {
            let id: String = row.get("id");
            let config_str: String = row.get("config");
            let config: Config = serde_json::from_str(&config_str)
                 .map_err(|e| furukawa_common::diagnostic::Error::new(SerializationError(e)))?;
            
            containers.push(Container::new(id, config));
        }
            
        Ok(containers)
    }

    async fn get(&self, id: &str) -> Result<Option<Container<Created>>> {
        let row = sqlx::query("SELECT id, config FROM containers WHERE id = ? AND state = 'created'")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| furukawa_common::diagnostic::Error::new(DbError(e)))?;

        match row {
            Some(row) => {
                let id: String = row.get("id");
                let config_str: String = row.get("config");
                let config: Config = serde_json::from_str(&config_str)
                     .map_err(|e| furukawa_common::diagnostic::Error::new(SerializationError(e)))?;
                Ok(Some(Container::new(id, config)))
            },
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

#[derive(Debug, thiserror::Error)]
#[error("Serialization error: {0}")]
struct SerializationError(#[from] serde_json::Error);

impl furukawa_common::diagnostic::Diagnosable for SerializationError {
    fn code(&self) -> String {
        "DB_SERIALIZATION_ERROR".to_string()
    }
    fn suggestion(&self) -> Option<String> {
        Some("Check data integrity".to_string())
    }
}
