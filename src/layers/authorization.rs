use std::pin::Pin;
use std::task::{Context, Poll};

use crate::tracing::info;

use tonic::body::BoxBody;
use tower::{Layer, Service};

#[derive(Debug, Clone, Default)]
pub struct AuthorizationLayer {}

impl<S> Layer<S> for AuthorizationLayer {
    type Service = Authorization<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Authorization { inner }
    }
}

#[derive(Debug, Clone)]
pub struct Authorization<S> {
    pub inner: S,
}

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

impl<S> Service<hyper::Request<BoxBody>> for Authorization<S>
where
    S: Service<hyper::Request<BoxBody>, Response = hyper::Response<BoxBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<BoxBody>) -> Self::Future {
        let fut = self.inner.call(req);

        info!("Executing authorizationlayer verification");

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
