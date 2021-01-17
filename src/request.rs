// use crate::http::{self, Body, Method, Mime, StatusCode, Url, Version};
use pin_project::pin_project;
use route_recognizer::Params;
// use std::pin::Pin;

#[pin_project]
#[derive(Debug)]
pub struct Request<State> {
    pub state: State,
    #[pin]
    pub req: http::Request<Vec<u8>>,
    pub route_params: Vec<Params>,
}

impl<State> Request<State> {
    /// Create a new `Request`.
    pub(crate) fn new(
        state: State,
        req: http::Request<Vec<u8>>,
        route_params: Vec<Params>,
    ) -> Self {
        Self {
            state,
            req,
            route_params,
        }
    }
}
