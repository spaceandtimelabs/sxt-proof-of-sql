# Running benchmarks

## Jaeger benchmarking

To run benchmarks with Jaeger, you need to do the following

1. Spin up Jaeger service on port 6831 to receive the benchmarks trace data, and provides Jaeger UI on port 16686.
    ```bash
    docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:1.62.0
    ```
2. Run a benchmark.
    ```bash
    cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
    cargo bench -p proof-of-sql --bench jaeger_benches Dory
    cargo bench -p proof-of-sql --bench jaeger_benches DynamicDory
    cargo bench -p proof-of-sql --bench jaeger_benches HyperKZG
    ```
3. Navigate to http://localhost:16686/ to see the results.
4. To end the Jaeger service, run
    ```bash
    docker kill jaeger
    ```

### Memory logging (optional)

Jaeger benchmarks default to logging any traces at `DEBUG` level and above. Memory consumption is logged at `TRACE` level. In order to capture memory consumption in the Jaeger benchmarks, add `RUST_LOG=trace` to the command.

Example
```
RUST_LOG=trace cargo bench -p proof-of-sql --bench jaeger_benches DynamicDory
```

## Criterion benchmarking

To run benchmarks with Criterion, you need to do the following

1. Run the benchmarks. (Warning: this takes a very long time.)
    ```bash
    cargo bench -p proof-of-sql --bench criterion_benches
    ```
2. Navigate to `target/criterion/report/index.html` to see the results.