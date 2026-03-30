use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    pub async fn is_open(&self) -> bool {
        let state = self.state.lock().await;
        *state == CircuitState::Open
    }

    pub async fn record_success(&self) {
        let mut state = self.state.lock().await;

        match *state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                if count >= self.success_threshold {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            _ => {}
        }
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.lock().await;
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

        if count >= self.failure_threshold {
            *state = CircuitState::Open;
            *self.last_failure_time.lock().await = Some(Instant::now());
        }
    }

    pub async fn check_half_open(&self) {
        let mut state = self.state.lock().await;

        if *state == CircuitState::Open {
            if let Some(last_failure) = *self.last_failure_time.lock().await {
                if last_failure.elapsed() >= self.timeout {
                    *state = CircuitState::HalfOpen;
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }
}
