# How to Run and Test the Wasm Build of the Verifier

This crate is used to generate inputs for the Wasm version of the verifier.

See the [verifier-wasm](../verifier-wasm/README.md) crate for the steps to build the Wasm version of the verifier first.

## Install Prerequisites

Install the Deno JavaScript/Wasm Runtime:
```
$ curl -fsSL https://deno.land/install.sh | sh
```

## Generate Test Inputs

Generate test inputs for the Wasm `verify()` function:
```
$ cargo run -p verifier-wasm-tester -- ./verifier-wasm-inputs
```

## Run the Wasm

```
$ deno run --allow-read crates/verifier-wasm-tester/js/run_verifier_wasm.js ./verifier-wasm-inputs
```

The JavaScript function should print "Verification SUCCESS".