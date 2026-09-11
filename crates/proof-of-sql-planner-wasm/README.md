# Proof of SQL planner WASM

This crate exposes the Proof of SQL planner to JavaScript. Callers provide one SQL statement and
the schemas of its referenced SXT tables. SQL and planner failures are returned as structured
validation results. The WebAssembly boundary accepts and returns JSON strings; malformed input is
reported with the same structured result contract.

```js
import init, { validateQuery } from "./pkg/proof_of_sql_planner_wasm.js";

await init();

const inputJson = JSON.stringify({
  sql: "SELECT DISTINCT ID FROM APP.T",
  schemas: {
    "APP.T": [{ name: "ID", dataType: "BIGINT" }],
  },
});
const result = JSON.parse(validateQuery(inputJson));

if (!result.ok) {
  console.log(result.error.code, result.error.message);
}
```

Table references must use `NAMESPACE.TABLE` form. Supported schema type strings are `BOOLEAN`,
`TINYINT`, `SMALLINT`, `INT`/`INTEGER`, `BIGINT`, `VARCHAR`, `BINARY`, `TIMESTAMP`, and
`DECIMAL(precision, scale)`.

## Build and test

```bash
cargo test -p proof-of-sql-planner-wasm
wasm-pack test --node crates/proof-of-sql-planner-wasm
wasm-pack build --release --target web crates/proof-of-sql-planner-wasm
```
