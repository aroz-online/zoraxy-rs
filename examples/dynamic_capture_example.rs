use std::net::SocketAddr;

use axum::{
    Router,
    body::Body,
    extract::State,
    handler::HandlerWithoutStateExt,
    http::Request,
    response::{Html, IntoResponse},
    routing::get,
};
use zoraxy_rs::prelude::*;

const SNIFF_INGRESS: &str = "/d_sniff";
const CAPTURE_INGRESS: &str = "/d_capture";
const UI_PATH: &str = "/ui";

fn instrospect() -> IntroSpect {
    let metadata = PluginMetadata::new(PluginType::Router)
        .with_id("org.aroz.zoraxy.dynamic-capture-example")
        .with_name("Zoraxy Dynamic Capture Example Plugin")
        .with_author("aroz.org")
        .with_contact("https://aroz.org")
        .with_description("An example Zoraxy plugin demonstrating dynamic path capture routing.")
        .with_url("https://zoraxy.aroz.org")
        .with_version((1, 0, 0));
    let settings = DynamicCaptureSettings::new(SNIFF_INGRESS, CAPTURE_INGRESS);
    IntroSpect::new(metadata)
        .with_dynamic_capture_settings(settings)
        .with_ui_path(UI_PATH)
}

async fn sniff(State(_): State<()>, sniff: DynamicSniffForwardRequest) -> SniffDecision {
    if sniff.request_uri.starts_with("/foobar") {
        tracing::info!("Sniffed request: {:?}", sniff);

        return SniffDecision::Accept;
    }
    SniffDecision::Skip
}

async fn capture(req: Request<Body>) -> impl IntoResponse {
    Html(format!(
        "<h1>Welcome to the dynamic capture handler!</h1><br/><h2>Request Info:</h2><p>Request URI: {}</p><p>Request Method: {}</p><p>Request Headers: {:#?}</p>",
        req.uri(),
        req.method(),
        req.headers()
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime_cfg = serve_and_recv_spec(std::env::args().collect(), &instrospect())?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let capture_service = DynamicCaptureService::new("/d_capture/", capture.into_service());

    let app = Router::new()
        .route(SNIFF_INGRESS, get(sniff))
        .nest_service(CAPTURE_INGRESS, capture_service);

    let addr: SocketAddr = format!("127.0.0.1:{}", runtime_cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Dynamic capture example listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
