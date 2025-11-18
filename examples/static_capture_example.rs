use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::handler::HandlerWithoutStateExt;
use axum::http::Request;
use axum::response::Html;
use axum::routing::get;
use zoraxy_rs::prelude::*;

const UI_PATH: &str = "/ui";
const UI_PATH_SLASH: &str = "/ui/";
const STATIC_CAPTURE_INGRESS: &str = "/s_capture";
const STATIC_CAPTURE_INGRESS_SLASH: &str = "/s_capture/";

fn introspect() -> IntroSpect {
    let metadata = PluginMetadata::new(PluginType::Router)
        .with_id("org.aroz.zoraxy.static-capture-example")
        .with_name("Zoraxy Static Capture Example Plugin")
        .with_author("aroz.org")
        .with_contact("https://aroz.org")
        .with_description("An example Zoraxy plugin demonstrating static path capture routing.")
        .with_url("https://zoraxy.aroz.org")
        .with_version((1, 0, 0));
    let settings = StaticCaptureSettings::new(STATIC_CAPTURE_INGRESS)
        .add_static_capture_path("/test_a")
        .add_static_capture_path("/test_b");
    IntroSpect::new(metadata)
        .with_static_capture_settings(settings)
        .with_ui_path(UI_PATH)
}

async fn default_handler(req: Request<Body>) -> Html<String> {
    Html(format!(
        "This request is captured by the default handler!<br/>Request URI: {}",
        req.uri()
    ))
}

async fn handler_a(req: Request<Body>) -> Html<String> {
    Html(format!(
        "This request is captured by A handler!<br/>Request URI: {}",
        req.uri()
    ))
}

async fn handler_b(req: Request<Body>) -> Html<String> {
    Html(format!(
        "This request is captured by B handler!<br/>Request URI: {}",
        req.uri()
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime_cfg = serve_and_recv_spec(std::env::args().collect(), &introspect())?;

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut path_router = StaticPathRouter::new(default_handler.into_service());

    path_router.register_path_service("/test_a", handler_a.into_service());

    path_router.register_path_service("/test_b", handler_b.into_service());

    let static_capture = Arc::new(path_router).into_capture_service();

    let app = Router::new()
        .route(UI_PATH_SLASH, get(render_debug_ui))
        .route_service(STATIC_CAPTURE_INGRESS_SLASH, static_capture);

    let addr: SocketAddr = format!("127.0.0.1:{}", runtime_cfg.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Static capture example listening on http://{}", addr);
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
