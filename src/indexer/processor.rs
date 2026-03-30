use crate::errors::app_error::AppError;
use crate::indexer::listener::StellarEvent;
use sqlx::PgPool;

pub struct EventProcessor {
    pool: PgPool,
}

impl EventProcessor {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn process_tip_event(&self, event: &StellarEvent) -> Result<(), AppError> {
        if self.is_processed(&event.id).await? {
            return Ok(());
        }

        tracing::info!("Processing tip event: {}", event.id);
        self.mark_processed(&event.id).await?;

        Ok(())
    }

    pub async fn process_withdraw_event(&self, event: &StellarEvent) -> Result<(), AppError> {
        if self.is_processed(&event.id).await? {
            return Ok(());
        }

        tracing::info!("Processing withdraw event: {}", event.id);
        self.mark_processed(&event.id).await?;

        Ok(())
    }

    async fn is_processed(&self, event_id: &str) -> Result<bool, AppError> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM indexed_events WHERE event_id = $1"
        )
        .bind(event_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(result.0 > 0)
    }

    async fn mark_processed(&self, event_id: &str) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO indexed_events (event_id, processed_at) VALUES ($1, NOW())
             ON CONFLICT (event_id) DO NOTHING"
        )
        .bind(event_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(())
    }
}
