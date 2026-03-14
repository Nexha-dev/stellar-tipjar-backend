use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tip {
    pub id: Uuid,
    pub creator_username: String,
    pub amount: String,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RecordTipRequest {
    pub username: String,
    pub amount: String,
    pub transaction_hash: String,
}

#[derive(Debug, Serialize)]
pub struct TipResponse {
    pub id: Uuid,
    pub creator_username: String,
    pub amount: String,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
}

impl From<Tip> for TipResponse {
    fn from(t: Tip) -> Self {
        Self {
            id: t.id,
            creator_username: t.creator_username,
            amount: t.amount,
            transaction_hash: t.transaction_hash,
            created_at: t.created_at,
        }
    }
}
