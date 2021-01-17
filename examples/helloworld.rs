use http::Request;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: Option<i32>,
    y: Option<i32>,
    a: String,
}

#[tokio::main]
async fn main() {
    let mut app = tokio_tide::Server::new();
    let addr = "127.0.0.1:8989"
        .parse()
        .expect("Unable to parse socket address");

    app.at("/helloworld", |req: Request<Vec<u8>>| async move {
        let body_str = std::str::from_utf8(req.body()).unwrap();
        let p: Point = serde_json::from_str(body_str).unwrap();
        println!("{:#?}", req.headers());
        println!("{:?}", p);
        "hellowrold\n".to_string()
    });
    app.listen(&addr).await.unwrap();
}
