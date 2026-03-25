use anyhow::Result;
use std::time::Instant;
use uuid::Uuid;

use crate::db::connection::AppState;
use crate::db::query_logger::QueryLogger;
use crate::models::pagination::{PaginatedResponse, PaginationParams};
use crate::models::tip::{RecordTipRequest, Tip};

#[tracing::instrument(skip(state), fields(username = %req.username, amount = %req.amount))]
pub async fn record_tip(state: &AppState, req: RecordTipRequest) -> Result<Tip> {
    let query = r#"
        INSERT INTO tips (id, creator_username, amount, transaction_hash, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING id, creator_username, amount, transaction_hash, created_at
        "#;

    let start = Instant::now();
    let tip = sqlx::query_as::<_, Tip>(query)
        .bind(Uuid::new_v4())
        .bind(&req.username)
        .bind(&req.amount)
        .bind(&req.transaction_hash)
        .fetch_one(&state.db)
        .await?;
    let duration = start.elapsed();

    QueryLogger::log_query(query, duration);
    state.performance.track_query(query, duration);
    tracing::info!(duration_ms = duration.as_millis(), tip_id = %tip.id, "Tip recorded");

    Ok(tip)
}

/// Fetch all tips for a creator without pagination (kept for internal use).
#[tracing::instrument(skip(state), fields(username = %username))]
pub async fn get_tips_for_creator(state: &AppState, username: &str) -> Result<Vec<Tip>> {
    let query = r#"
        SELECT id, creator_username, amount, transaction_hash, created_at
        FROM tips
        WHERE creator_username = $1
        ORDER BY created_at DESC
        "#;

    let start = Instant::now();
    let tips = sqlx::query_as::<_, Tip>(query)
        .bind(username)
        .fetch_all(&state.db)
        .await?;
    let duration = start.elapsed();

    QueryLogger::log_query(query, duration);
    state.performance.track_query(query, duration);
    tracing::debug!(duration_ms = duration.as_millis(), count = tips.len(), "Tips fetched");

    Ok(tips)
}

/// Fetch paginated tips for a creator with total count.
#[tracing::instrument(skip(state), fields(username = %username, page = params.page, limit = params.limit))]
pub async fn get_tips_paginated(
    state: &AppState,
    username: &str,
    params: PaginationParams,
) -> Result<PaginatedResponse<Tip>> {
    let params = params.validated();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tips WHERE creator_username = $1",
    )
    .bind(username)
    .fetch_one(&state.db)
    .await?;

    let start = Instant::now();
    let tips = sqlx::query_as::<_, Tip>(
        r#"
        SELECT id, creator_username, amount, transaction_hash, created_at
        FROM tips
        WHERE creator_username = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(username)
    .bind(params.limit)
    .bind(params.offset())
    .fetch_all(&state.db)
    .await?;
    let duration = start.elapsed();

    tracing::debug!(
        duration_ms = duration.as_millis(),
        count = tips.len(),
        total,
        "Paginated tips fetched"
    );

    Ok(PaginatedResponse::new(tips, total, &params))
}
