use tracing::info;

pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("owlenbar=debug")),
        )
        .init();
}

pub fn milestone(name: &str) {
    info!(milestone = name, "startup");
}

pub fn placement_decision(widget: &str, reason: &str, output: Option<&str>, fallback: bool) {
    info!(widget, reason, output, fallback, "placement");
}
