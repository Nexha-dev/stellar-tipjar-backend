use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Creator {
    pub id: Uuid,
    pub username: String,
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCreatorRequest {
    pub username: String,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct CreatorResponse {
    pub id: Uuid,
    pub username: String,
    pub wallet_address: String,
    pub created_at: DateTime<Utc>,
}

impl From<Creator> for CreatorResponse {
    fn from(c: Creator) -> Self {
        Self {
            id: c.id,
            username: c.username,
            wallet_address: c.wallet_address,
            created_at: c.created_at,
        }
    }
}
