# Running benchmarks

## Jaeger benchmarking

The Jaeger benchmarks/tracing is wrapped by a binary. The motivation of the wrapper is to allow greater control over benchmark parameters. To run benchmarks with Jaeger, you need to do the following

1. Spin up Jaeger service on port 6831 to receive the benchmarks trace data, and provides Jaeger UI on port 16686.
    ```bash
    docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:1.62.0
    ```
2. See all the options to run a benchmark.
    ```bash
    cargo run --release --bin jaeger_benches --features="bench" -- --help
    ```
3. Navigate to http://localhost:16686/ to see the results.
4. To end the Jaeger service, run
    ```bash
    docker kill jaeger
    ```

All the options are outlined in the help and `jaeger_benches.rs` module.

### Example

To run a benchmark on the `HyperKZG` commitment scheme using the `Single Column Filter` query with a table size of `1_000_000` for `3` iterations, your command would be the following.

```bash
cargo run --release --bin jaeger_benches --features="bench" -- --s hyper-kzg -i 3 -t 1000000 -q single-column-filter
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
    cargo bench -p proof-of-sql --bench bench_append_rows --features="test"
    ```
2. Navigate to `target/criterion/report/index.html` to see the results.
