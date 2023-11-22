To run the benchmarks with Jaeger, you need to follow the next steps:

```bash
# Spin up Jaeger service on port 6831 to receive the benchmarks trace data, and provides Jaeger UI on port 16686.
docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest

# Set tracing env variables (both optional)
export CARGO_LOCK_PATH=Cargo.lock
export JAEGER_AGENT_ENDPOINT=localhost:6831

# Wait two seconds for the Jaeger service to be ready
sleep 2 

# Run the benchmark suite
python3 benches/provable_sql/scripts/run_benches.py --num-samples 5 --force-build 1 --generate-plots 1 --output-dir temp-bench-results/

# After executing the above command, you can access the Jaeger UI to inspect the traces. Follow the next steps to find your traces:
# 1) Go to `http://localhost:16686/`
# 2) Under the `Service` tab (left panel), select the `proofs-benchmark-server` service.
# 3) Then Click on the `Find Traces` button.
# 4) Select the trace you want to inspect. For instance, the `process_query` trace.
#
# After accessing the correct trace, you can inspect the proof creation and verification time.
# Note that there is relevant information under process_query trace `tags` and `process` tabs (such as the executed query and lib versions).

# Kill the Jaeger service
docker kill jaeger
```
