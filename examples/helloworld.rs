// use http::Request;
use route_recognizer::{Match, Params, Router as MethodRouter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
// use tokio::sync::Mutex;
use std::future::Future;
use std::pin::Pin;

use tokio_tide::Next;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: Option<i32>,
    y: Option<i32>,
    a: String,
}

#[tokio::main]
async fn main() {
    let mut contacts = HashMap::new();

    contacts.insert("Daniel".to_string(), "798-1364".to_string());

    let state = Arc::new(contacts);
    let mut app = tokio_tide::server::Server::with_state(state);
    let addr = "127.0.0.1:8989"
        .parse()
        .expect("Unable to parse socket address");

    // app.at(
    //     "/helloworld",
    //     |state: Arc<HashMap<String, String>>, req: http::Request<Vec<u8>>| async move {
    //         let body_str = std::str::from_utf8(req.body()).unwrap();
    //         let p: Point = serde_json::from_str(body_str).unwrap();
    //         println!("{:#?}", req.headers());
    //         println!("{:?}", p);
    //         println!("{:?}", state.get("Daniel"));
    //         "hellowrold\n".to_string()
    //     },
    // );

    app.at("/he/:n").get(
        |state: Arc<HashMap<String, String>>,
         _req: http::Request<Vec<u8>>,
         route_params: Vec<Params>| async move {
            //     // let body_str = std::str::from_utf8(req.body()).unwrap();
            //     // let p: Point = serde_json::from_str(body_str).unwrap();
            //     // println!("{:#?}", req.headers());
            //     // println!("{:?}", p);
            println!("{:?}", route_params);
            "hellowrold\n".to_string()
        },
    );
    app.with(test_middleware)
        .with(test_middleware_2)
        .at("/he")
        .get(
            |state: Arc<HashMap<String, String>>,
             req: http::Request<Vec<u8>>,
             _route_params: Vec<Params>| async move {
                //     // let body_str = std::str::from_utf8(req.body()).unwrap();
                //     // let p: Point = serde_json::from_str(body_str).unwrap();
                //     // println!("{:#?}", req.headers());
                //     // println!("{:?}", p);
                let data = req.extensions().get::<&str>();
                println!("{:?}", data);
                "hellowrold\n".to_string()
            },
        );
    println!("Listening on: {}", addr);
    app.listen(&addr).await.unwrap();
}

fn test_middleware<'a, State: Clone + Send + Sync + 'static>(
    state: State,
    mut request: http::Request<Vec<u8>>,
    route_params: Vec<Params>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = String> + Send + 'a>> {
    Box::pin(async {
        println!("middleware");
        // "ok".to_owned();
        let result = request.extensions_mut().insert("hello middleware");
        println!("{:?}", result);
        next.run(state, request, route_params).await
        // if let Some(user) = request.state().find_user().await {
        //     tide::log::trace!("user loaded", {user: user.name});
        //     request.set_ext(user);
        //     Ok(next.run(request).await)
        // // this middleware only needs to run before the endpoint, so
        // // it just passes through the result of Next
        // } else {
        //     // do not run endpoints, we could not find a user
        //     Ok(Response::new(StatusCode::Unauthorized))
        // }
    })
}

fn test_middleware_2<'a, State: Clone + Send + Sync + 'static>(
    state: State,
    mut request: http::Request<Vec<u8>>,
    route_params: Vec<Params>,
    next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = String> + Send + 'a>> {
    Box::pin(async {
        println!("middleware2");
        // "ok".to_owned();
        let result = request.extensions_mut().insert("hello middleware2");
        next.run(state, request, route_params).await
        // if let Some(user) = request.state().find_user().await {
        //     tide::log::trace!("user loaded", {user: user.name});
        //     request.set_ext(user);
        //     Ok(next.run(request).await)
        // // this middleware only needs to run before the endpoint, so
        // // it just passes through the result of Next
        // } else {
        //     // do not run endpoints, we could not find a user
        //     Ok(Response::new(StatusCode::Unauthorized))
        // }
    })
}
