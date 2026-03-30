use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub healthy: bool,
}

pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, service: ServiceInstance) {
        let mut services = self.services.write().await;
        services
            .entry(service.name.clone())
            .or_insert_with(Vec::new)
            .push(service);
    }

    pub async fn deregister(&self, service_name: &str, service_id: Uuid) {
        let mut services = self.services.write().await;
        if let Some(instances) = services.get_mut(service_name) {
            instances.retain(|s| s.id != service_id);
        }
    }

    pub async fn discover(&self, service_name: &str) -> Option<ServiceInstance> {
        let services = self.services.read().await;
        services
            .get(service_name)
            .and_then(|instances| instances.first().cloned())
    }

    pub async fn discover_all(&self, service_name: &str) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        services
            .get(service_name)
            .map(|instances| instances.clone())
            .unwrap_or_default()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
