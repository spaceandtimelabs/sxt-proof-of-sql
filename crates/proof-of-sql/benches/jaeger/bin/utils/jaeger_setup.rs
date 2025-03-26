//! # Jaeger Setup Module
//!
//! This module provides functionality to set up Jaeger tracing for benchmarks.
//! Jaeger is a distributed tracing system that helps in monitoring and troubleshooting
//! performance issues in distributed systems. This module integrates Jaeger with the
//! `tracing` crate to enable tracing for Rust applications.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Sets up Jaeger tracing for the benchmarks.
///
/// This function initializes a Jaeger tracer using the `opentelemetry_jaeger` crate and
/// integrates it with the `tracing` crate. It configures the tracing subscriber to use
/// the Jaeger tracer and applies an environment filter for log levels.
///
/// ### Returns
/// - `Ok(())` if the tracing setup is successful.
/// - `Err(Box<dyn std::error::Error>)` if an error occurs during setup.
///
/// ### Panics
///
/// This function panics if the tracing subscriber fails to initialize.
pub fn setup_jaeger_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("benches")
        .install_simple()
        .unwrap();

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("DEBUG"));

    Ok(tracing_subscriber::registry()
        .with(opentelemetry)
        .with(filter)
        .try_init()?)
}

/// Stops Jaeger tracing for the benchmarks.
///
/// This function shuts down the global tracer provider for Jaeger tracing.
pub fn stop_jaeger_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
