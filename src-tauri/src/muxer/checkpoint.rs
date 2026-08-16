use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SessionCheckpoint {
    pub session_id: String,
    pub timestamp_ms: i64,
    pub total_frames: i64,
    pub file_size_bytes: i64,
    pub storage_status: String,
}

pub async fn init_db(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true),
        )
        .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS checkpoints (
            session_id TEXT PRIMARY KEY,
            timestamp_ms INTEGER NOT NULL,
            total_frames INTEGER NOT NULL,
            file_size_bytes INTEGER NOT NULL,
            storage_status TEXT NOT NULL
        )"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn checkpoint_session(db: &SqlitePool, session: &SessionCheckpoint) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO checkpoints (session_id, timestamp_ms, total_frames, file_size_bytes, storage_status)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(session_id) DO UPDATE SET
            timestamp_ms=excluded.timestamp_ms,
            total_frames=excluded.total_frames,
            file_size_bytes=excluded.file_size_bytes,
            storage_status=excluded.storage_status"
    )
    .bind(&session.session_id)
    .bind(session.timestamp_ms)
    .bind(session.total_frames)
    .bind(session.file_size_bytes)
    .bind(&session.storage_status)
    .execute(db)
    .await?;

    Ok(())
}
