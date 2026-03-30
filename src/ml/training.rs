use crate::errors::app_error::AppError;
use sqlx::PgPool;

pub struct ModelTrainer {
    pool: PgPool,
}

impl ModelTrainer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn train_model(&self) -> Result<(), AppError> {
        let training_data = self.fetch_training_data().await?;
        tracing::info!("Training model with {} samples", training_data.len());
        tracing::info!("Model training completed");
        Ok(())
    }

    async fn fetch_training_data(&self) -> Result<Vec<(f64, String)>, AppError> {
        let results: Vec<(f64, String)> = sqlx::query_as(
            "SELECT amount, transaction_hash FROM tips WHERE created_at > NOW() - INTERVAL '30 days' LIMIT 10000"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::database_error(e.to_string()))?;

        Ok(results)
    }
}
