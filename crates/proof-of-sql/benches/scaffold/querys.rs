#![allow(clippy::cast_possible_wrap)]
use super::OptionalRandBound;
use proof_of_sql::base::database::ColumnType;

const SINGLE_COLUMN_FILTER_TITLE: &str = "Single Column Filter";
const SINGLE_COLUMN_FILTER_SQL: &str = "SELECT b FROM table WHERE a = 0";
const SINGLE_COLUMN_FILTER_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("b", ColumnType::VarChar, None),
];
const MULTI_COLUMN_FILTER_TITLE: &str = "Multi Column Filter";
const MULTI_COLUMN_FILTER_SQL: &str =
    "SELECT * FROM table WHERE ((a = 0) or (b = 1)) and (not (c = 'a'))";
const MULTI_COLUMN_FILTER_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "b",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("c", ColumnType::VarChar, None),
];
const ARITHMETIC_TITLE: &str = "Arithmetic";
const ARITHMETIC_SQL: &str = "SELECT a + b as r0, a * b - 2 as r1, c FROM table WHERE a >= b";
const ARITHMETIC_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "b",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("c", ColumnType::VarChar, None),
];

#[allow(clippy::type_complexity)]
pub const QUERIES: &[(&str, &str, &[(&str, ColumnType, OptionalRandBound)])] = &[
    //(
    //    SINGLE_COLUMN_FILTER_TITLE,
    //    SINGLE_COLUMN_FILTER_SQL,
    //    SINGLE_COLUMN_FILTER_COLUMNS,
    //),
    (
        MULTI_COLUMN_FILTER_TITLE,
        MULTI_COLUMN_FILTER_SQL,
        MULTI_COLUMN_FILTER_COLUMNS,
    ),
    //(ARITHMETIC_TITLE, ARITHMETIC_SQL, ARITHMETIC_COLUMNS),
];
