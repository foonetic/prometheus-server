/// Launches a local prometheus server.
use prometheus_server;

use lazy_static::lazy_static;
use prometheus::{register_int_gauge, IntGauge};

lazy_static! {
    pub static ref CUSTOM_GAUGE: IntGauge = register_int_gauge!(
        "the_answer",
        "the answer to life, the universe, and everything"
    )
    .unwrap();
}

#[tokio::main]
async fn main() {
    CUSTOM_GAUGE.set(42);
    prometheus_server::launch_server(
        tokio::runtime::Handle::current(),
        "0.0.0.0:8080".parse().unwrap(),
    )
    .await
    .unwrap()
    .unwrap();
}
