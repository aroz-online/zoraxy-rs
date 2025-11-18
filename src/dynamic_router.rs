use axum::body::{self, Body};
use axum::extract::FromRequest;
use axum::http::{Request, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;
use tower::util::BoxCloneSyncService;

const REQUEST_ID_HEADER: &str = "x-zoraxy-requestid";

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicSniffForwardRequest {
    pub method: String,
    pub hostname: String,
    pub url: String,
    pub header: HashMap<String, Vec<String>>,
    pub remote_addr: String,
    pub host: String,
    pub request_uri: String,
    pub proto: String,
    pub proto_major: i32,
    pub proto_minor: i32,
    #[serde(skip)]
    request_uuid: Option<String>,
    #[serde(skip)]
    raw_request: Option<Request<Body>>,
}

impl DynamicSniffForwardRequest {
    pub fn request_uuid(&self) -> Option<&str> {
        self.request_uuid.as_deref()
    }

    fn set_request_uuid(&mut self, value: Option<String>) {
        self.request_uuid = value;
    }

    pub fn raw_request(&self) -> Option<&Request<Body>> {
        self.raw_request.as_ref()
    }

    fn set_raw_request(&mut self, req: Option<Request<Body>>) {
        self.raw_request = req;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SniffExtractorError {
    #[error("failed to read sniff request body: {0}")]
    Body(#[from] axum::Error),
    #[error("failed to decode sniff payload: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for SniffExtractorError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl<S> FromRequest<S> for DynamicSniffForwardRequest
where
    S: Send + Sync + Clone,
{
    type Rejection = SniffExtractorError;

    async fn from_request(req: axum::extract::Request, _: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let bytes = body::to_bytes(body, DEFAULT_SNIFF_BODY_LIMIT).await?;
        let mut payload: DynamicSniffForwardRequest = serde_json::from_slice(&bytes)?;
        let request_uuid = parts
            .headers
            .remove(REQUEST_ID_HEADER)
            .and_then(|value| value.to_str().ok().map(|s| s.to_string()));
        payload.set_request_uuid(request_uuid);
        payload.set_raw_request(Some(Request::from_parts(parts, Body::from(bytes))));
        Ok(payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniffDecision {
    Accept,
    Skip,
}

impl IntoResponse for SniffDecision {
    fn into_response(self) -> Response {
        match self {
            SniffDecision::Accept => (StatusCode::OK, "OK").into_response(),
            SniffDecision::Skip => (StatusCode::NOT_IMPLEMENTED, "SKIP").into_response(),
        }
    }
}

const DEFAULT_SNIFF_BODY_LIMIT: usize = 256 * 1024;

#[derive(Clone)]
pub struct DynamicCaptureService {
    inner: BoxCloneSyncService<Request<Body>, Response, Infallible>,
    ingress: String,
}

impl DynamicCaptureService {
    pub fn new<H>(ingress: &str, handler: H) -> Self
    where
        H: Service<Request<Body>, Response = Response, Error = Infallible>
            + Send
            + Sync
            + Clone
            + 'static,
        H::Future: Send + 'static,
    {
        Self {
            inner: BoxCloneSyncService::new(handler),
            ingress: normalize_ingress(ingress),
        }
    }
}

impl Service<Request<Body>> for DynamicCaptureService {
    type Response = Response;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Service::<Request<Body>>::poll_ready(&mut self.inner, cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        rewrite_capture_request(&self.ingress, &mut req);
        self.inner.call(req)
    }
}

fn rewrite_capture_request(ingress: &str, req: &mut Request<Body>) {
    let current_path = req.uri().path();
    if !current_path.starts_with(ingress) {
        return;
    }

    let remainder = &current_path[ingress.len()..];
    let mut normalized = if remainder.is_empty() {
        String::from('/')
    } else {
        format!("/{}", remainder.trim_start_matches('/'))
    };

    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    if let Some(query) = req.uri().query() {
        normalized.push('?');
        normalized.push_str(query);
    }

    if let Ok(path_and_query) = normalized.parse() {
        let mut parts = req.uri().clone().into_parts();
        parts.path_and_query = Some(path_and_query);
        if let Ok(new_uri) = Uri::from_parts(parts) {
            *req.uri_mut() = new_uri;
        }
    }
}

fn normalize_ingress(value: &str) -> String {
    let mut normalized = value.trim().to_string();
    if !normalized.starts_with('/') {
        normalized.insert(0, '/');
    }
    if !normalized.ends_with('/') {
        normalized.push('/');
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::handler::HandlerWithoutStateExt;
    use axum::http::Request;
    use axum::{Router, debug_handler};
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn parses_sniff_payload_and_request_id() {
        let body = json!({
            "method": "GET",
            "hostname": "example.com",
            "url": "http://example.com/test",
            "header": {},
            "remote_addr": "127.0.0.1:8080",
            "host": "example.com",
            "request_uri": "/test",
            "proto": "HTTP/1.1",
            "proto_major": 1,
            "proto_minor": 1
        });
        let req = Request::builder()
            .uri("/sniff")
            .header(REQUEST_ID_HEADER, "abc123")
            .body(Body::from(body.to_string()))
            .unwrap();

        let payload = DynamicSniffForwardRequest::from_request(req, &())
            .await
            .unwrap();
        assert_eq!(payload.method, "GET");
        assert_eq!(payload.request_uuid(), Some("abc123"));
    }

    #[tokio::test]
    async fn capture_rewrites_path() {
        #[debug_handler]
        async fn handler(req: Request<Body>) -> (StatusCode, String) {
            (StatusCode::OK, req.uri().path().to_string())
        }

        let capture_service = DynamicCaptureService::new("/d_capture/", handler.into_service());

        let app = Router::<()>::new().nest_service("/d_capture/", capture_service);

        let req = Request::builder()
            .uri("/d_capture/some/path?query=1")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        let body_bytes = body::to_bytes(response.into_body(), 1024).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(body_str, "/some/path");
    }
}
