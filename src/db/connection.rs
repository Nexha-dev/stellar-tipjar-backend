use redis::aio::ConnectionManager;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::services::stellar_service::StellarService;
use crate::ws::TipEvent;
use super::performance::PerformanceMonitor;

pub struct AppState {
    pub db: PgPool,
    pub stellar: StellarService,
    pub performance: Arc<PerformanceMonitor>,
    /// Broadcast channel for real-time tip notifications over WebSocket.
    pub broadcast_tx: broadcast::Sender<TipEvent>,
}
