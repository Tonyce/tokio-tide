use crate::endpoint::{DynEndpoint, Endpoint};
use crate::router::{self, Router};

pub struct Route<'a, State> {
    router: &'a mut Router<State>,
    path: String,
}

impl<'a, State: Clone + Send + Sync + 'static> Route<'a, State> {
    pub(crate) fn new(router: &'a mut Router<State>, path: String) -> Self {
        Self { router, path }
    }

    pub fn method(&mut self, method: http::Method, ep: impl Endpoint<State>) -> &mut Self {
        self.router.add(&self.path, method, Box::new(ep));
        self
    }

    pub fn get(&mut self, ep: impl Endpoint<State>) -> &mut Self {
        self.method(http::Method::GET, ep);
        self
    }
}
