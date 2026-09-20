use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init(verbose: bool, quiet: bool) {
    let level = if quiet {
        "off"
    } else if verbose {
        "debug"
    } else {
        "info"
    };
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(fmt::layer())
        // Parse log filters from the RUST_LOG environment variable (defaults to "info")
        .with(env_filter)
        // Set this registry as the global default for the entire application
        .init();
}
