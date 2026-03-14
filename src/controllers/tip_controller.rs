use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::tip::{RecordTipRequest, Tip};

pub async fn record_tip(pool: &PgPool, req: RecordTipRequest) -> Result<Tip> {
    let tip = sqlx::query_as::<_, Tip>(
        r#"
        INSERT INTO tips (id, creator_username, amount, transaction_hash, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING id, creator_username, amount, transaction_hash, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&req.username)
    .bind(&req.amount)
    .bind(&req.transaction_hash)
    .fetch_one(pool)
    .await?;

    Ok(tip)
}

pub async fn get_tips_for_creator(pool: &PgPool, username: &str) -> Result<Vec<Tip>> {
    let tips = sqlx::query_as::<_, Tip>(
        r#"
        SELECT id, creator_username, amount, transaction_hash, created_at
        FROM tips
        WHERE creator_username = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(username)
    .fetch_all(pool)
    .await?;

    Ok(tips)
}
