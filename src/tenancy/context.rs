use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub tenant_name: String,
    pub config: TenantConfig,
    pub quotas: ResourceQuotas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub max_creators: i32,
    pub max_tips_per_day: i32,
    pub features: HashSet<String>,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuotas {
    pub max_creators: i32,
    pub max_tips_per_day: i32,
    pub max_storage_gb: i32,
}

impl ResourceQuotas {
    pub fn get_limit(&self, resource: &str) -> i32 {
        match resource {
            "creators" => self.max_creators,
            "tips_per_day" => self.max_tips_per_day,
            "storage" => self.max_storage_gb,
            _ => 0,
        }
    }
}
