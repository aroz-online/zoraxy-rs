use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use include_dir::include_dir;
use zoraxy_rs::prelude::*;

static WWW: include_dir::Dir = include_dir!("examples/helloworld_www");
const UI_PATH: &str = "/ui";
const UI_PATH_SLASH: &str = "/ui/";

#[tokio::main]
async fn main() -> Result<()> {
    let metadata = PluginMetadata::new(PluginType::Utilities)
        .with_id("com.example.helloworld")
        .with_name("Hello World Plugin")
        .with_description("A simple \"hello world\"")
        .with_author("foobar")
        .with_contact("foobar@example.com")
        .with_url("https://example.com")
        .with_version((1, 0, 0));
    let intro_spect = IntroSpect::new(metadata).with_ui_path(UI_PATH);
    let runtime_cfg = serve_and_recv_spec(std::env::args().collect(), &intro_spect)?;

    init_tracing_subscriber(true);

    let ui_router = Arc::new(PluginUiRouter::new(&WWW, "/"));
    let app = Router::new().nest_service(UI_PATH_SLASH, ui_router.into_service());

    let addr: SocketAddr = format!("127.0.0.1:{}", runtime_cfg.port).parse()?;
    tracing::info!("Hello World UI ready at http://{}", addr);
    start_plugin(app, (), addr, Some(UI_PATH)).await
}
