# PoSQL SQL Syntax

Proof of SQL currently supports the following syntax. The syntax support is rapidly expanding, and we are happy to take suggestions about what should be added. Anyone submitting a PR must ensure that this is kept up to date.

```
SELECT [* | expression [ [ AS ] output_name ] [, …]]
FROM table
[WHERE condition]
[GROUP BY expression]
[ORDER BY expression [ASC | DESC]]
[LIMIT count]
[OFFSET start]
```
## Supported in the Prover
* DataTypes
    - Bool / Boolean
    - Numeric Types
        * TinyInt (8 bits)
        * SmallInt (16 bits)
        * Int / Integer (32 bits)
        * BigInt (64 bits)
        * Int128
        * Decimal75
    - Character Types
        * Varchar [^1]
    - Date / Time Types
        * Timestamp
    - Binary Types
        * FixedSizeBinary
* Operators
    - Logical Operators
        * AND, OR
        * NOT
    - Numerical Operators
        * +, -, *
    - Comparison Operators
        * =, !=
        * \>, >=, <, <=
* Aggregate Functions
    - SUM
    - COUNT
* SELECT syntax
    - WHERE clause
    - GROUP BY clause
## Currently Only Supported in Post-Processing

Note: this post-processing is still trustworthy because it is done by the verifier after verifying the result. The prime example of why this is valuable is for the query `SELECT SUM(price) / COUNT(price) FROM table`.
It is far more efficient for the verifier to compute the actual division, while the prover produces a proof for the `SUM` and `COUNT`. While we plan to support `/` in the prover soon, we will still defer to post-processing when it is possible, cheap enough for the verifier, and more efficient overall.

* Operators
    - Numerical Operators
        * /
    - Aggregate Functions
        * MAX, MIN
        * FIRST
* SELECT syntax
    - ORDER BY clause
    - LIMIT clause
    - OFFSET clause

[^1]: Currently, we do not support any string operations beyond = and !=.