cd crates/proof-of-sql/examples/posql_db
cargo run --example posql_db create -t sxt.table -c a,b -d BIGINT,VARCHAR
cargo run --example posql_db append -t sxt.table -f hello_world.csv
cargo run --example posql_db prove -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
cargo run --example posql_db verify -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof