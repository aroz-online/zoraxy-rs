pub mod dynamic_router;
pub mod embed_webserver;
pub mod prelude;
pub mod spec;
pub mod static_router;
pub mod types;

pub use prelude::*;

pub fn init_tracing_subscriber(debug: bool) {
    let level = if debug { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive(level.parse().unwrap()),
        )
        .without_time()
        .with_ansi(false)
        .init();
}
