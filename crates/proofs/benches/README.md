# Running benchmarks

## Jaeger benchmarking

To run benchmarks with Jaeger, you need to do the following

1. Spin up Jaeger service on port 6831 to receive the benchmarks trace data, and provides Jaeger UI on port 16686.
    ```bash
    docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest
    ```
2. Run the benchmark.
    ```bash
    cargo bench -p proofs --bench jaeger_benches
    ```
3. Navigate to http://localhost:16686/ to see the results.
4. To end the Jaeger service, run
    ```bash
    docker kill jaeger
    ```

## Criterion benchmarking

To run benchmarks with Criterion, you need to do the following

1. Run the benchmarks. (Warning: this takes a very long time.)
    ```bash
    cargo bench -p proofs --bench criterion_benches
    ```
2. Navigate to `target/criterion/report/index.html` to see the results.