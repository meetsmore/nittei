use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, Request},
    response::Response,
};
use tower::{Layer, Service};
use uuid::Uuid;

static CORRELATION_ID_HEADER: HeaderName = HeaderName::from_static("x-correlation-id");

#[derive(Clone, Default)]
pub struct CorrelationIdLayer;

impl<S> Layer<S> for CorrelationIdLayer {
    type Service = CorrelationIdService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        CorrelationIdService { inner }
    }
}

#[derive(Clone)]
pub struct CorrelationIdService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for CorrelationIdService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        // 1) get or generate
        let cid = req
            .headers()
            .get(&CORRELATION_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // 2) put in extensions so handlers & TraceLayer can read it
        req.extensions_mut().insert(CorrelationId(cid.clone()));

        // proceed
        let fut = self.inner.clone().call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // 3) echo back on response
            res.headers_mut().insert(
                CORRELATION_ID_HEADER.clone(),
                HeaderValue::from_str(&cid).unwrap_or(HeaderValue::from_static("invalid")),
            );

            Ok(res)
        })
    }
}

#[derive(Clone, Debug)]
pub struct CorrelationId(pub String);
