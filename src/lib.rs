pub mod dynamic_router;
pub mod embed_webserver;
pub mod prelude;
pub mod spec;
pub mod static_router;
mod termination;
pub mod types;

pub use prelude::*;

/// Initializes the tracing subscriber for logging.
///
/// Disables timestamps and ANSI colors for better compatibility with Zoraxy's logging system.
/// # Arguments
/// * `debug` - A boolean indicating whether to set the logging level to debug or info
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

/// Starts the axum web server for the plugin, handling termination signals from Zoraxy if the ui_path is provided.
/// # Arguments
/// * `app` - The axum Router instance to serve.
/// * `state` - The shared state to be used by the axum application.
/// * `addr` - The socket address to bind the server to.
/// # Errors
/// * Returns an error if the server fails to start or encounters an error during execution.
/// # Returns
/// returns when the server is terminated.
pub async fn start_plugin<S, P>(
    app: axum::Router<S>,
    state: S,
    addr: std::net::SocketAddr,
    ui_path: Option<P>,
) -> anyhow::Result<()>
where
    S: Clone + Send + Sync + 'static,
    P: AsRef<str>,
{
    let (terminator, mut rx) = termination::create_termination();

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // listen for termination requests from zoraxy via HTTP POST /term/
    let termination_handler = async move |_: axum::extract::State<S>| {
        if let Err(e) = terminator.terminate(termination::Interrupted::UserInt) {
            tracing::error!("Failed to send termination signal: {e:?}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        } else {
            axum::http::StatusCode::OK
        }
    };

    let app = if let Some(ui_path) = ui_path {
        let term_path = format!("{}/term", ui_path.as_ref().trim_end_matches('/'));

        app.route(&term_path, axum::routing::get(termination_handler.clone()))
    } else {
        app
    };

    let server_future = axum::serve(listener, app.with_state(state));

    tokio::select! {
        _ = rx.recv() => {
            tracing::info!("Termination signal received, shutting down plugin.");
            Ok(())
        }
        result = server_future => {
            if let Err(e) = result {
                tracing::error!("Axum server error: {:?}", e);
                Err(anyhow::anyhow!("Axum server error: {:?}", e))
            } else {
                tracing::info!("Axum server has shut down.");
                Ok(())
            }
        }
    }
}
