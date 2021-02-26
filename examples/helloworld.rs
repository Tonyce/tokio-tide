// use http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
// use tokio::sync::Mutex;

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
    let mut app = tokio_tide::Server::with_state(state);
    let addr = "127.0.0.1:8989"
        .parse()
        .expect("Unable to parse socket address");

    app.at(
        "/helloworld",
        |state: Arc<HashMap<String, String>>, req: http::Request<Vec<u8>>| async move {
            let body_str = std::str::from_utf8(req.body()).unwrap();
            let p: Point = serde_json::from_str(body_str).unwrap();
            println!("{:#?}", req.headers());
            println!("{:?}", p);
            println!("{:?}", state.get("Daniel"));
            "hellowrold\n".to_string()
        },
    );
    app.listen(&addr).await.unwrap();
}
