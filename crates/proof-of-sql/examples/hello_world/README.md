# Proof of SQL "Hello World"

This example demonstrates generating and verifying a proof of the query `SELECT b FROM table WHERE a = 2` for the table:

|     a      |      b      |
|------------|-------------|
|     1      |     hi      |
|     2      |    hello    |
|     3      |    there    |
|     2      |    world    |

#### Run

```bash
cargo run --example hello_world 
```

> [!NOTE]
> To run this example without the `blitzar` (i.e CPU only) feature:
> ```bash
> cargo run --example hello_world --no-default-features --features="test rayon"
> ```

#### Output

```
Warming up GPU... 520.959485ms
Loading data... 3.229767ms
Parsing Query... 1.870256ms
Generating Proof... 467.45371ms
Verifying Proof... 7.106864ms
Valid proof!
Query result: OwnedTable { table: {Identifier { name: "b" }: VarChar(["hello", "world"])} }
```
