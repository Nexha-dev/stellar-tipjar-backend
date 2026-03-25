use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;

use crate::controllers::creator_controller;
use crate::controllers::tip_controller;
use crate::db::connection::AppState;
use crate::models::creator::{CreateCreatorRequest, CreatorResponse};
use crate::models::pagination::PaginationParams;
use crate::models::tip::TipResponse;
use crate::validation::ValidatedJson;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/creators", post(create_creator))
        .route("/creators/:username", get(get_creator))
        .route("/creators/:username/tips", get(get_creator_tips))
}

/// Create a new creator profile
#[utoipa::path(
    post,
    path = "/creators",
    tag = "creators",
    request_body = CreateCreatorRequest,
    responses(
        (status = 201, description = "Creator created successfully", body = CreatorResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_creator(
    State(state): State<Arc<AppState>>,
    ValidatedJson(body): ValidatedJson<CreateCreatorRequest>,
) -> impl IntoResponse {
    match creator_controller::create_creator(&state, body).await {
        Ok(creator) => {
            let response: CreatorResponse = creator.into();
            (StatusCode::CREATED, Json(serde_json::json!(response))).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create creator: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to create creator" })),
            )
                .into_response()
        }
    }
}

/// Get a creator by username
#[utoipa::path(
    get,
    path = "/creators/{username}",
    tag = "creators",
    params(
        ("username" = String, Path, description = "Creator's unique username")
    ),
    responses(
        (status = 200, description = "Creator found", body = CreatorResponse),
        (status = 404, description = "Creator not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_creator(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    match creator_controller::get_creator_by_username(&state, &username).await {
        Ok(Some(creator)) => {
            let response: CreatorResponse = creator.into();
            (StatusCode::OK, Json(serde_json::json!(response))).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Creator not found" })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get creator: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to get creator" })),
            )
                .into_response()
        }
    }
}

/// List tips for a creator with pagination
#[utoipa::path(
    get,
    path = "/creators/{username}/tips",
    tag = "creators",
    params(
        ("username" = String, Path, description = "Creator's unique username"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of tips"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_creator_tips(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    match tip_controller::get_tips_paginated(&state, &username, params).await {
        Ok(page) => {
            let response = serde_json::json!({
                "data": page.data.into_iter().map(TipResponse::from).collect::<Vec<_>>(),
                "total": page.total,
                "page": page.page,
                "limit": page.limit,
                "total_pages": page.total_pages,
                "has_next": page.has_next,
                "has_prev": page.has_prev,
            });
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get tips: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to get tips" })),
            )
                .into_response()
        }
    }
}
