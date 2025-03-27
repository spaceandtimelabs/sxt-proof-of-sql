#![allow(clippy::cast_possible_wrap)]
use super::OptionalRandBound;
use proof_of_sql::base::database::ColumnType;

/// Type alias for a single column definition in a query.
type ColumnDefinition = (&'static str, ColumnType, OptionalRandBound);

/// Type alias for a single query entry in the `QUERIES` constant.
pub type QueryEntry = (&'static str, &'static str, &'static [ColumnDefinition]);

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
const ARITHMETIC_SQL: &str =
    "SELECT a + b as r0, a * b - 2 as r1, c FROM table WHERE a <= b AND a >= 0";
const ARITHMETIC_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "b",
        ColumnType::TinyInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("c", ColumnType::VarChar, None),
];
const GROUPBY_TITLE: &str = "Group By";
const GROUPBY_SQL: &str =
    "SELECT a, COUNT(*) FROM table WHERE (c = TRUE) and (a <= b) and (a > 0) GROUP BY a";
const GROUPBY_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::Int128,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "b",
        ColumnType::TinyInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("c", ColumnType::Boolean, None),
];
const AGGREGATE_TITLE: &str = "Aggregate";
const AGGREGATE_SQL: &str = "SELECT SUM(a) FROM table WHERE b = a OR c = 'yz'";
const AGGREGATE_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "b",
        ColumnType::Int,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("c", ColumnType::VarChar, None),
];
const BOOLEAN_FILTER_TITLE: &str = "Boolean Filter";
const BOOLEAN_FILTER_SQL: &str = "SELECT * FROM table WHERE c = TRUE and b = 'xyz' or a = 0";
const BOOLEAN_FILTER_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    (
        "a",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("b", ColumnType::VarChar, None),
    ("c", ColumnType::Boolean, None),
];
const LARGE_COLUMN_SET_TITLE: &str = "Large Column Set";
const LARGE_COLUMN_SET_SQL: &str = "SELECT * FROM table WHERE b = d";
const LARGE_COLUMN_SET_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
    ("a", ColumnType::Boolean, None),
    (
        "b",
        ColumnType::TinyInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "c",
        ColumnType::SmallInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "d",
        ColumnType::Int,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "e",
        ColumnType::BigInt,
        Some(|size| (size / 10).max(10) as i64),
    ),
    (
        "f",
        ColumnType::Int128,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("g", ColumnType::VarChar, None),
    ("h", ColumnType::Scalar, None),
];
const COMPLEX_CONDITION_TITLE: &str = "Complex Condition";
const COMPLEX_CONDITION_SQL: &str =
    "SELECT * FROM table WHERE (a > c * c AND b < c + 10) OR (d = 'xyz')";
const COMPLEX_CONDITION_COLUMNS: &[(&str, ColumnType, OptionalRandBound)] = &[
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
    (
        "c",
        ColumnType::Int128,
        Some(|size| (size / 10).max(10) as i64),
    ),
    ("d", ColumnType::VarChar, None),
];

pub const QUERIES: &[QueryEntry] = &[
    (
        SINGLE_COLUMN_FILTER_TITLE,
        SINGLE_COLUMN_FILTER_SQL,
        SINGLE_COLUMN_FILTER_COLUMNS,
    ),
    (
        MULTI_COLUMN_FILTER_TITLE,
        MULTI_COLUMN_FILTER_SQL,
        MULTI_COLUMN_FILTER_COLUMNS,
    ),
    (ARITHMETIC_TITLE, ARITHMETIC_SQL, ARITHMETIC_COLUMNS),
    (GROUPBY_TITLE, GROUPBY_SQL, GROUPBY_COLUMNS),
    (AGGREGATE_TITLE, AGGREGATE_SQL, AGGREGATE_COLUMNS),
    (
        BOOLEAN_FILTER_TITLE,
        BOOLEAN_FILTER_SQL,
        BOOLEAN_FILTER_COLUMNS,
    ),
    (
        LARGE_COLUMN_SET_TITLE,
        LARGE_COLUMN_SET_SQL,
        LARGE_COLUMN_SET_COLUMNS,
    ),
    (
        COMPLEX_CONDITION_TITLE,
        COMPLEX_CONDITION_SQL,
        COMPLEX_CONDITION_COLUMNS,
    ),
];

/// Retrieves a single query from the `QUERIES` constant by its title.
///
/// # Arguments
/// * `title` - The title of the query to retrieve.
///
/// # Returns
/// * `Some((&str, &str, &[(&str, ColumnType, OptionalRandBound)]))` if the query is found.
/// * `None` if no query with the given title exists.
#[allow(dead_code)]
pub fn get_query(title: &str) -> Option<QueryEntry> {
    QUERIES
        .iter()
        .find(|(query_title, _, _)| *query_title == title)
        .copied()
}
