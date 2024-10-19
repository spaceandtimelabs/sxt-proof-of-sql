use crate::base::map::IndexMap;
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use sqlparser::{
    ast::{DataType, ExactNumberInfo, Statement},
    dialect::GenericDialect,
    parser::Parser,
};

#[allow(clippy::cast_possible_truncation)]
/// Parse a DDL file and return a map of table names to bigdecimal columns
///
/// # Panics
/// Panics if there is an error parsing the SQL
#[must_use]
pub fn find_bigdecimals(queries: &str) -> IndexMap<String, Vec<(String, u8, i8)>> {
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, queries).expect("Failed to parse SQL");
    // Find all `CREATE TABLE` statements
    ast.iter()
        .filter_map(|statement| match statement {
            Statement::CreateTable { name, columns, .. } => {
                // Find all `DECIMAL` columns where precision > 38
                // Find the table name
                // Add the table name and column name to the map
                let str_name = name.to_string();
                let big_decimal_specs: Vec<(String, u8, i8)> = columns
                    .iter()
                    .filter_map(|column_def| match column_def.data_type {
                        DataType::Decimal(ExactNumberInfo::PrecisionAndScale(precision, scale))
                            if precision > 38 =>
                        {
                            Some((column_def.name.to_string(), precision as u8, scale as i8))
                        }
                        _ => None,
                    })
                    .collect();
                Some((str_name, big_decimal_specs))
            }
            _ => None,
        })
        .collect::<IndexMap<String, Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_bigdecimals() {
        let sql = "CREATE TABLE IF NOT EXISTS ETHEREUM.BLOCKS(
            BLOCK_NUMBER BIGINT NOT NULL,
            TIME_STAMP TIMESTAMP,
            BLOCK_HASH VARCHAR,
            MINER VARCHAR,
            REWARD DECIMAL(78, 0),
            SIZE_ INT,
            GAS_USED INT,
            GAS_LIMIT INT,
            BASE_FEE_PER_GAS DECIMAL(78, 0),
            TRANSACTION_COUNT INT,
            PARENT_HASH VARCHAR,
            PRIMARY KEY(BLOCK_NUMBER)
          );
          
        CREATE TABLE IF NOT EXISTS ETHEREUM.BLOCK_DETAILS(
            BLOCK_NUMBER BIGINT NOT NULL,
            TIME_STAMP TIMESTAMP,
            SHA3_UNCLES VARCHAR,
            STATE_ROOT VARCHAR,
            TRANSACTIONS_ROOT VARCHAR,
            RECEIPTS_ROOT VARCHAR,
            UNCLES_COUNT INT,
            VERSION VARCHAR,
            LOGS_BLOOM VARCHAR,
            NONCE VARCHAR,
            PRIMARY KEY(BLOCK_NUMBER)
        );";
        let bigdecimals = find_bigdecimals(sql);
        assert_eq!(
            bigdecimals.get("ETHEREUM.BLOCKS").unwrap(),
            &[
                ("REWARD".to_string(), 78, 0),
                ("BASE_FEE_PER_GAS".to_string(), 78, 0)
            ]
        );
        let empty_vec: Vec<(String, u8, i8)> = vec![];
        assert_eq!(
            bigdecimals.get("ETHEREUM.BLOCK_DETAILS").unwrap(),
            &empty_vec
        );
    }
}
