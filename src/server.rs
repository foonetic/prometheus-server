use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use prometheus::{register_int_counter, Encoder, Error, IntCounter, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;

lazy_static! {
    pub static ref METRICS_QUERY_COUNT: IntCounter = register_int_counter!(
        "metrics_query_count",
        "total number of times /metrics has been queried"
    )
    .unwrap();
}

/// Launches an HTTP server at the given address that serves prometheus metrics
/// on the /metrics path. Returns a channel that communicates the server result.
pub fn launch_server(
    runtime: tokio::runtime::Handle,
    address: SocketAddr,
) -> tokio::task::JoinHandle<Result<(), hyper::Error>> {
    runtime.spawn(async move {
        let make_service =
            make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(metrics_function)) });
        let server = Server::bind(&address);
        let server = server.serve(make_service);
        return server.await;
    })
}

/// Returns the metrics in standard Prometheus format.
fn metrics() -> Result<String, Error> {
    METRICS_QUERY_COUNT.inc();
    let families = prometheus::gather();
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    encoder.encode(&families, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| Error::Msg(e.to_string()))
}

/// Defines the metrics serving function.
async fn metrics_function(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match req.uri().path() {
        "/metrics" => match metrics() {
            Err(err) => Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(err.to_string()))
                .unwrap()),
            Ok(result) => Ok(Response::new(result.into())),
        },

        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()),
    }
}
