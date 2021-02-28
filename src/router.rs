use crate::endpoint::{DynEndpoint, Endpoint};
use route_recognizer::{Match, Params, Router as MethodRouter};
use std::collections::HashMap;

pub struct Router<State> {
    pub method_map: HashMap<http::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    // method_map: String, // HashMap<http_types::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    // all_method_router: String, // MethodRouter<Box<DynEndpoint<State>>>,
}

pub(crate) struct Selection<'a, State> {
    pub(crate) endpoint: &'a DynEndpoint<State>,
    pub(crate) params: Params,
}

impl<State: Clone + Send + Sync + 'static> Router<State> {
    pub fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            // all_method_router: "state".to_string(),
        }
    }

    pub(crate) fn add(&mut self, path: &str, method: http::Method, ep: Box<DynEndpoint<State>>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, ep)
    }

    pub(crate) fn route(&self, path: &str, method: &http::Method) -> Selection<'_, State> {
        if let Some(m) = self
            .method_map
            .get(method)
            .and_then(|r| r.recognize(path).ok())
        {
            let handler = m.handler();
            let params = m.params().clone();
            Selection {
                endpoint: &***handler,
                params,
            }
        } else if method == http::Method::HEAD {
            // TODO ?
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual

            self.route(path, &http::Method::GET)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.recognize(path).is_ok())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection {
                endpoint: &method_not_allowed,
                params: Params::new(),
            }
        } else {
            Selection {
                endpoint: &not_found_endpoint,
                params: Params::new(),
            }
        }
    }
}

async fn not_found_endpoint<State: Clone + Send + Sync + 'static>(
    _state: State,
    _req: http::Request<Vec<u8>>,
    _route_params: Vec<Params>,
) -> http::Response<Vec<u8>> {
    // "Not Found".to_owned()
    let response = http::Response::builder();
    let response = response.status(http::StatusCode::NOT_FOUND);
    response.body(vec![]).unwrap()
    // Ok(Response::new(StatusCode::NotFound))
}

async fn method_not_allowed<State: Clone + Send + Sync + 'static>(
    _state: State,
    _req: http::Request<Vec<u8>>,
    _route_params: Vec<Params>,
) -> http::Response<Vec<u8>> {
    // "Method Not Allowed".to_owned()
    // Ok(Response::new(StatusCode::NotFound))
    let response = http::Response::builder();
    let response = response.status(http::StatusCode::NOT_IMPLEMENTED);
    response.body(vec![]).unwrap()
}
