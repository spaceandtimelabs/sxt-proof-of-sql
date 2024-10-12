cd crates/proof-of-sql/examples/posql_db
cargo run --features="arrow blitzar" --example posql_db create -t sxt.table -c a,b -d BIGINT,VARCHAR
cargo run --features="arrow blitzar" --example posql_db append -t sxt.table -f hello_world.csv
cargo run --features="arrow blitzar" --example posql_db prove -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
cargo run --features="arrow blitzar" --example posql_db verify -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
