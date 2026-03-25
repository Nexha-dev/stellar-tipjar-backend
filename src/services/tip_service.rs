use anyhow::Result;

use crate::controllers::tip_controller;
use crate::db::connection::AppState;
use crate::models::tip::{RecordTipRequest, Tip};

#[allow(dead_code)]
pub struct TipService;

#[allow(dead_code)]
impl TipService {
    pub fn new() -> Self {
        Self
    }

    /// Record a new tip after optionally verifying the transaction on-chain.
    pub async fn record_tip(&self, state: &AppState, req: RecordTipRequest) -> Result<Tip> {
        tip_controller::record_tip(state, req).await
    }

    /// Retrieve all tips for a given creator username.
    pub async fn get_tips_for_creator(&self, state: &AppState, username: &str) -> Result<Vec<Tip>> {
        tip_controller::get_tips_for_creator(state, username).await
    }

    /// Process multiple tips in a single atomic database transaction.
    /// Uses SAVEPOINTs to provide error recovery: if one tip fails (e.g. duplicate hash), 
    /// it is rolled back without aborting the entire bulk operation.
    pub async fn bulk_record_tips(&self, state: &AppState, requests: Vec<RecordTipRequest>) -> Result<Vec<Tip>> {
        let mut tx = crate::db::transaction::begin_transaction(&state.db).await?;
        let mut results = Vec::new();

        for (i, req) in requests.into_iter().enumerate() {
            let sp = format!("tip_record_{}", i);
            crate::db::transaction::create_savepoint(&mut tx, &sp).await?;
            
            match tip_controller::record_tip_in_tx(&mut tx, &req).await {
                Ok(tip) => {
                    results.push(tip);
                    crate::db::transaction::release_savepoint(&mut tx, &sp).await?;
                }
                Err(e) => {
                    tracing::error!("Bulk tip record failed for index {}: {}. Rolling back savepoint.", i, e);
                    crate::db::transaction::rollback_savepoint(&mut tx, &sp).await?;
                }
            }
        }

        tx.commit().await?;
        Ok(results)
    }
}
