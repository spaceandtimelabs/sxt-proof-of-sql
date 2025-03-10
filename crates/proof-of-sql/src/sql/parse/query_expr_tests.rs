use super::ConversionError;
use crate::{
    base::{
        database::{ColumnType, TableRef, TestSchemaAccessor},
        map::{indexmap, IndexMap, IndexSet},
    },
    sql::{
        parse::QueryExpr,
        postprocessing::{test_utility::*, PostprocessingError},
        proof_exprs::test_utility::*,
        proof_plans::{test_utility::*, DynProofPlan},
    },
};
use itertools::Itertools;
use proof_of_sql_parser::{
    sql::SelectStatementParser,
    utility::{
        add as padd, aliased_expr, col, count, count_all, lit, max, min, mul as pmul, sub as psub,
        sum,
    },
};
use sqlparser::ast::Ident;

/// # Panics
///
/// Will panic if:
/// - The `parse` method of `SelectStatementParser` fails, causing `unwrap()` to panic.
/// - The `try_new` method of `QueryExpr` fails, causing `unwrap()` to panic.
fn query_to_provable_ast(
    table: &TableRef,
    query: &str,
    accessor: &TestSchemaAccessor,
) -> QueryExpr {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    QueryExpr::try_new(
        intermediate_ast,
        table.schema_id().cloned().unwrap(),
        accessor,
    )
    .unwrap()
}

fn invalid_query_to_provable_ast(table: &TableRef, query: &str, accessor: &TestSchemaAccessor) {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    assert!(QueryExpr::try_new(
        intermediate_ast,
        table.schema_id().cloned().unwrap(),
        accessor
    )
    .is_err());
}

#[cfg(test)]
pub fn schema_accessor_from_table_ref_with_schema(
    table: &TableRef,
    schema: IndexMap<Ident, ColumnType>,
) -> TestSchemaAccessor {
    TestSchemaAccessor::new(indexmap! {table.clone() => schema})
}

#[test]
fn we_can_convert_an_ast_with_one_column() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(3)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_column_and_i128_data() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::Int128,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(3_i64)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_column_and_a_filter_by_a_string_literal() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::VarChar,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab where a = 'abc'", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_varchar("abc")),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_duplicate_aliases() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select a as c, b as c from sxt_tab where a = 3",
        &accessor,
    );
    invalid_query_to_provable_ast(&t, "select a as b, b from sxt_tab where a = 3", &accessor);
    invalid_query_to_provable_ast(
        &t,
        "select a as b, a as b from sxt_tab where a = 3",
        &accessor,
    );
    invalid_query_to_provable_ast(&t, "select a, a from sxt_tab where a = 3", &accessor);
}

#[test]
fn we_dont_have_duplicate_filter_result_expressions() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a as b, a as c from sxt_tab where a = 3",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            aliased_cols_expr_plan(&t, &[("a", "b"), ("a", "c")], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(3)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_two_columns() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
            "c".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a,  b from sxt_tab where c = 123", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a", "b"], &accessor),
            tab(&t),
            equal(column(&t, "c", &accessor), const_bigint(123)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_two_columns_and_arithmetic() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
            "c".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a,  b from sxt_tab where c = a + b - 1",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a", "b"], &accessor),
            tab(&t),
            equal(
                column(&t, "c", &accessor),
                subtract(
                    add(column(&t, "a", &accessor), column(&t, "b", &accessor)),
                    const_bigint(1),
                ),
            ),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_select_star() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "b".into() => ColumnType::BigInt,
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select * from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["b", "a"], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(3)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_positive_cond() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab where b = +4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            equal(column(&t, "b", &accessor), const_bigint(4)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_not_equals_cond() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab where b <> +4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            not(equal(column(&t, "b", &accessor), const_bigint(4))),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_negative_cond() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab where b <= -4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            lte(column(&t, "b", &accessor), const_bigint(-4)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_and() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
            "c".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a from sxt_tab where (b = 3) and (c <= -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            and(
                equal(column(&t, "b", &accessor), const_bigint(3)),
                lte(column(&t, "c", &accessor), const_bigint(-2)),
            ),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_or() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
            "c".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a from sxt_tab where (b * 3 = 3) or (c = -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            or(
                equal(
                    multiply(column(&t, "b", &accessor), const_bigint(3)),
                    const_bigint(3),
                ),
                equal(column(&t, "c", &accessor), const_bigint(-2)),
            ),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_or_not() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
            "c".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a from sxt_tab where (b <= 3) or (not (c >= -2))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            or(
                lte(column(&t, "b", &accessor), const_bigint(3)),
                not(gte(column(&t, "c", &accessor), const_bigint(-2))),
            ),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_not_and_or() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
            "c".into() => ColumnType::BigInt,
            "f".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a, not (a = b or c = f) as boolean from sxt_tab where not (((f >= 45) or (c <= -2)) and (b = 3))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_expr_plan(&t, "a", &accessor),
                aliased_plan(
                    not(or(
                        equal(column(&t, "a", &accessor), column(&t, "b", &accessor)),
                        equal(column(&t, "c", &accessor), column(&t, "f", &accessor)),
                    )),
                    "boolean",
                ),
            ],
            tab(&t),
            not(and(
                or(
                    gte(column(&t, "f", &accessor), const_bigint(45)),
                    lte(column(&t, "c", &accessor), const_bigint(-2)),
                ),
                equal(column(&t, "b", &accessor), const_bigint(3)),
            )),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_min_i128_filter_value_and_const() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a, -170141183460469231731687303715884105728 as b from sxt_tab where a = -170141183460469231731687303715884105728",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_expr_plan(&t, "a", &accessor),
                aliased_plan(const_int128(i128::MIN), "b"),
            ],
            tab(&t),
            equal(column(&t, "a", &accessor), const_int128(i128::MIN)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_max_i128_filter_value_and_const() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a, 170141183460469231731687303715884105727 as ma from sxt_tab where a = 170141183460469231731687303715884105727",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_expr_plan(&t, "a", &accessor),
                aliased_plan(const_int128(i128::MAX), "ma"),
            ],
            tab(&t),
            equal(column(&t, "a", &accessor), const_int128(i128::MAX)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_using_an_aliased_column() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "b".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a as b_rename, a = b as boolean from sxt_tab where b >= +4",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                aliased_col_expr_plan(&t, "a", "b_rename", &accessor),
                aliased_plan(
                    equal(column(&t, "a", &accessor), column(&t, "b", &accessor)),
                    "boolean",
                ),
            ],
            tab(&t),
            gte(column(&t, "b", &accessor), const_bigint(4)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_a_nonexistent_column() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "b".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(&t, "select * from sxt_tab where a = 3", &accessor);
}

#[test]
fn we_cannot_convert_an_ast_with_a_column_type_different_than_equal_literal() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "b".into() => ColumnType::VarChar,
        },
    );
    invalid_query_to_provable_ast(&t, "select * from sxt_tab where b = 123", &accessor);
}

#[test]
fn we_can_convert_an_ast_with_a_schema() {
    let t = TableRef::new("eth", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from eth.sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(3)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_without_any_filter() {
    let t = TableRef::new("eth", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_expr_plan(&t, "a", &accessor),
                aliased_plan(const_bigint(3), "b"),
            ],
            tab(&t),
            const_bool(true),
        ),
        vec![],
    );
    let queries = [
        "select *, 3 as b from eth.sxt_tab",
        "select a, 3 as b from eth.sxt_tab",
    ];
    for query in queries {
        let ast = query_to_provable_ast(&t, query, &accessor);
        assert_eq!(ast, expected_ast);
    }
}

/////////////////////////
/// `OrderBy`
/////////////////////////
#[test]
fn we_can_parse_order_by_with_a_single_column() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "b".into() => ColumnType::BigInt,
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select * from sxt_tab where a = 3 order by b",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["b", "a"], &accessor),
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(3)),
        ),
        vec![orders(&[0_usize], &[true])],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_with_multiple_columns() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "b".into() => ColumnType::BigInt,
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a, b from sxt_tab where a = b + 3 order by b desc, a asc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a", "b"], &accessor),
            tab(&t),
            equal(
                column(&t, "a", &accessor),
                add(column(&t, "b", &accessor), const_bigint(3)),
            ),
        ),
        vec![orders(&[1_usize, 0], &[false, true])],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_referencing_an_alias_associated_with_column_b_but_with_name_equals_column_a_also_renamed(
) {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select salary as s, name as salary from sxt_tab where salary = 5 order by salary desc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                aliased_col_expr_plan(&t, "salary", "s", &accessor),
                aliased_col_expr_plan(&t, "name", "salary", &accessor),
            ],
            tab(&t),
            equal(column(&t, "salary", &accessor), const_bigint(5)),
        ),
        vec![orders(&[1_usize], &[false])],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_order_by_referencing_a_column_name_instead_of_an_alias() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select salary as s from sxt_tab order by salary",
        &accessor,
    );
}

#[test]
fn we_cannot_parse_order_by_referencing_invalid_aliased_expressions() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "b".into() => ColumnType::BigInt,
            "a".into() => ColumnType::BigInt,
        },
    );
    // Note: While this operation is acceptable with PostgreSQL, we do not currently support it.
    invalid_query_to_provable_ast(&t, "select a from sxt_tab order by b desc", &accessor);
    invalid_query_to_provable_ast(&t, "select a as b from sxt_tab order by a desc", &accessor);
    invalid_query_to_provable_ast(&t, "select sum(a) from sxt_tab order by a desc", &accessor);
    invalid_query_to_provable_ast(&t, "select 2 * a from sxt_tab order by a desc", &accessor);
}

#[test]
fn we_cannot_parse_order_by_referencing_an_alias_name_associated_with_two_different_columns() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select salary as s, name as s from sxt_tab order by s desc",
        &accessor,
    );
    invalid_query_to_provable_ast(
        &t,
        "select salary as name, name from sxt_tab order by name desc",
        &accessor,
    );
    // Note: While this is not ambiguous with PostgreSQL,
    // it currently is with our code because there is
    // no way to differentiate between the two columns
    // in the record batch since they share the same name.
    invalid_query_to_provable_ast(
        &t,
        "select salary as name, name from sxt_tab order by salary desc",
        &accessor,
    );
    // Note: This is not ambiguous with PostgreSQL either,
    // but it is with our code for the reasons mentioned above.
    invalid_query_to_provable_ast(
        &t,
        "select salary as s, name as s from sxt_tab order by salary desc",
        &accessor,
    );
}

#[test]
fn we_can_parse_order_by_queries_with_the_same_column_name_appearing_more_than_once_and_with_different_alias_name(
) {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
        },
    );
    for (index, order_by) in [(0_usize, "s"), (2_usize, "d")] {
        let ast = query_to_provable_ast(
            &t,
            &("select salary as s, name, salary as d from sxt_tab order by ".to_owned() + order_by),
            &accessor,
        );
        let expected_ast = QueryExpr::new(
            filter(
                vec![
                    aliased_col_expr_plan(&t, "salary", "s", &accessor),
                    col_expr_plan(&t, "name", &accessor),
                    aliased_col_expr_plan(&t, "salary", "d", &accessor),
                ],
                tab(&t),
                const_bool(true),
            ),
            vec![orders(&[index], &[true])],
        );
        assert_eq!(ast, expected_ast);
    }
}

/////////////////////////
// Slice
/////////////////////////

#[test]
fn we_can_parse_a_query_having_a_simple_limit_clause() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab limit 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![slice(Some(3), Some(0))],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn slice_is_still_applied_when_limit_is_u64_max_and_offset_is_zero() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab offset 0", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![slice(Some(u64::MAX), Some(0))],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_simple_positive_offset_clause() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab offset 7", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![slice(Some(u64::MAX), Some(7))],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_negative_offset_clause() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab offset -7", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![slice(Some(u64::MAX), Some(-7))],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_a_simple_limit_and_offset_clause() {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(&t, "select a from sxt_tab limit 55 offset 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["a"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![slice(Some(55), Some(3))],
    );
    assert_eq!(ast, expected_ast);
}

///////////////////////////
// Composition Expressions
///////////////////////////
#[test]
fn we_can_parse_a_query_having_a_simple_limit_and_offset_clause_preceded_by_where_expr_and_order_by(
) {
    let t = TableRef::new("sxt", "sxt_tab");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "boolean".into() => ColumnType::Boolean,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select a, boolean and a >= 4 as res from sxt_tab where a = -3 order by a desc limit 55 offset 3",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_expr_plan(&t, "a", &accessor),
                aliased_plan(
                    and(
                        column(&t, "boolean", &accessor),
                        gte(column(&t, "a", &accessor), const_bigint(4)),
                    ),
                    "res",
                ),
            ],
            tab(&t),
            equal(column(&t, "a", &accessor), const_bigint(-3)),
        ),
        vec![orders(&[0_usize], &[false]), slice(Some(55), Some(3))],
    );
    assert_eq!(ast, expected_ast);
}

///////////////////////////
// Group By Expressions - Prover
///////////////////////////
#[test]
fn we_can_do_provable_group_by() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select department, sum(salary) as total_salary, count(*) as num_employee from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(&t, &["department"], &accessor),
            vec![sum_expr(column(&t, "salary", &accessor), "total_salary")],
            "num_employee",
            tab(&t),
            const_bool(true),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_do_provable_group_by_without_sum() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select department, count(*) as num_employee from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(&t, &["department"], &accessor),
            vec![],
            "num_employee",
            tab(&t),
            const_bool(true),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_do_provable_group_by_with_two_group_by_columns() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "state".into() => ColumnType::VarChar,
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select state, department, sum(salary) as total_salary, count(*) as num_employee from employees group by state, department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(&t, &["state", "department"], &accessor),
            vec![sum_expr(column(&t, "salary", &accessor), "total_salary")],
            "num_employee",
            tab(&t),
            const_bool(true),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_do_provable_group_by_with_two_sums_and_filter() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "tax".into() => ColumnType::BigInt,
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select department, sum(salary) as total_salary, sum(tax) as total_tax, count(*) as num_employee from employees where tax <= 1 group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        group_by(
            cols_expr(&t, &["department"], &accessor),
            vec![
                sum_expr(column(&t, "salary", &accessor), "total_salary"),
                sum_expr(column(&t, "tax", &accessor), "total_tax"),
            ],
            "num_employee",
            tab(&t),
            lte(column(&t, "tax", &accessor), const_bigint(1)),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

///////////////////////////
// Group By Expressions - Postprocessing
///////////////////////////
#[test]
fn we_can_group_by_without_using_aggregate_functions() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select department, true as is_remote from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![col_expr_plan(&t, "department", &accessor)],
            tab(&t),
            const_bool(true),
        ),
        vec![
            group_by_postprocessing(
                &["department"],
                &[
                    aliased_expr(col("department"), "department"),
                    aliased_expr(lit(true), "is_remote"),
                ],
            ),
            select_expr(&[
                aliased_expr(col("department"), "department"),
                aliased_expr(lit(true), "is_remote"),
            ]),
        ],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn group_by_expressions_are_parsed_before_an_order_by_referencing_an_aggregate_alias_result() {
    let query_text =
        "select max(salary) max_sal, department_budget d, count(department_budget) from sxt.employees group by department_budget, tax order by max_sal";

    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "department_budget".into() => ColumnType::BigInt,
            "salary".into() => ColumnType::BigInt,
            "tax".into() => ColumnType::BigInt,
        },
    );

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let query =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_query = QueryExpr::new(
        filter(
            vec![
                col_expr_plan(&t, "department_budget", &accessor),
                col_expr_plan(&t, "salary", &accessor),
                col_expr_plan(&t, "tax", &accessor),
            ],
            tab(&t),
            const_bool(true),
        ),
        vec![
            group_by_postprocessing(
                &["department_budget", "tax"],
                &[
                    aliased_expr(max(col("salary")), "max_sal"),
                    aliased_expr(col("department_budget"), "d"),
                    aliased_expr(count(col("department_budget")), "__count__"),
                ],
            ),
            orders(&[0_usize], &[true]),
        ],
    );
    assert_eq!(query, expected_query);
}

#[test]
fn we_cannot_parse_non_aggregated_or_non_group_by_columns_in_the_select_clause() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select department, salary from sxt.employees group by department",
        &accessor,
    );
}

#[test]
fn alias_references_are_not_allowed_in_the_group_by() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select department, min(salary) as min_salary from employees group by min_salary",
        &accessor,
    );
    invalid_query_to_provable_ast(
        &t,
        "select salary as min_salary from employees group by min_salary",
        &accessor,
    );
}

#[test]
fn order_by_cannot_reference_an_invalid_group_by_column() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select department as d from sxt.employees group by department order by department",
        &accessor,
    );
    invalid_query_to_provable_ast(
        &t,
        "select department, min(salary) from sxt.employees group by department order by salary",
        &accessor,
    );
}

#[test]
fn group_by_column_cannot_be_a_column_result_alias() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select min(salary) as min_sal from sxt.employees group by min_sal",
        &accessor,
    );
}

#[test]
fn we_can_have_aggregate_functions_without_a_group_by_clause() {
    let query_text = "select count(name) from sxt.employees";
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "name".into() => ColumnType::VarChar,
        },
    );

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let ast =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_ast = QueryExpr::new(
        group_by(vec![], vec![], "__count__", tab(&t), const_bool(true)),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_query_having_group_by_with_the_same_name_as_the_aggregation_expression() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
            "bonus".into() => ColumnType::VarChar,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select count(bonus) department from sxt.employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["bonus", "department"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["department"],
            &[aliased_expr(count(col("bonus")), "department")],
        )],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn count_aggregate_functions_can_be_used_with_non_numeric_columns() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
            "bonus".into() => ColumnType::VarChar,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select department, count(bonus), count(department) as dep from sxt.employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["bonus", "department"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["department"],
            &[
                aliased_expr(col("department"), "department"),
                aliased_expr(count(col("bonus")), "__count__"),
                aliased_expr(count(col("department")), "dep"),
            ],
        )],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn count_all_uses_the_first_group_by_identifier_as_default_result_column() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
            "bonus".into() => ColumnType::VarChar,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select count(*) from sxt.employees where salary = 4 group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["department", "salary"], &accessor),
            tab(&t),
            equal(column(&t, "salary", &accessor), const_bigint(4)),
        ),
        vec![group_by_postprocessing(
            &["department"],
            &[aliased_expr(count_all(), "__count__")],
        )],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn aggregate_result_columns_cannot_reference_invalid_columns() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
            "bonus".into() => ColumnType::VarChar,
        },
    );
    invalid_query_to_provable_ast(
        &t,
        "select department, max(non_existent) from sxt.employees group by department",
        &accessor,
    );
}

#[test]
fn we_can_use_the_same_result_columns_with_different_aliases_and_associate_it_with_group_by() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
            "bonus".into() => ColumnType::VarChar,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "SELECT department as d1, department as d2 from employees group by department",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["department"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["department"],
            &[
                aliased_expr(col("department"), "d1"),
                aliased_expr(col("department"), "d2"),
            ],
        )],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_use_multiple_group_by_clauses_with_multiple_agg_and_non_agg_exprs() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "bonus".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
            "salary".into() => ColumnType::BigInt,
            "tax".into() => ColumnType::BigInt,
        },
    );
    let query_text = "select salary d1, max(tax), salary d2, sum(bonus) sum_bonus, count(name) count_s from sxt.employees group by salary, bonus, salary";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let ast =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["bonus", "name", "salary", "tax"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["salary", "bonus", "salary"],
            &[
                aliased_expr(col("salary"), "d1"),
                aliased_expr(max(col("tax")), "__max__"),
                aliased_expr(col("salary"), "d2"),
                aliased_expr(sum(col("bonus")), "sum_bonus"),
                aliased_expr(count(col("name")), "count_s"),
            ],
        )],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_a_simple_add_mul_sub_div_arithmetic_expressions_in_the_result_expr() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "a".into() => ColumnType::BigInt,
            "f".into() => ColumnType::Int128,
            "b".into() => ColumnType::BigInt,
            "h".into() => ColumnType::Int128,
        },
    );
    // TODO: add `a / b as a_div_b` result expr once polars properly
    // supports decimal division without panicking in production
    let ast = query_to_provable_ast(
        &t,
        "select a + b, 2 * f as f2, -77 - h as col, a + f as af from employees",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                aliased_plan(
                    add(column(&t, "a", &accessor), column(&t, "b", &accessor)),
                    "__expr__",
                ),
                aliased_plan(multiply(const_bigint(2), column(&t, "f", &accessor)), "f2"),
                aliased_plan(
                    subtract(const_bigint(-77), column(&t, "h", &accessor)),
                    "col",
                ),
                aliased_plan(
                    add(column(&t, "a", &accessor), column(&t, "f", &accessor)),
                    "af",
                ),
            ],
            tab(&t),
            const_bool(true),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_multiple_arithmetic_expression_where_multiplication_has_precedence_in_the_result_expr(
) {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "c".into() => ColumnType::BigInt,
            "f".into() => ColumnType::BigInt,
            "g".into() => ColumnType::BigInt,
            "h".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select (2 + f) * (c + g + 2 * h), ((h - g) * 2 + c + g) * (f + 2) as d from employees",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                aliased_plan(
                    multiply(
                        add(const_bigint(2), column(&t, "f", &accessor)),
                        add(
                            add(column(&t, "c", &accessor), column(&t, "g", &accessor)),
                            multiply(const_bigint(2), column(&t, "h", &accessor)),
                        ),
                    ),
                    "__expr__",
                ),
                aliased_plan(
                    multiply(
                        add(
                            add(
                                multiply(
                                    subtract(
                                        column(&t, "h", &accessor),
                                        column(&t, "g", &accessor),
                                    ),
                                    const_bigint(2),
                                ),
                                column(&t, "c", &accessor),
                            ),
                            column(&t, "g", &accessor),
                        ),
                        add(column(&t, "f", &accessor), const_bigint(2)),
                    ),
                    "d",
                ),
            ],
            tab(&t),
            const_bool(true),
        ),
        vec![],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_arithmetic_expression_within_aggregations_in_the_result_expr() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "c".into() => ColumnType::BigInt,
            "f".into() => ColumnType::BigInt,
            "g".into() => ColumnType::BigInt,
            "k".into() => ColumnType::BigInt,
        },
    );
    let ast = query_to_provable_ast(
        &t,
        "select c, sum(2 * f + c - -7) as d from employees group by c",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["c", "f"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["c"],
            &[
                aliased_expr(col("c"), "c"),
                aliased_expr(
                    sum(psub(padd(pmul(lit(2), col("f")), col("c")), lit(-7))),
                    "d",
                ),
            ],
        )],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_use_non_grouped_columns_outside_agg() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
        },
    );
    let identifier_not_in_agg_queries = vec![
        "select salary from sxt.employees group by name",
        "select sum(salary), salary from sxt.employees group by name",
        "select min(salary) + salary from sxt.employees group by name",
        "select 2 * salary, min(salary) from sxt.employees group by name",
    ];

    for query_text in &identifier_not_in_agg_queries {
        let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
        let result =
            QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

        assert!(matches!(
            result,
            Err(ConversionError::PostprocessingError {
                source: PostprocessingError::IdentNotInAggregationOperatorOrGroupByClause { .. }
            })
        ));
    }

    let invalid_group_by_queries = vec![
        "select 2 * salary, min(salary) from sxt.employees",
        "select sum(salary), salary from sxt.employees",
        "select max(salary) + 2 * salary from sxt.employees",
    ];

    for query_text in &invalid_group_by_queries {
        let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
        let result =
            QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

        assert!(matches!(
            result,
            Err(ConversionError::InvalidGroupByColumnRef { .. })
        ));
    }
}

#[test]
fn varchar_column_is_not_compatible_with_integer_column() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
        },
    );

    let bigint_to_varchar_queries = vec![
        "select -123 * name from sxt.employees",
        "select salary - name from sxt.employees",
    ];

    let varchar_to_bigint_queries = vec![
        "select name from sxt.employees where 'abc' = salary",
        "select name from sxt.employees where 'abc' != salary",
    ];

    for query_text in &bigint_to_varchar_queries {
        let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
        let result =
            QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

        assert_eq!(
            result,
            Err(ConversionError::DataTypeMismatch {
                left_type: ColumnType::BigInt.to_string(),
                right_type: ColumnType::VarChar.to_string(),
            })
        );
    }

    for query_text in &varchar_to_bigint_queries {
        let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
        let result =
            QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

        assert_eq!(
            result,
            Err(ConversionError::DataTypeMismatch {
                left_type: ColumnType::VarChar.to_string(),
                right_type: ColumnType::BigInt.to_string(),
            })
        );
    }
}

#[test]
fn arithmetic_operations_are_not_allowed_with_varchar_column() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "name".into() => ColumnType::VarChar,
            "position".into() => ColumnType::VarChar,
        },
    );

    let query_text = "select name - position from sxt.employees";
    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let result = QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

    assert_eq!(
        result,
        Err(ConversionError::DataTypeMismatch {
            left_type: ColumnType::VarChar.to_string(),
            right_type: ColumnType::VarChar.to_string(),
        })
    );
}

#[test]
fn varchar_column_is_not_allowed_within_numeric_aggregations() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "name".into() => ColumnType::VarChar,
        },
    );
    let sum_query = "select sum(name) from sxt.employees";
    let intermediate_ast = SelectStatementParser::new().parse(sum_query).unwrap();
    let result = QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

    assert!(matches!(
        result,
        Err(ConversionError::InvalidExpression { expression })
            if expression == "cannot use expression of type 'varchar' with numeric aggregation function 'sum'"
    ));

    let max_query = "select max(name) from sxt.employees";
    let intermediate_ast = SelectStatementParser::new().parse(max_query).unwrap();
    let result = QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

    assert!(matches!(
        result,
        Err(ConversionError::InvalidExpression { expression })
            if expression == "cannot use expression of type 'varchar' with numeric aggregation function 'max'"
    ));

    let min_query = "select min(name) from sxt.employees";
    let intermediate_ast = SelectStatementParser::new().parse(min_query).unwrap();
    let result = QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

    assert!(matches!(
        result,
        Err(ConversionError::InvalidExpression { expression })
            if expression == "cannot use expression of type 'varchar' with numeric aggregation function 'min'"
    ));
}

#[test]
fn group_by_with_bigint_column_is_valid() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
        },
    );
    let query_text = "select salary from sxt.employees group by salary";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let query =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_query = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["salary"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["salary"],
            &[aliased_expr(col("salary"), "salary")],
        )],
    );
    assert_eq!(query, expected_query);
}

#[test]
fn group_by_with_decimal_column_is_valid() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::Int128,
        },
    );
    let query_text = "select salary from sxt.employees group by salary";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let query =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_query = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["salary"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["salary"],
            &[aliased_expr(col("salary"), "salary")],
        )],
    );
    assert_eq!(query, expected_query);
}

#[test]
fn group_by_with_varchar_column_is_valid() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "name".into() => ColumnType::VarChar,
        },
    );
    let query_text = "select name from sxt.employees group by name";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let query =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_query = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["name"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(
            &["name"],
            &[aliased_expr(col("name"), "name")],
        )],
    );
    assert_eq!(query, expected_query);
}

#[test]
fn we_can_use_arithmetic_outside_agg_expressions_while_using_group_by() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "tax".into() => ColumnType::BigInt,
        },
    );
    let query_text =
        "select 2 * salary + sum(salary) - tax from sxt.employees group by salary, tax";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let query =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_query = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["salary", "tax"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![
            group_by_postprocessing(
                &["salary", "tax"],
                &[aliased_expr(
                    psub(
                        padd(pmul(lit(2), col("salary")), sum(col("salary"))),
                        col("tax"),
                    ),
                    "__expr__",
                )],
            ),
            select_expr(&[aliased_expr(
                psub(
                    padd(pmul(lit(2), col("salary")), col("__col_agg_0")),
                    col("tax"),
                ),
                "__expr__",
            )]),
        ],
    );
    assert_eq!(query, expected_query);
}

#[test]
fn we_can_use_arithmetic_outside_agg_expressions_without_using_group_by() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "bonus".into() => ColumnType::Int128,
        },
    );
    let query_text = "select 7 + max(salary) as max_i, min(salary + 777 * bonus) * -5 as min_d from sxt.employees";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let ast =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["bonus", "salary"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![
            group_by_postprocessing(
                &[],
                &[
                    aliased_expr(padd(lit(7), max(col("salary"))), "max_i"),
                    aliased_expr(
                        pmul(
                            min(padd(col("salary"), pmul(lit(777), col("bonus")))),
                            lit(-5),
                        ),
                        "min_d",
                    ),
                ],
            ),
            select_expr(&[
                aliased_expr(padd(lit(7), col("__col_agg_0")), "max_i"),
                aliased_expr(pmul(col("__col_agg_1"), lit(-5)), "min_d"),
            ]),
        ],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn count_aggregation_always_have_integer_type() {
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "name".into() => ColumnType::VarChar,
            "salary".into() => ColumnType::BigInt,
            "tax".into() => ColumnType::Int128,
        },
    );
    let query_text =
        "select 7 + count(name) as cs, count(salary) * -5 as ci, count(tax) from sxt.employees";

    let intermediate_ast = SelectStatementParser::new().parse(query_text).unwrap();
    let ast =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_ast = QueryExpr::new(
        filter(
            cols_expr_plan(&t, &["name", "salary", "tax"], &accessor),
            tab(&t),
            const_bool(true),
        ),
        vec![
            group_by_postprocessing(
                &[],
                &[
                    aliased_expr(padd(lit(7), count(col("name"))), "cs"),
                    aliased_expr(pmul(count(col("salary")), lit(-5)), "ci"),
                    aliased_expr(count(col("tax")), "__count__"),
                ],
            ),
            select_expr(&[
                aliased_expr(padd(lit(7), col("__col_agg_0")), "cs"),
                aliased_expr(pmul(col("__col_agg_1"), lit(-5)), "ci"),
                aliased_expr(col("__col_agg_2"), "__count__"),
            ]),
        ],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn select_wildcard_is_valid_with_group_by_exprs() {
    let columns = [
        "employee_name",
        "base_salary",
        "annual_bonus",
        "manager_name",
        "manager_salary",
        "manager_bonus",
        "department_name",
        "department_budget",
        "department_headcount",
    ];
    let sorted_columns = columns.iter().sorted().collect::<Vec<_>>();
    let aliased_exprs = columns
        .iter()
        .map(|c| aliased_expr(col(c), c))
        .collect::<Vec<_>>();
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "employee_name".into() => ColumnType::VarChar,
            "base_salary".into() => ColumnType::BigInt,
            "annual_bonus".into() => ColumnType::Int128,
            "manager_name".into() => ColumnType::VarChar,
            "manager_salary".into() => ColumnType::BigInt,
            "manager_bonus".into() => ColumnType::Int128,
            "department_name".into() => ColumnType::VarChar,
            "department_budget".into() => ColumnType::BigInt,
            "department_headcount".into() => ColumnType::Int128,
        },
    );

    let query_text = format!(
        "SELECT * FROM {} GROUP BY {}",
        "sxt.employees",
        columns.join(", ")
    );

    let intermediate_ast = SelectStatementParser::new().parse(&query_text).unwrap();
    let ast =
        QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor).unwrap();

    let expected_ast = QueryExpr::new(
        filter(
            sorted_columns
                .iter()
                .map(|c| col_expr_plan(&t, c, &accessor))
                .collect(),
            tab(&t),
            const_bool(true),
        ),
        vec![group_by_postprocessing(&columns, &aliased_exprs)],
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn nested_aggregations_are_not_supported() {
    let supported_agg = ["max", "min", "sum", "count"];
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
        },
    );

    for perm_aggs in supported_agg.iter().permutations(2) {
        let query_text = format!(
            "SELECT {}({}(salary)) FROM sxt.employees",
            perm_aggs[0], perm_aggs[1]
        );

        let intermediate_ast = SelectStatementParser::new().parse(&query_text).unwrap();
        let result =
            QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor);

        assert_eq!(
            result,
            Err(ConversionError::InvalidExpression {
                expression: "nested aggregations are not supported".to_string()
            })
        );
    }
}

#[test]
fn select_group_and_order_by_preserve_the_column_order_reference() {
    const N: usize = 4;
    let t = TableRef::new("sxt", "employees");
    let accessor = schema_accessor_from_table_ref_with_schema(
        &t,
        indexmap! {
            "salary".into() => ColumnType::BigInt,
            "department".into() => ColumnType::BigInt,
            "tax".into() => ColumnType::BigInt,
            "name".into() => ColumnType::VarChar,
        },
    );
    let base_cols: [&str; N] = ["salary", "department", "tax", "name"]; // sorted because of `select: [cols = ... ]`
    let base_ordering = [true, false, true, false];
    for (idx, perm_cols) in base_cols
        .into_iter()
        .permutations(N)
        .collect::<IndexSet<_>>()
        .into_iter()
        .enumerate()
    {
        let perm_col_plans = perm_cols
            .iter()
            .sorted()
            .map(|c| col_expr_plan(&t, c, &accessor))
            .collect();
        let aliased_perm_cols = perm_cols
            .iter()
            .map(|c| aliased_expr(col(c), c))
            .collect::<Vec<_>>();
        let group_cols = perm_cols.clone().into_iter().cycle().skip(1).take(N);
        let group_cols_vec = group_cols.clone().collect::<Vec<_>>();
        let order_cols = perm_cols.clone().into_iter().cycle().skip(2).take(N);
        let order_cols_vec = order_cols.clone().collect::<Vec<_>>();
        let ordering = base_ordering.into_iter().cycle().skip(idx).take(N);
        let ordering_vec = ordering.clone().collect::<Vec<_>>();
        let ordering_query_vec = ordering_vec
            .iter()
            .map(|b| match b {
                true => "ASC",
                false => "DESC",
            })
            .collect::<Vec<_>>();
        let query_text = format!(
            "SELECT {} FROM {} GROUP BY {} ORDER BY {}",
            perm_cols.join(", "),
            t,
            group_cols_vec.join(", "),
            order_cols_vec
                .iter()
                .zip(ordering_query_vec.iter())
                .map(|(c, o)| format!("{c} {o}"))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let intermediate_ast = SelectStatementParser::new().parse(&query_text).unwrap();
        let query =
            QueryExpr::try_new(intermediate_ast, t.schema_id().cloned().unwrap(), &accessor)
                .unwrap();

        let expected_query = QueryExpr::new(
            filter(perm_col_plans, tab(&t), const_bool(true)),
            vec![
                group_by_postprocessing(&group_cols_vec, &aliased_perm_cols),
                orders(&[2_usize, 3, 0, 1], &ordering_vec),
            ],
        );
        assert_eq!(query, expected_query);
    }
}

/// Creates a new [`QueryExpr`], with the given select statement and a sample schema accessor.
fn query_expr_for_test_table(sql_text: &str) -> QueryExpr {
    let schema_accessor = schema_accessor_from_table_ref_with_schema(
        &TableRef::new("test", "table"),
        indexmap! {
            "bigint_column".into() => ColumnType::BigInt,
            "varchar_column".into() => ColumnType::VarChar,
            "int128_column".into() => ColumnType::Int128,
        },
    );
    let default_schema: Ident = "test".into();
    let select_statement = SelectStatementParser::new().parse(sql_text).unwrap();
    QueryExpr::try_new(select_statement, default_schema, &schema_accessor).unwrap()
}

/// Serializes and deserializes [`QueryExpr`] with flexbuffers and asserts that it remains the same.
fn assert_query_expr_serializes_to_and_from_flex_buffers(query_expr: &QueryExpr) {
    let serialized = flexbuffers::to_vec(query_expr).unwrap();
    let deserialized: QueryExpr = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(deserialized, *query_expr);
}

#[test]
fn basic_query_expr_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table("select * from table");
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
}

#[test]
fn query_expr_with_selected_columns_can_serialize_to_and_from_flex_buffers() {
    let query_expr =
        query_expr_for_test_table("select bigint_column, varchar_column, int128_column from table");
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
}

#[test]
fn query_expr_with_aggregation_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table("select count(*) from table group by bigint_column");
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
}

#[test]
fn query_expr_with_filters_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table(
        "select * from table where bigint_column != 5 and varchar_column = 'example' or int128_column = 10",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
}

#[test]
fn query_expr_with_order_and_limits_can_serialize_to_and_from_flex_buffers() {
    let query_expr = query_expr_for_test_table(
        "select * from table order by int128_column desc limit 1 offset 1",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
}

#[test]
fn we_can_serialize_list_of_filters_from_query_expr() {
    let query_expr = query_expr_for_test_table("select * from table");
    let filter_execs = vec![query_expr.proof_expr()];
    let serialized = flexbuffers::to_vec(&filter_execs).unwrap();
    let deserialized: Vec<DynProofPlan> = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    let deserialized_as_ref: Vec<&DynProofPlan> = deserialized.iter().collect();
    assert_eq!(filter_execs.len(), deserialized_as_ref.len());
    assert_eq!(filter_execs[0], deserialized_as_ref[0]);
}

/// Creates a new [`QueryExpr`], with the given select statement and a sample schema accessor
/// with nullable columns.
fn query_expr_for_nullable_test_table(sql_text: &str) -> QueryExpr {
    let schema_accessor = schema_accessor_from_table_ref_with_schema(
        &TableRef::new("test", "nullable_table"),
        indexmap! {
            "nullable_int".into() => ColumnType::BigInt,
            "nullable_varchar".into() => ColumnType::VarChar,
            "non_nullable_int".into() => ColumnType::Int128,
            "nullable_bool".into() => ColumnType::Boolean,
        },
    );
    let default_schema: Ident = "test".into();
    let select_statement = SelectStatementParser::new().parse(sql_text).unwrap();
    QueryExpr::try_new(select_statement, default_schema, &schema_accessor).unwrap()
}

#[test]
fn we_can_parse_query_with_is_null_expression() {
    let query_expr = query_expr_for_nullable_test_table(
        "select * from nullable_table where nullable_int IS NULL",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
    
    let filter_plan = query_expr.proof_expr();
    let serialized = flexbuffers::to_vec(&filter_plan).unwrap();
    let deserialized: DynProofPlan = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(filter_plan, &deserialized);
}

#[test]
fn we_can_parse_query_with_is_not_null_expression() {
    let query_expr = query_expr_for_nullable_test_table(
        "select * from nullable_table where nullable_int IS NOT NULL",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
    
    let filter_plan = query_expr.proof_expr();
    let serialized = flexbuffers::to_vec(&filter_plan).unwrap();
    let deserialized: DynProofPlan = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(filter_plan, &deserialized);
}

#[test]
fn postgres_like_null_comparison_behavior() {
    let query_expr = query_expr_for_nullable_test_table(
        "select * from nullable_table where nullable_int = 5 OR nullable_int IS NULL",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
    
    let filter_plan = query_expr.proof_expr();
    let serialized = flexbuffers::to_vec(&filter_plan).unwrap();
    let deserialized: DynProofPlan = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(filter_plan, &deserialized);
}

#[test]
fn null_literal_in_select_clause() {
    let query_expr = query_expr_for_nullable_test_table(
        "select NULL from nullable_table",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
    
    let proof_plan = query_expr.proof_expr();
    let serialized = flexbuffers::to_vec(&proof_plan).unwrap();
    let deserialized: DynProofPlan = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(proof_plan, &deserialized);
}

#[test]
fn we_can_group_by_nullable_columns() {
    let query_expr = query_expr_for_nullable_test_table(
        "select nullable_int, count(*) from nullable_table group by nullable_int",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
    
    let proof_plan = query_expr.proof_expr();
    let serialized = flexbuffers::to_vec(&proof_plan).unwrap();
    let deserialized: DynProofPlan = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(proof_plan, &deserialized);
}

#[test]
fn we_can_order_by_nullable_columns() {
    let query_expr = query_expr_for_nullable_test_table(
        "select * from nullable_table order by nullable_int",
    );
    assert_query_expr_serializes_to_and_from_flex_buffers(&query_expr);
    
    let proof_plan = query_expr.proof_expr();
    let serialized = flexbuffers::to_vec(&proof_plan).unwrap();
    let deserialized: DynProofPlan = flexbuffers::from_slice(serialized.as_slice()).unwrap();
    assert_eq!(proof_plan, &deserialized);
}
