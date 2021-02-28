use crate::middleware::Next;
use crate::route::Route;
use crate::router::{Router, Selection};
use crate::{
    endpoint::{DynEndpoint, Endpoint},
    middleware::Middleware,
};

use bytes::BytesMut;
use futures::SinkExt;
use http::{header::HeaderValue, Request, Response, StatusCode};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::{error::Error, fmt, io};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};

pub struct Server<State> {
    router: Arc<Router<State>>,
    // method_map: Arc<HashMap<&'static str, Box<DynEndpoint<State>>>>,
    state: State,
    middleware: Arc<Vec<Arc<dyn Middleware<State>>>>,
}

impl Server<()> {
    #[must_use]
    pub fn new() -> Self {
        Self::with_state(())
    }
}

impl Default for Server<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> Server<State>
where
    State: Clone + Send + Sync + 'static,
{
    pub fn with_state(state: State) -> Self {
        Self {
            router: Arc::new(Router::new()),
            middleware: Arc::new(vec![
                // #[cfg(feature = "cookies")]
                // Arc::new(cookies::CookiesMiddleware::new()),
                // #[cfg(feature = "logger")]
                // Arc::new(log::LogMiddleware::new()),
            ]),
            state,
        }
    }

    pub async fn listen(self, addr: &SocketAddr) -> Result<(), Box<dyn Error>> {
        let server = TcpListener::bind(addr).await?;
        loop {
            let (stream, _) = server.accept().await?;
            // let method_map = self.router.method_map.clone();
            let state = self.state.clone();
            let router = self.router.clone();
            let middleware = self.middleware.clone();

            tokio::spawn(async move {
                let mut transport = Framed::new(stream, Http);

                while let Some(request) = transport.next().await {
                    // let response = Response::builder();
                    match request {
                        Ok(request) => {
                            let path = request.uri().path();
                            let method = request.method();
                            let Selection { endpoint, params } = router.route(path, method);
                            let route_params = vec![params];

                            let next = Next {
                                endpoint,
                                next_middleware: &middleware,
                            };

                            // let (status, headers, body) =
                            let response = next.run(state.clone(), request, route_params).await;

                            // let res = endpoint.call(state.clone(), request, route_params).await;
                            // let response = response.status(status);

                            // let response = response.he(key, value).body(body).unwrap();

                            transport.send(response).await.unwrap();
                        }
                        Err(_e) => {}
                    }
                }
            });
        }
    }

    // pub fn at(&mut self, path: &'static str, ep: impl Endpoint<State>) {
    // let m = Arc::get_mut(&mut self.method_map).unwrap();
    // let mut map = self.method_map;
    // m.entry(path).or_insert_with(|| Box::new(ep));
    // }

    pub fn with<M>(&mut self, middleware: M) -> &mut Self
    where
        M: Middleware<State>,
    {
        let m = Arc::get_mut(&mut self.middleware)
            .expect("Registering middleware is not possible after the Server has started");
        m.push(Arc::new(middleware));
        self
    }

    pub fn at<'a>(&'a mut self, path: &str) -> Route<'a, State> {
        let router = Arc::get_mut(&mut self.router)
            .expect("Registering routes is not possible after the Server has started");
        Route::new(router, path.to_owned())
    }
}

struct Http;

/// Implementation of encoding an HTTP response into a `BytesMut`, basically
/// just writing out an HTTP/1.1 response.
impl Encoder<Response<Vec<u8>>> for Http {
    type Error = io::Error;

    fn encode(&mut self, item: Response<Vec<u8>>, dst: &mut BytesMut) -> io::Result<()> {
        use std::fmt::Write;
        use std::time::SystemTime;

        let now = SystemTime::now();
        write!(
            BytesWrite(dst),
            "\
             HTTP/1.1 {}\r\n\
             Server: Tokio-Tide\r\n\
             Content-Length: {}\r\n\
             Date: {}\r\n\
             ",
            item.status(),
            item.body().len(),
            httpdate::fmt_http_date(now),
        )
        .unwrap();

        for (k, v) in item.headers() {
            dst.extend_from_slice(k.as_str().as_bytes());
            dst.extend_from_slice(b": ");
            dst.extend_from_slice(v.as_bytes());
            dst.extend_from_slice(b"\r\n");
        }

        dst.extend_from_slice(b"\r\n");
        dst.extend_from_slice(item.body());

        return Ok(());

        // Right now `write!` on `Vec<u8>` goes through io::Write and is not
        // super speedy, so inline a less-crufty implementation here which
        // doesn't go through io::Error.
        struct BytesWrite<'a>(&'a mut BytesMut);

        impl fmt::Write for BytesWrite<'_> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                self.0.extend_from_slice(s.as_bytes());
                Ok(())
            }

            fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
                fmt::write(self, args)
            }
        }
    }
}

/// Implementation of decoding an HTTP request from the bytes we've read so far.
/// This leverages the `httparse` crate to do the actual parsing and then we use
/// that information to construct an instance of a `http::Request` object,
/// trying to avoid allocations where possible.
impl Decoder for Http {
    type Item = Request<Vec<u8>>;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Request<Vec<u8>>>> {
        // TODO: we should grow this headers array if parsing fails and asks for more headers
        let mut headers = [None; 16];
        let (method, path, version, amt) = {
            let mut parsed_headers = [httparse::EMPTY_HEADER; 16];
            let mut r = httparse::Request::new(&mut parsed_headers);
            let status = r.parse(src).map_err(|e| {
                let msg = format!("failed to parse http request: {:?}", e);
                io::Error::new(io::ErrorKind::Other, msg)
            })?;

            let amt = match status {
                httparse::Status::Complete(amt) => amt,
                httparse::Status::Partial => return Ok(None),
            };

            let toslice = |a: &[u8]| {
                let start = a.as_ptr() as usize - src.as_ptr() as usize;
                assert!(start < src.len());
                (start, start + a.len())
            };

            for (i, header) in r.headers.iter().enumerate() {
                let k = toslice(header.name.as_bytes());
                let v = toslice(header.value);
                headers[i] = Some((k, v));
            }

            (
                toslice(r.method.unwrap().as_bytes()),
                toslice(r.path.unwrap().as_bytes()),
                r.version.unwrap(),
                amt,
            )
        };
        if version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "only HTTP/1.1 accepted",
            ));
        }
        let data = src.split_to(amt).freeze();
        let mut ret = Request::builder();
        ret = ret.method(&data[method.0..method.1]);
        let s = data.slice(path.0..path.1);
        let s = unsafe { String::from_utf8_unchecked(Vec::from(s.as_ref())) };
        ret = ret.uri(s);
        ret = ret.version(http::Version::HTTP_11);
        for header in headers.iter() {
            let (k, v) = match *header {
                Some((ref k, ref v)) => (k, v),
                None => break,
            };
            let value = HeaderValue::from_bytes(data.slice(v.0..v.1).as_ref())
                .map_err(|_| io::Error::new(io::ErrorKind::Other, "header decode error"))?;
            ret = ret.header(&data[k.0..k.1], value);
        }

        let req = ret
            .body(src[..].to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Some(req))
    }
}
