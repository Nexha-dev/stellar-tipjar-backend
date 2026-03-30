use crate::errors::app_error::AppError;
use crate::indexer::cursor::CursorManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarEvent {
    pub id: String,
    pub event_type: String,
    pub transaction_hash: String,
    pub paging_token: String,
}

pub struct BlockchainListener {
    horizon_url: String,
    contract_id: String,
    cursor: Arc<RwLock<String>>,
    cursor_manager: Arc<CursorManager>,
}

impl BlockchainListener {
    pub fn new(
        horizon_url: String,
        contract_id: String,
        cursor_manager: Arc<CursorManager>,
    ) -> Self {
        Self {
            horizon_url,
            contract_id,
            cursor: Arc::new(RwLock::new("0".to_string())),
            cursor_manager,
        }
    }

    pub async fn start_listening(&self) -> Result<(), AppError> {
        if let Ok(Some(saved_cursor)) = self.cursor_manager.get_cursor().await {
            *self.cursor.write().await = saved_cursor;
        }

        tracing::info!(
            "Starting blockchain listener for contract: {}",
            self.contract_id
        );

        loop {
            let cursor = self.cursor.read().await.clone();
            tracing::debug!("Listening from cursor: {}", cursor);
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    pub async fn update_cursor(&self, paging_token: &str) -> Result<(), AppError> {
        *self.cursor.write().await = paging_token.to_string();
        self.cursor_manager.save_cursor(paging_token).await
    }
}
