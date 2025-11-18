use std::borrow::Cow;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use axum::body::{Body, Bytes};
use axum::http::header::{self, HeaderValue};
use axum::http::{Method, Request, Response, StatusCode};
use include_dir::{Dir, File};
use tower::Service;
use tracing::{debug, warn};

const CSRF_HEADER: &str = "x-zoraxy-csrf";
const DEFAULT_CSRF_TOKEN: &str = "missing-csrf-token";

type BoxedFuture =
    Pin<Box<dyn Future<Output = Result<Response<Body>, Infallible>> + Send + 'static>>;

/// Axum-compatible service that serves plugin UI files from an embedded directory.
pub struct PluginUiRouter {
    handler_prefix: String,
    ui_dir: &'static Dir<'static>,
    debug_enabled: AtomicBool,
}

impl PluginUiRouter {
    pub fn new(ui_dir: &'static Dir<'static>, handler_prefix: impl AsRef<str>) -> Self {
        Self {
            handler_prefix: normalize_prefix(handler_prefix.as_ref()),
            ui_dir,
            debug_enabled: AtomicBool::new(false),
        }
    }

    pub fn handler_prefix(&self) -> &str {
        &self.handler_prefix
    }

    pub fn set_debug(&self, enable: bool) {
        self.debug_enabled.store(enable, Ordering::Relaxed);
    }

    fn debug_enabled(&self) -> bool {
        self.debug_enabled.load(Ordering::Relaxed)
    }

    fn serve_request(&self, req: &Request<Body>) -> Response<Body> {
        let method = req.method().clone();
        let path = req.uri().path().to_owned();
        let csrf_token = req
            .headers()
            .get(CSRF_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or(DEFAULT_CSRF_TOKEN);

        let Some(stripped) = self.strip_handler_prefix(&path) else {
            return not_found(&path);
        };

        let trimmed = stripped.trim_matches('/');
        let Some(sanitized) = sanitize_virtual_path(trimmed) else {
            return not_found(&path);
        };
        let candidates = candidate_paths(&sanitized);

        if self.debug_enabled() {
            debug!(
                target: "zoraxy::plugin_ui",
                ?path,
                handler_prefix = %self.handler_prefix,
                sanitized = %sanitized,
                ?candidates,
                "Serving embedded asset"
            );
        }

        for candidate in candidates {
            if let Some(file) = self.ui_dir.get_file(&candidate).cloned() {
                return Self::build_response(&file, &method, csrf_token);
            }
        }

        if self.debug_enabled() {
            warn!(target: "zoraxy::plugin_ui", %path, "Embedded asset not found");
        }

        not_found(&path)
    }

    fn build_response(file: &File<'_>, method: &Method, csrf_token: &str) -> Response<Body> {
        let mime = mime_guess::from_path(file.path()).first_or_octet_stream();
        let bytes = file.contents();
        let body_bytes = if is_html_file(file) {
            render_html(bytes, csrf_token)
        } else {
            Cow::Borrowed(bytes)
        };

        let bytes = match body_bytes {
            Cow::Owned(buf) => Bytes::from(buf),
            Cow::Borrowed(slice) => Bytes::copy_from_slice(slice),
        };

        let content_len = bytes.len();
        let mut response = if method == Method::HEAD {
            Response::new(Body::empty())
        } else {
            Response::new(Body::from(bytes))
        };

        *response.status_mut() = StatusCode::OK;
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime.as_ref())
                .unwrap_or(HeaderValue::from_static("application/octet-stream")),
        );
        if let Ok(value) = HeaderValue::from_str(&content_len.to_string()) {
            headers.insert(header::CONTENT_LENGTH, value);
        }
        response
    }

    fn strip_handler_prefix<'a>(&self, path: &'a str) -> Option<&'a str> {
        if self.handler_prefix == "/" {
            return Some(path);
        }

        if path == self.handler_prefix {
            return Some("/");
        }

        if path.starts_with(&self.handler_prefix) {
            let remainder = &path[self.handler_prefix.len()..];
            if remainder.starts_with('/') {
                return Some(remainder);
            }
        }

        None
    }

    pub const fn into_service(self: Arc<Self>) -> PluginUiRouterService {
        PluginUiRouterService { inner: self }
    }
}

#[derive(Clone)]
pub struct PluginUiRouterService {
    inner: Arc<PluginUiRouter>,
}

impl Service<Request<Body>> for PluginUiRouterService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxedFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let router = self.inner.clone();
        Box::pin(async move { Ok(router.serve_request(&req)) })
    }
}

fn not_found(path: &str) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(format!("{path} not found")))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}

fn normalize_prefix(prefix: &str) -> String {
    let trimmed = prefix.trim();
    if trimmed.is_empty() || trimmed == "/" {
        return "/".to_string();
    }

    let mut normalized = trimmed.to_string();
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    while normalized.ends_with('/') {
        normalized.pop();
    }
    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized
    }
}

fn sanitize_virtual_path(path: &str) -> Option<String> {
    let mut sanitized = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return None;
        }
        sanitized.push(segment);
    }
    Some(sanitized.join("/"))
}

fn candidate_paths(sanitized: &str) -> Vec<String> {
    if sanitized.is_empty() {
        vec!["index.html".to_string()]
    } else {
        vec![sanitized.to_string(), format!("{}/index.html", sanitized)]
    }
}

fn is_html_file(file: &File<'_>) -> bool {
    file.path()
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("html"))
}

fn render_html<'a>(bytes: &'a [u8], csrf: &str) -> Cow<'a, [u8]> {
    std::str::from_utf8(bytes).map_or(Cow::Borrowed(bytes), |content| {
        if content.contains("{{.csrfToken}}") {
            Cow::Owned(content.replace("{{.csrfToken}}", csrf).into_bytes())
        } else {
            Cow::Borrowed(bytes)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use include_dir::include_dir;

    static TEST_UI: Dir = include_dir!("$CARGO_MANIFEST_DIR/examples/helloworld_www");

    #[tokio::test]
    async fn serves_index_html() {
        let router = Arc::new(PluginUiRouter::new(&TEST_UI, "/"));
        let request = Request::builder()
            .uri("/")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.serve_request(&request);
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(body.windows(11).any(|window| window == b"Hello World"));
    }

    #[test]
    fn returns_404_for_missing_asset() {
        let router = Arc::new(PluginUiRouter::new(&TEST_UI, "/"));
        let request = Request::builder()
            .uri("/does-not-exist.js")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = router.serve_request(&request);
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
