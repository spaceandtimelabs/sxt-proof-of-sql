# Proof of SQL Syntax
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
    - Boolean
    - Numeric Types
        * SmallInt / Int16
        * Int / Int32
        * BigInt / Int64
        * Int128
        * Decimal75
    - Character Types
        * Varchar [^1]
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
## Only Supported in Post-Processing
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