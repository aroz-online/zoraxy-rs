use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use axum::body::Body;
use axum::extract::State;
use axum::http::Request;
use axum::response::Html;
use axum::routing::get;
use axum::{Router, debug_handler};
use html_escape::encode_text;
use zoraxy_rs::prelude::*;

fn introspect() -> IntroSpect {
    let metadata = PluginMetadata::new(PluginType::Utilities)
        .with_id("org.aroz.zoraxy.api-call-example")
        .with_name("API Call Example Plugin")
        .with_author("Anthony Rubick")
        .with_description("An example plugin for making API calls")
        .with_url("https://zoraxy.aroz.org")
        .with_version((1, 0, 0));
    IntroSpect::new(metadata)
        .with_ui_path("/ui")
        .add_permitted_api_endpoint(
            PermittedApiEndpoint::new("GET", "/plugin/api/access/list")
                .with_reason("Used to display all configured Access Rules"),
        )
}

#[derive(Clone, Debug)]
struct Context {
    port: u16,
    api_key: String,
    zoraxy_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime_cfg = serve_and_recv_spec(std::env::args().collect(), &introspect())?;

    init_tracing_subscriber(false);

    let api_key = runtime_cfg
        .api_key
        .clone()
        .ok_or(anyhow!("missing API Key in runtime configuration"))?;
    let zoraxy_port = runtime_cfg
        .zoraxy_port
        .ok_or(anyhow!("missing Zoraxy Port in runtime configuration"))?;
    tracing::info!(
        "API Call Example Plugin initialized with port: {}, api_key: {}, zoraxy_port: {}",
        runtime_cfg.port,
        api_key,
        zoraxy_port
    );

    let state = Context {
        port: runtime_cfg.port,
        api_key,
        zoraxy_port,
    };

    let app = Router::new()
        .route("/ui/", get(render_ui))
        .fallback(get(async |_: Request<Body>| Html("<h1>Not Found</h1>")));
    let addr: SocketAddr = format!("127.0.0.1:{}", runtime_cfg.port).parse()?;
    tracing::info!("API Call Example Plugin UI ready at http://{addr}");
    start_plugin(app, state, addr, Some("/ui")).await
}

async fn allowed_endpoint(ctx: &Context) -> Result<String> {
    // Make an API call to the permitted endpoint
    let client = reqwest::Client::new();
    let api_url = format!(
        "http://localhost:{}/plugin/api/access/list",
        ctx.zoraxy_port
    );
    let resp = client
        .get(&api_url)
        .bearer_auth(&ctx.api_key)
        .send()
        .await?;
    tracing::info!("Allowed endpoint response status: {}", resp.status());
    if resp.status().is_success() {
        return Ok(resp.text().await?);
    } else {
        return Err(anyhow!(resp.text().await?));
    }
}

async fn allowed_endpoint_invalid_key(ctx: &Context) -> Result<String> {
    // Make an API call to the permitted endpoint with an invalid key
    let client = reqwest::Client::new();
    let api_url = format!(
        "http://localhost:{}/plugin/api/access/list",
        ctx.zoraxy_port
    );
    let resp = client
        .get(&api_url)
        .bearer_auth("invalid-key")
        .send()
        .await?;
    tracing::info!(
        "Allowed endpoint invalid key response status: {}",
        resp.status()
    );
    if resp.status().is_success() {
        return Ok(resp.text().await?);
    } else {
        return Err(anyhow!(resp.text().await?));
    }
}

async fn unaccessible_endpoint(ctx: &Context) -> Result<String> {
    // Make an API call to an endpoint that is not permitted
    let client = reqwest::Client::new();
    let api_url = format!(
        "http://localhost:{}/api/acme/listExpiredDomains",
        ctx.zoraxy_port
    );
    let resp = client
        .get(&api_url)
        .bearer_auth(&ctx.api_key)
        .send()
        .await?;
    tracing::info!("Unaccessible endpoint response status: {}", resp.status());
    if resp.status().is_success() {
        return Ok(resp.text().await?);
    } else {
        return Err(anyhow!(resp.text().await?));
    }
}

async fn unpermitted_endpoint(ctx: &Context) -> Result<String> {
    // Make an API call to an endpoint that is plugin-accessible but is not permitted
    let client = reqwest::Client::new();
    let api_url = format!("http://localhost:{}/plugin/api/proxy/list", ctx.zoraxy_port);
    let resp = client
        .get(&api_url)
        .bearer_auth(&ctx.api_key)
        .send()
        .await?;
    tracing::info!("Unpermitted endpoint response status: {}", resp.status());
    if resp.status().is_success() {
        return Ok(resp.text().await?);
    } else {
        return Err(anyhow!(resp.text().await?));
    }
}

#[debug_handler]
async fn render_ui(State(ctx): State<Context>, _: Request<Body>) -> Html<String> {
    let head = r#"<head>
        <title>API Call Example Plugin UI</title>
        <meta charset="UTF-8">
        <link rel="stylesheet" href="/script/semantic/semantic.min.css">
        <script src="/script/jquery-3.6.0.min.js"></script>
        <script src="/script/semantic/semantic.min.js"></script>
        <link rel="stylesheet" href="/main.css">
        <style>
            body {
                background: none;
            }

            .response-block {
                background-color: var(--theme_bg_primary);
                border: 1px solid var(--theme_divider);
                border-radius: 8px;
                padding: 20px;
                margin: 15px 0;
                box-shadow: 0 2px 4px rgba(0,0,0,0.1);
                transition: box-shadow 0.3s ease;
            }
            .response-block:hover {
                box-shadow: 0 4px 8px rgba(0,0,0,0.15);
            }
            .response-block h3 {
                margin-top: 0;
                color: var(--text_color);
                border-bottom: 2px solid #007bff;
                padding-bottom: 8px;
            }
            .response-block.success {
                border-left: 4px solid #28a745;
            }
            .response-block.error {
                border-left: 4px solid #dc3545;
            }
            .response-block.warning {
                border-left: 4px solid #ffc107;
            }
            .response-content {
                margin-top: 10px;
            }
            .response-content pre {
                background-color: var(--theme_highlight);
                border: 1px solid var(--theme_divider);
                border-radius: 4px;
                padding: 10px;
                overflow: auto;
                font-size: 12px;
                line-height: 1.4;
                height: 200px;
                max-height: 80vh;
                min-height: 100px;
                resize: vertical;
                box-sizing: border-box;
            }
        </style>
    </head>"#;

    let rendered_access_list_html = match allowed_endpoint(&ctx).await {
        Ok(html) => format!("<pre>{}</pre>", encode_text(html.as_str())),
        Err(e) => format!(
            "<p>Error fetching access list: {}</p>",
            encode_text(&e.to_string())
        ),
    };

    let rendered_invalid_key_response_html = match allowed_endpoint_invalid_key(&ctx).await {
        Ok(html) => format!("<pre>{}</pre>", encode_text(html.as_str())),
        Err(e) => format!(
            "<p>Error fetching invalid key response: {}</p>",
            encode_text(&e.to_string())
        ),
    };
    let rendered_unpermitted_response_html = match unpermitted_endpoint(&ctx).await {
        Ok(html) => format!("<pre>{}</pre>", encode_text(html.as_str())),
        Err(e) => format!(
            "<p>Error fetching unpermitted response: {}</p>",
            encode_text(&e.to_string())
        ),
    };
    let rendered_unaccessible_response_html = match unaccessible_endpoint(&ctx).await {
        Ok(html) => format!("<pre>{}</pre>", encode_text(html.as_str())),
        Err(e) => format!(
            "<p>Error fetching unaccessible response: </p><pre>{}</pre>",
            encode_text(&e.to_string())
        ),
    };

    let body = format!("<body><!-- Dark theme script must be included after body tag-->
	<link rel=\"stylesheet\" href=\"/darktheme.css\">
	<script src=\"/script/darktheme.js\"></script>
	<div class=\"ui container\">

		<div class=\"ui basic segment\">
			<h1 class=\"ui header\">Welcome to the API Call Example Plugin UI</h1>
			<p>Plugin is running on port: {}</p>
		</div>
		<div class=\"ui divider\"></div>

		<h2>API Call Examples</h2>

		<div class=\"response-block success\">
			<h3>✅ Allowed Endpoint (Valid API Key)</h3>
			<p>Making a GET request to <code>/plugin/api/access/list</code> with a valid API key:</p>
			<div class=\"response-content\">{}</div>
		</div>

		<div class=\"response-block warning\">
			<h3>⚠️ Invalid API Key</h3>
			<p>Making a GET request to <code>/plugin/api/access/list</code> with an invalid API key:</p>
			<div class=\"response-content\">{}</div>
		</div>

		<div class=\"response-block warning\">
			<h3>⚠️ Unpermitted Endpoint</h3>
			<p>Making a GET request to <code>/plugin/api/proxy/list</code> (not a permitted endpoint):</p>
			<div class=\"response-content\">{}</div>
		</div>

		<div class=\"response-block error\">
			<h3>❌ Disallowed Endpoint</h3>
			<p>Making a GET request to <code>/api/acme/listExpiredDomains</code> (not a plugin-accessible endpoint):</p>
			<div class=\"response-content\">{}</div>
		</div>
	</div>
	</body>", ctx.port,
		rendered_access_list_html,
		rendered_invalid_key_response_html,
		rendered_unpermitted_response_html,
		rendered_unaccessible_response_html,
		);

    let html = format!("<!DOCTYPE html><html>{head}{body}</html>");

    Html(html)
}
