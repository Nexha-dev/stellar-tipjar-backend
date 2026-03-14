use sqlx::PgPool;

use crate::services::stellar_service::StellarService;

pub struct AppState {
    pub db: PgPool,
    pub stellar: StellarService,
}
