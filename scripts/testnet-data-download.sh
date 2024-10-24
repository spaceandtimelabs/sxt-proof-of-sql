cd /testnet-data/parquets
rm -r v2/ETHEREUM
az storage blob download-batch --source ops-publicblocks2hnsenabled-testnet-stct-wus2 --account-name opspublicblocks2hnsenabl --pattern v2/ETHEREUM/SQL_ETHEREUM_$1/**.parquet --destination .
cd /home/stuart.white/sxt-node
cargo run --bin parquet-to-clean-parquet -- /testnet-data/parquets/v2/ETHEREUM
mv /testnet-data/parquets/v2/ETHEREUM/* /testnet-data/parquets/v2-cleaned-no-commit/ETHEREUM/.
