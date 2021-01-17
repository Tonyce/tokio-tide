use std::collections::HashMap;

use super::endpoint::DynEndpoint;

pub struct Router<'a, State> {
    method_map: HashMap<&'a str, Box<DynEndpoint<State>>>,
    // method_map: String, // HashMap<http_types::Method, MethodRouter<Box<DynEndpoint<State>>>>,
    all_method_router: String, // MethodRouter<Box<DynEndpoint<State>>>,
}

impl<'a, State> Router<'a, State> {
    pub fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: "state".to_string(),
        }
    }
}
