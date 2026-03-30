use crate::service_mesh::discovery::ServiceInstance;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastConnections,
    Random,
}

pub struct LoadBalancer {
    strategy: LoadBalancingStrategy,
    counter: Arc<AtomicUsize>,
}

impl LoadBalancer {
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn select(&self, instances: &[ServiceInstance]) -> Option<ServiceInstance> {
        if instances.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                let idx = self.counter.fetch_add(1, Ordering::SeqCst) % instances.len();
                Some(instances[idx].clone())
            }
            LoadBalancingStrategy::LeastConnections => instances.first().cloned(),
            LoadBalancingStrategy::Random => {
                let idx = (self.counter.fetch_add(1, Ordering::SeqCst) * 7919) % instances.len();
                Some(instances[idx].clone())
            }
        }
    }
}
