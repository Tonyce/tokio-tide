use crate::middleware::Middleware;
use crate::router::{self, Router};
use crate::{
    endpoint::{DynEndpoint, Endpoint, MiddlewareEndpoint},
    middleware,
};

use std::sync::Arc;

pub struct Route<'a, State> {
    router: &'a mut Router<State>,
    path: String,
    middleware: Vec<Arc<dyn Middleware<State>>>,
}

impl<'a, State: Clone + Send + Sync + 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Self {
        Self {
            router,
            path,
            middleware: Vec::new(),
        }
    }

    pub fn method(&mut self, method: http::Method, ep: impl Endpoint<State>) -> &mut Self {
        // self.router.add(&self.path, method, Box::new(ep));
        self.router.add(
            &self.path,
            method,
            MiddlewareEndpoint::wrap_with_middleware(ep, &self.middleware),
        );
        self
    }

    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::GET, ep);
        self
    }

    /// Apply the given middleware to the current route.
    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        // println!(
        //     "Adding middleware {} to route {:?}",
        //     middleware.name(),
        //     self.path
        // );
        self.middleware.push(Arc::new(middleware));
        self
    }
}
