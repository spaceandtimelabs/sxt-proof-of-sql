cd crates/proofs/examples/csv_db
cargo run --example csv_db create -t sxt.table -c a,b -d BIGINT,VARCHAR
cargo run --example csv_db append -t sxt.table -f hello_world.csv
cargo run --example csv_db prove -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
cargo run --example csv_db verify -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof