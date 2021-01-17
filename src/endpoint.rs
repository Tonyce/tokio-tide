use async_trait::async_trait;
use futures::Future;
use http::{header::HeaderValue, Request, Response, StatusCode};

#[async_trait]
pub trait Endpoint<State: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    async fn call(&self, req: Request<Vec<u8>>) -> String;
}

pub type DynEndpoint<State> = dyn Endpoint<State>;

#[async_trait]
impl<State, F, Fut> Endpoint<State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Request<Vec<u8>>) -> Fut,
    Fut: Future<Output = String> + Send + 'static,
{
    async fn call(&self, req: Request<Vec<u8>>) -> String {
        let fut = (self)(req);

        fut.await
    }
}