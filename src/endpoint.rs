use async_trait::async_trait;
use futures::Future;

use http::Request;
use route_recognizer::Params;

#[async_trait]
pub trait Endpoint<State: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    async fn call(&self, state: State, req: Request<Vec<u8>>, route_params: Vec<Params>) -> String;
}

pub type DynEndpoint<State> = dyn Endpoint<State>;

#[async_trait]
impl<State, F, Fut> Endpoint<State> for F
where
    State: Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(State, Request<Vec<u8>>, Vec<Params>) -> Fut,
    Fut: Future<Output = String> + Send + 'static,
{
    async fn call(&self, state: State, req: Request<Vec<u8>>, route_params: Vec<Params>) -> String {
        let fut = (self)(state, req, route_params);
        fut.await
    }
}
