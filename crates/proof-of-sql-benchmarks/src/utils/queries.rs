#![expect(clippy::cast_possible_wrap)]
use super::OptionalRandBound;
use proof_of_sql::base::database::ColumnType;

/// Type alias for a single column definition in a query.
type ColumnDefinition = (&'static str, ColumnType, OptionalRandBound);

/// Type alias for a single query entry.
pub type QueryEntry = (&'static str, &'static str, &'static [ColumnDefinition]);

/// Trait for defining a base query.
pub trait BaseEntry {
    fn title(&self) -> &'static str;
    fn sql(&self) -> &'static str;
    fn columns(&self) -> &'static [ColumnDefinition];
    fn entry(&self) -> QueryEntry {
        (self.title(), self.sql(), self.columns())
    }
}

/// Single column filter query.
pub struct SingleColumnFilter;
impl BaseEntry for SingleColumnFilter {
    fn title(&self) -> &'static str {
        "Single Column Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT b FROM table WHERE a = 0"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("b", ColumnType::VarChar, None),
        ]
    }
}

/// Multi-column filter query.
pub struct MultiColumnFilter;
impl BaseEntry for MultiColumnFilter {
    fn title(&self) -> &'static str {
        "Multi Column Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM table WHERE ((a = 0) or (b = 1)) and (not (c = 'a'))"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
        ]
    }
}

/// Arithmetic query.
pub struct Arithmetic;
impl BaseEntry for Arithmetic {
    fn title(&self) -> &'static str {
        "Arithmetic"
    }

    fn sql(&self) -> &'static str {
        "SELECT a + b as r0, a * b - 2 as r1, c FROM table WHERE a <= b AND a >= 0"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
        ]
    }
}

/// Group by query.
pub struct GroupBy;
impl BaseEntry for GroupBy {
    fn title(&self) -> &'static str {
        "Group By"
    }

    fn sql(&self) -> &'static str {
        "SELECT a, COUNT(*) FROM table WHERE (c = TRUE) and (a <= b) and (a > 0) GROUP BY a"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
        ]
    }
}

/// Aggregate query.
pub struct Aggregate;
impl BaseEntry for Aggregate {
    fn title(&self) -> &'static str {
        "Aggregate"
    }

    fn sql(&self) -> &'static str {
        "SELECT SUM(a) FROM table WHERE b = a OR c = 'yz'"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
        ]
    }
}

/// Boolean filter query.
pub struct BooleanFilter;
impl BaseEntry for BooleanFilter {
    fn title(&self) -> &'static str {
        "Boolean Filter"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM table WHERE c = TRUE and b = 'xyz' or a = 0"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
            (
                "a",
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("b", ColumnType::VarChar, None),
            ("c", ColumnType::Boolean, None),
        ]
    }
}

/// Large column entry query.
pub struct LargeColumnSet;
impl BaseEntry for LargeColumnSet {
    fn title(&self) -> &'static str {
        "Large Column Set"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM table WHERE b = d"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
            ("g", ColumnType::VarChar, None),
            ("h", ColumnType::Scalar, None),
        ]
    }
}

/// Complex condition query.
pub struct ComplexCondition;
impl BaseEntry for ComplexCondition {
    fn title(&self) -> &'static str {
        "Complex Condition"
    }

    fn sql(&self) -> &'static str {
        "SELECT * FROM table WHERE (a > c * c AND b < c + 10) OR (d = 'xyz')"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
        ]
    }
}

/// Sum Count query.
pub struct SumCount;
impl BaseEntry for SumCount {
    fn title(&self) -> &'static str {
        "Sum Count"
    }

    fn sql(&self) -> &'static str {
        "SELECT SUM(a*b*c) as foo, SUM(a*b) as bar, COUNT(1) FROM table WHERE a = 0 OR c-b = 2 AND d = 'a'"
    }

    fn columns(&self) -> &'static [ColumnDefinition] {
        &[
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
                ColumnType::BigInt,
                Some(|size| (size / 10).max(10) as i64),
            ),
            ("d", ColumnType::VarChar, None),
        ]
    }
}

/// Retrieves all available queries.
pub fn all_queries() -> Vec<QueryEntry> {
    vec![
        SingleColumnFilter.entry(),
        MultiColumnFilter.entry(),
        Arithmetic.entry(),
        GroupBy.entry(),
        Aggregate.entry(),
        BooleanFilter.entry(),
        LargeColumnSet.entry(),
        ComplexCondition.entry(),
        SumCount.entry(),
    ]
}

/// Retrieves a single query by its title.
///
/// # Arguments
/// * `title` - The title of the query to retrieve.
///
/// # Returns
/// * `Some<QueryEntry>` if the query is found.
/// * `None` if no query with the given title exists.
pub fn get_query(title: &str) -> Option<QueryEntry> {
    all_queries()
        .into_iter()
        .find(|(query_title, _, _)| *query_title == title)
}
