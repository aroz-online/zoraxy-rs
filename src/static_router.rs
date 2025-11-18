use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use axum::body::Body;
use axum::http::{Request, Uri};
use axum::response::{IntoResponse, Response};
use tower::Service;
use tracing::{debug, warn};

const CAPTURE_HEADER: &str = "x-zoraxy-capture";
const ORIGINAL_URI_HEADER: &str = "x-zoraxy-uri";

type HandlerFuture = Pin<Box<dyn Future<Output = Response> + Send + 'static>>;
type SharedCaptureHandler = Arc<dyn CaptureHandler + Send + Sync>;

/// Trait alias for anything that can be invoked as a static-capture handler.
pub trait CaptureHandler: Send + Sync + 'static {
    fn call(&self, req: Request<Body>) -> HandlerFuture;
}

impl<F, Fut, R> CaptureHandler for F
where
    F: Fn(Request<Body>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + 'static,
{
    fn call(&self, req: Request<Body>) -> HandlerFuture {
        let fut = (self)(req);
        Box::pin(async move { fut.await.into_response() })
    }
}

/// Router that mimics the Go plugin static path capture behavior atop Axum services.
pub struct StaticPathRouter {
    handlers: HashMap<String, SharedCaptureHandler>,
    default_handler: SharedCaptureHandler,
    debug_enabled: AtomicBool,
}

impl Default for StaticPathRouter {
    fn default() -> Self {
        Self::new(|_req: Request<Body>| async move {
            (
                axum::http::StatusCode::NOT_FOUND,
                "No capture handler registered",
            )
                .into_response()
        })
    }
}

impl StaticPathRouter {
    pub fn new(default_handler: impl CaptureHandler + 'static) -> Self {
        let default_handler: SharedCaptureHandler =
            Arc::new(default_handler) as SharedCaptureHandler;
        Self {
            handlers: HashMap::new(),
            default_handler,
            debug_enabled: AtomicBool::new(false),
        }
    }

    pub fn register_path_handler<H>(&mut self, path: impl AsRef<str>, handler: H)
    where
        H: CaptureHandler + Send + Sync + 'static,
    {
        let normalized = normalize_capture_path(path.as_ref());
        let handler: SharedCaptureHandler = Arc::new(handler) as SharedCaptureHandler;
        self.handlers.insert(normalized, handler);
    }

    pub fn remove_path_handler(&mut self, path: impl AsRef<str>) {
        let normalized = normalize_capture_path(path.as_ref());
        self.handlers.remove(&normalized);
    }

    pub fn set_debug_print_mode(&self, enable: bool) {
        self.debug_enabled.store(enable, Ordering::Relaxed);
    }

    pub fn debug_enabled(&self) -> bool {
        self.debug_enabled.load(Ordering::Relaxed)
    }

    fn handler_for_path(&self, path: &str) -> Option<SharedCaptureHandler> {
        self.handlers.get(path).cloned()
    }

    fn fallback_handler(&self) -> SharedCaptureHandler {
        self.default_handler.clone()
    }

    fn log_capture_path(&self, capture_path: &str) {
        if self.debug_enabled() {
            debug!(target: "zoraxy::static_router", capture_path, "Using capture path");
        }
    }

    pub(crate) async fn dispatch_capture(&self, mut req: Request<Body>) -> Response {
        if let Some(capture_path) = header_value(req.headers().get(CAPTURE_HEADER)) {
            let normalized_path = normalize_capture_path(&capture_path);
            self.log_capture_path(&normalized_path);

            if let Some(original_uri) = header_value(req.headers().get(ORIGINAL_URI_HEADER))
                && let Err(err) = rewrite_request_path(req.uri_mut(), &original_uri)
            {
                warn!(target: "zoraxy::static_router", %original_uri, %err, "Failed to rewrite request URI");
            }

            if let Some(handler) = self.handler_for_path(&normalized_path) {
                return handler.call(req).await;
            }
        }

        self.fallback_handler().call(req).await
    }

    pub const fn into_capture_service(self: Arc<Self>) -> StaticCaptureService {
        StaticCaptureService::new(self)
    }
}

fn header_value(value: Option<&axum::http::HeaderValue>) -> Option<String> {
    value
        .and_then(|val| val.to_str().ok())
        .map(ToString::to_string)
}

fn normalize_capture_path(path: &str) -> String {
    if path.len() > 1 && path.ends_with('/') {
        path[..path.len() - 1].to_owned()
    } else {
        path.to_owned()
    }
}

fn rewrite_request_path(uri: &mut Uri, new_path: &str) -> Result<(), axum::http::Error> {
    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(new_path.parse()?);
    *uri = Uri::from_parts(parts)?;
    Ok(())
}

#[derive(Clone)]
pub struct StaticCaptureService {
    router: Arc<StaticPathRouter>,
}

impl StaticCaptureService {
    pub const fn new(router: Arc<StaticPathRouter>) -> Self {
        Self { router }
    }
}

impl Service<Request<Body>> for StaticCaptureService {
    type Response = Response;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let router = self.router.clone();
        Box::pin(async move { Ok(router.dispatch_capture(req).await) })
    }
}
