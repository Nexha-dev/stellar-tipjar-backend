use crate::errors::app_error::AppError;
use crate::service_mesh::circuit_breaker::CircuitBreaker;
use crate::service_mesh::discovery::ServiceRegistry;
use crate::service_mesh::load_balancer::{LoadBalancer, LoadBalancingStrategy};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct ApiGateway {
    registry: Arc<ServiceRegistry>,
    circuit_breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    load_balancer: Arc<LoadBalancer>,
}

impl ApiGateway {
    pub fn new(registry: Arc<ServiceRegistry>) -> Self {
        Self {
            registry,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            load_balancer: Arc::new(LoadBalancer::new(LoadBalancingStrategy::RoundRobin)),
        }
    }

    pub async fn route_request(&self, service_name: &str) -> Result<String, AppError> {
        let instances = self.registry.discover_all(service_name).await;
        if instances.is_empty() {
            return Err(AppError::not_found(format!(
                "Service {} not found",
                service_name
            )));
        }

        let breaker = {
            let mut breakers = self.circuit_breakers.write().await;
            breakers
                .entry(service_name.to_string())
                .or_insert_with(|| {
                    Arc::new(CircuitBreaker::new(5, 2, Duration::from_secs(30)))
                })
                .clone()
        };

        breaker.check_half_open().await;
        if breaker.is_open().await {
            return Err(AppError::service_unavailable(
                "Circuit breaker is open".to_string(),
            ));
        }

        let instance = self
            .load_balancer
            .select(&instances)
            .ok_or_else(|| AppError::not_found("No available instances"))?;

        breaker.record_success().await;

        Ok(format!("http://{}:{}", instance.host, instance.port))
    }
}
