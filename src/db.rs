use oxidizer::*;
use chrono::{DateTime, Utc};

const DB_CONNECTION: &'static str = "postgres://postgres:docker@172.17.0.2:5432/postgres";

#[derive(Entity)]
pub struct Logging {
    #[primary_key]
    pub id: i32,

    pub event: String,

    pub timestamp: DateTime<Utc>,
}

impl Logging {
    pub fn for_event(event: &str) -> Self {
        Logging {
            id: i32::default(),
            event: event.to_string(),
            timestamp: Utc::now(),
        }
    }
}

pub async fn connect_db() -> Result<DB, String> {
    let db = DB::connect(DB_CONNECTION, 50, None).await.map_err(|_e| "DatabaseError".to_string())?;

    let logging_migration = Logging::create_migration().map_err(|_e| "CannotCreateMigrations".to_string())?;
    db.migrate_tables(&[logging_migration]).await.map_err(|_e| "MigrationError".to_string())?;

    Ok(db)
}

#[cfg(test)]
mod db_tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_logging_entity() {
        let db = connect_db().await.unwrap();

        let mut entity = Logging::for_event("testEvent");
        let creating = entity.save(&db).await.unwrap();
        assert_eq!(creating, true);
    }
}