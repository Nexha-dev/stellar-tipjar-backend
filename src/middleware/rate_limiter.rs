use tower_governor::{GovernorConfigBuilder, GovernorLayer};

pub fn build_rate_limiter() -> GovernorLayer {
    let per_second = std::env::var("RATE_LIMIT_PER_SECOND")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);
        
    let burst_size = std::env::var("RATE_LIMIT_BURST_SIZE")
        .unwrap_or_else(|_| "20".to_string())
        .parse()
        .unwrap_or(20);

    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(burst_size)
            .finish()
            .unwrap(),
    );

    GovernorLayer { config: Box::leak(governor_conf) }
}
