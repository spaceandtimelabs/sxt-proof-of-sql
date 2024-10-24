export BLITZAR_PARTITION_WINDOW_WIDTH=14
export NVIDIA_VISIBLE_DEVICES=0
echo $BLITZAR_PARTITION_WINDOW_WIDTH
cd /home/stuart.white/sxt-proof-of-sql/scripts/parquet-to-commitments
cargo run --release --bin parquet-to-commitments -- /testnet-data/parquets/v2-cleaned-no-commit/ETHEREUM /testnet-data/commitments/ETHEREUM
mv /testnet-data/parquets/v2-cleaned-no-commit/ETHEREUM/* /testnet-data/parquets/v2-cleaned/ETHEREUM/.
