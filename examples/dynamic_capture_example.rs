use std::net::SocketAddr;

use axum::{
    Router,
    body::Body,
    debug_handler,
    extract::State,
    handler::HandlerWithoutStateExt,
    http::Request,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use zoraxy_rs::prelude::*;

const SNIFF_INGRESS: &str = "/d_sniff";
const SNIFF_INGRESS_SLASH: &str = "/d_sniff/";
const CAPTURE_INGRESS: &str = "/d_capture";
const CAPTURE_INGRESS_SLASH: &str = "/d_capture/";
const UI_PATH: &str = "/debug";
const UI_PATH_SLASH: &str = "/debug/";

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

#[debug_handler]
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

    init_tracing_subscriber(true);

    let capture_service = DynamicCaptureService::new("/d_capture/", capture.into_service());

    let app = Router::new()
        .route(SNIFF_INGRESS_SLASH, post(sniff))
        .nest_service(CAPTURE_INGRESS_SLASH, capture_service)
        .route(UI_PATH_SLASH, get(render_debug_ui));

    let addr: SocketAddr = format!("127.0.0.1:{}", runtime_cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Dynamic capture example listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn render_debug_ui(req: Request<Body>) -> Html<String> {
    let mut headers: Vec<_> = req
        .headers()
        .iter()
        .map(|(name, value)| {
            let value = value.to_str().unwrap_or("<non-utf8>");
            format!("<li><strong>{name}</strong>: {value}</li>")
        })
        .collect();
    headers.sort();

    let body = format!(
        r#"
		<html>
			<head>
				<title>Zoraxy Static Capture Example</title>
			</head>
			<body>
				<h1>Plugin UI Debug Interface</h1>
				<h2>Received Headers</h2>
				<ul>{}</ul>
				<h2>Request Details</h2>
				<ul>
					<li><strong>Method:</strong> {}</li>
					<li><strong>URI:</strong> {}</li>
					<li><strong>Version:</strong> {:?}</li>
				</ul>
			</body>
		</html>
		"#,
        headers.join(""),
        req.method(),
        req.uri(),
        req.version()
    );

    Html(body)
}
