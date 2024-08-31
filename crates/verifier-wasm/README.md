# Build Verifier into Wasm

Build the verifier into a Wasm binary. The Wasm binary will export a `verify` function that can be called from JavaScript.

## Install Prerequisites

Enable the Wasm build target:
```
$ rustup target add wasm32-unknown-unknown
```

Install `wasm-pack`:
```
$ cargo install wasm-pack
```

## Build the verifier into Wasm

Build a Wasm version of the verifier:
```
$ CARGO_PROFILE_RELEASE_OPT_LEVEL=z wasm-pack build --release --target deno --no-typescript --no-pack ./crates/verifier-wasm
$ rm -rf ./verifier-wasm-artifacts
$ mv ./crates/verifier-wasm/pkg ./verifier-wasm-artifacts
$ rm verifier-wasm-artifacts/.gitignore
```

Setting the environment variable `CARGO_PROFILE_RELEASE_OPT_LEVEL=z` is equivalent to defining `opt-level = "z"` under `[profile.release]` in Cargo.toml. See the [Cargo Reference](https://doc.rust-lang.org/nightly/cargo/reference/environment-variables.html) for a list of supported environment variables.

## (Optional) Check the Size of the Wasm

```
$ du -h verifier-wasm-artifacts/*.wasm
```

## Overview of Build Output

These artifacts were created in the `./verifier-wasm-artifacts` folder:
- `verifier_wasm_bg.wasm` is the Wasm binary with the verifier
- `verifier_wasm.js` is JavaScript glue code that lets us call the `verify()` function that is in the Wasm

## How to Run the Wasm

See the [verifier-wasm-tester](../verifier-wasm-tester/README.md) crate for the steps to run and test the Wasm.

## (Appendix) Alternative Build Steps: Build Without `wasm-pack`

`wasm-pack` uses Cargo, `wasm-bindgen` and `wasm-opt` to build and optimize the Wasm binary. This section shows how to build the Wasm using these tools directly instead of using `wasm-pack`.

Install `wasm-bindgen` command-line tool (the version must be the same as the version of the `wasm-bindgen` library with which the Wasm will be built):
```
$ cargo install --force wasm-bindgen-cli --version 0.2.93
```
You must build the Wasm with the exact same version of the `wasm-bindgen` library (use `=` in Cargo.toml: `wasm-bindgen = { version = "=0.2.93" }`). Otherwise, the `wasm-bindgen` CLI could return an error about the Wasm having a mismatching version of `wasm-bindgen`.

Install `wasm-opt`:
```
$ cargo install --force wasm-opt
```

Build a Wasm version of the verifier:
```
$ CARGO_PROFILE_RELEASE_OPT_LEVEL=z cargo build -p verifier-wasm --release --no-default-features --target wasm32-unknown-unknown
$ wasm-bindgen --target deno --no-typescript ./target/wasm32-unknown-unknown/release/verifier_wasm.wasm --out-dir ./verifier-wasm-artifacts
```

Optimize the Wasm binary to reduce its size:
```
$ mv ./verifier-wasm-artifacts/verifier_wasm_bg.wasm ./verifier-wasm-artifacts/tmp.wasm
$ wasm-opt ./verifier-wasm-artifacts/tmp.wasm -o ./verifier-wasm-artifacts/verifier_wasm_bg.wasm -O
$ rm ./verifier-wasm-artifacts/tmp.wasm
```
