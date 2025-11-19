use std::{net::SocketAddr, sync::Arc};

use axum::{
    Json, Router, debug_handler,
    extract::{Path, State},
    response::Html,
    routing::{get, post},
};
use tokio::sync::Mutex;
use zoraxy_rs::prelude::*;

fn instrospect() -> IntroSpect {
    let metadata = PluginMetadata::new(PluginType::Utilities)
        .with_id("org.aroz.zoraxy.event_subscriber_example")
        .with_name("Event Subscriber Example Plugin")
        .with_author("Anthony Rubick")
        .with_description(
            "An example plugin for event subscriptions, will display all events in the UI",
        )
        .with_url("https://zoraxy.aroz.org")
        .with_version((1, 0, 0));
    IntroSpect::new(metadata)
        .with_ui_path("/ui")
        .with_subscriptions(
            SubscriptionsSettings::new("/notifyme")
                .add_event_subscription(
                    EventName::BlacklistedIpBlocked,
                    "This event is triggered when a blacklisted IP is blocked",
                )
                .add_event_subscription(
                    EventName::BlacklistToggled,
                    "This event is triggered when the blacklist is toggled for an access rule",
                )
                .add_event_subscription(
                    EventName::AccessRuleCreated,
                    "This event is triggered when a new access ruleset is created",
                )
                .add_event_subscription(
                    EventName::CustomEvent,
                    "This event is a custom event that can be emitted by any plugin, we subscribe to it to demonstrate a \"monitor\" plugin that can see all custom events emitted by other plugins",
                ),
        )
}

#[derive(Clone)]
struct AppState {
    event_log: Arc<Mutex<Vec<Event>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime_cfg = serve_and_recv_spec(std::env::args().collect(), &instrospect())?;

    init_tracing_subscriber(true);

    let app = Router::new()
        .route("/ui/", get(render_ui))
        .route("/notifyme/{event_name}", post(handle_event));

    let state = AppState {
        event_log: Arc::new(Mutex::new(Vec::new())),
    };

    let addr: SocketAddr = format!("127.0.0.1:{}", runtime_cfg.port).parse()?;
    tracing::info!("Event Subscriber Example Plugin UI ready at http://{addr}");
    start_plugin(app, state, addr, Some("/ui")).await
}

#[debug_handler]
async fn handle_event(
    State(state): State<AppState>,
    Path(event_name): Path<String>,
    Json(event): Json<Event>,
) -> String {
    tracing::info!("Received event: {event_name:?}");
    let mut log = state.event_log.lock().await;
    if log.len() >= 100 {
        log.remove(0);
    }
    log.push(event.clone());

    format!("Event received: {}", event.name)
}

#[debug_handler]
async fn render_ui(State(state): State<AppState>) -> Html<String> {
    let log = state.event_log.lock().await;
    let mut event_log_html = String::new();

    if log.is_empty() {
        event_log_html.push_str(
            "<p>No events received yet<br>Try toggling a blacklist or something like that</p>",
        );
    } else {
        for event in log.iter() {
            let raw_event_data = serde_json::to_string_pretty(event).unwrap_or_default();
            let formatted_timestamp =
                chrono::DateTime::<chrono::Utc>::from_timestamp(event.timestamp as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Invalid timestamp".to_string());
            event_log_html.push_str(&format!(
                "<div class='response-block'>
                    <h3>{} at {}</h3>
                    <div class='response-content'>
                        <p class='ui meta'>Event Data:</p>
                        <pre>{}</pre>
                    </div>
                </div>",
                event.name, formatted_timestamp, raw_event_data
            ));
        }
    }

    let html = r#"
    
    <!DOCTYPE html>
    <html>
    <head>
        <title>Event Log</title>
        <meta charset="UTF-8">
        <link rel="stylesheet" href="/script/semantic/semantic.min.css">
        <script src="/script/jquery-3.6.0.min.js"></script>
        <script src="/script/semantic/semantic.min.js"></script>
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
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
                margin: 5px 0;
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
                height: fit-content;
                max-height: 80vh;
                resize: vertical;
                box-sizing: border-box;
            }
        </style>
    </head>
    <body>
    <!-- Dark theme script must be included after body tag-->
    <link rel="stylesheet" href="/darktheme.css">
    <script src="/script/darktheme.js"></script>
    <div class="ui container">

        <h1>Event Log</h1>
        <div id="event-log" class="ui basic segment">"#
        .to_string()
        + &event_log_html
        + r#"</div>
    </body>
    </html>
    "#;

    Html(html)
}
