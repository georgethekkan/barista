use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        // Parse log filters from the RUST_LOG environment variable (defaults to "info")
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        // Set this registry as the global default for the entire application
        .init();
}
