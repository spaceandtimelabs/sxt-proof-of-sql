//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest
//! cargo bench -p proof-of-sql --bench jaeger_benches
//! ```
//! Then, navigate to http://localhost:16686 to view the traces.

use blitzar::{compute::init_backend, proof::InnerProductProof};
mod scaffold;
use crate::scaffold::querys::QUERIES;
use scaffold::jaeger_scaffold;

const SIZE: usize = 1_000_000;

fn main() {
    init_backend();
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("benches")
        .install_simple()
        .unwrap();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(opentelemetry)
        .try_init()
        .unwrap();
    {
        // Run 3 times to ensure that warm-up of the GPU has occured.
        for _ in 0..3 {
            for (title, query, columns) in QUERIES.iter() {
                jaeger_scaffold::<InnerProductProof>(title, query, columns, SIZE, &(), &());
            }
        }
    }
    opentelemetry::global::shutdown_tracer_provider();
}
