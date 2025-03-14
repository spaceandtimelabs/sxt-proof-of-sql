# Null Arithmetic Example

This example demonstrates how SQL handles NULL values in arithmetic operations, specifically in WHERE clauses.

## Dataset

The dataset contains a simple table with three columns (A, B, C) where some values in columns A and B are NULL:

| A    | B    | C   |
|------|------|-----|
| 1    | 1    | 101 |
| 1    | NULL | 102 |
| NULL | 1    | 103 |
| NULL | NULL | 104 |
| 2    | 2    | 105 |
| 2    | NULL | 106 |
| NULL | 2    | 107 |

## Queries

The example runs the following queries:

1. `SELECT * FROM tab` - Shows all data in the table
2. `SELECT * FROM tab WHERE A + B = 2` - Should return only the row where A=1 and B=1
3. `SELECT * FROM tab WHERE A + B = 4` - Should return only the row where A=2 and B=2
4. `SELECT * FROM tab WHERE A + B = 10` - Should return an empty result

## Expected Behavior

In SQL, when NULL values are involved in arithmetic operations, the result is NULL. When comparing NULL to any value (including NULL) using operators like =, >, <, etc., the result is UNKNOWN (not TRUE or FALSE).

In a WHERE clause, only rows that evaluate to TRUE are included in the result. Rows that evaluate to FALSE or UNKNOWN are excluded.

Therefore:
- For `A + B = 2`, only the row where A=1 and B=1 is returned because 1+1=2
- For `A + B = 4`, only the row where A=2 and B=2 is returned because 2+2=4
- Any row where either A or B is NULL will be excluded from the results because A+B evaluates to NULL, and NULL = any_number evaluates to UNKNOWN

## Running the Example

```bash
cargo run --release --example null_arithmetic
```

Or, if you don't have GPU drivers:

```bash
cargo run --release --example null_arithmetic --no-default-features --features="arrow cpu-perf"
``` 