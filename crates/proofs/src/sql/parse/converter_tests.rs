use crate::base::database::{ColumnRef, ColumnType, SchemaAccessor, TableRef, TestAccessor};
use crate::sql::ast::{
    AndExpr, ConstBoolExpr, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr, TableExpr,
};
use crate::sql::parse::Converter;
use curve25519_dalek::scalar::Scalar;
use indexmap::IndexMap;
use proofs_sql::sql::SelectStatementParser;
use proofs_sql::Identifier;

#[test]
fn we_can_convert_an_ast_with_one_column() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![3])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(3_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_two_columns() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a,  b from sxt_tab where c = 123")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([
            ("a".to_string(), vec![]),
            ("b".to_string(), vec![]),
            ("c".to_string(), vec![]),
        ]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![
            FilterResultExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("a").unwrap(),
                    ColumnType::BigInt,
                ),
                Identifier::try_new("a").unwrap(),
            ),
            FilterResultExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                Identifier::try_new("b").unwrap(),
            ),
        ],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("c").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(123_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_select_star() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select * from sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("b".to_string(), vec![5, 6]), ("a".to_string(), vec![3, 2])]),
        0_usize,
    );

    let result_columns: Vec<_> = accessor
        .lookup_schema(&table_ref)
        .into_iter()
        .map(|(column_name_id, column_type)| {
            let column_name = column_name_id.name().to_string();
            FilterResultExpr::new(
                ColumnRef::new(table_ref.clone(), column_name_id, column_type),
                Identifier::try_new(&column_name).unwrap(),
            )
        })
        .collect();

    assert_eq!(result_columns.len(), 2);

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        result_columns,
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(3_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_more_complex_select_star() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a, *, b,* from sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([
            ("b".to_string(), vec![5, 6]),
            ("a".to_string(), vec![3, 2]),
            ("c".to_string(), vec![78, 8]),
        ]),
        0_usize,
    );

    let all_schema_columns: Vec<_> = accessor
        .lookup_schema(&table_ref)
        .into_iter()
        .map(|(column_name_id, column_type)| {
            let column_name = column_name_id.name().to_string();
            FilterResultExpr::new(
                ColumnRef::new(table_ref.clone(), column_name_id, column_type),
                Identifier::try_new(&column_name).unwrap(),
            )
        })
        .collect();

    assert_eq!(all_schema_columns.len(), 3);

    let mut result_columns = vec![FilterResultExpr::new(
        ColumnRef::new(
            table_ref.clone(),
            Identifier::try_new("a").unwrap(),
            ColumnType::BigInt,
        ),
        Identifier::try_new("a").unwrap(),
    )];

    result_columns.extend_from_slice(&all_schema_columns[..]);

    result_columns.push(FilterResultExpr::new(
        ColumnRef::new(
            table_ref.clone(),
            Identifier::try_new("b").unwrap(),
            ColumnType::BigInt,
        ),
        Identifier::try_new("b").unwrap(),
    ));

    result_columns.extend(all_schema_columns);

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        result_columns,
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(3_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_positive_cond() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where b = +4")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![]), ("b".to_string(), vec![])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(4_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_not_equals_cond() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where b <> +4")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![]), ("b".to_string(), vec![])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(NotExpr::new(Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(4_u64),
        )))),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_negative_cond() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where b = -4")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![]), ("b".to_string(), vec![])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            -Scalar::from(4_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_and() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) and (c = -2)")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([
            ("a".to_string(), vec![]),
            ("b".to_string(), vec![]),
            ("c".to_string(), vec![]),
        ]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(AndExpr::new(
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                Scalar::from(3_u64),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("c").unwrap(),
                    ColumnType::BigInt,
                ),
                -Scalar::from(2_u64),
            )),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_or() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) or (c = -2)")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([
            ("a".to_string(), vec![]),
            ("b".to_string(), vec![]),
            ("c".to_string(), vec![]),
        ]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(OrExpr::new(
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                Scalar::from(3_u64),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("c").unwrap(),
                    ColumnType::BigInt,
                ),
                -Scalar::from(2_u64),
            )),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_or_not() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) or (not (c = -2))")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([
            ("a".to_string(), vec![]),
            ("b".to_string(), vec![]),
            ("c".to_string(), vec![]),
        ]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(OrExpr::new(
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                Scalar::from(3_u64),
            )),
            Box::new(NotExpr::new(Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("c").unwrap(),
                    ColumnType::BigInt,
                ),
                -Scalar::from(2_u64),
            )))),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_not_and_or() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where not (((f = 45) or (c = -2)) and (b = 3))")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([
            ("a".to_string(), vec![]),
            ("b".to_string(), vec![]),
            ("c".to_string(), vec![]),
            ("f".to_string(), vec![]),
        ]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(NotExpr::new(Box::new(AndExpr::new(
            Box::new(OrExpr::new(
                Box::new(EqualsExpr::new(
                    ColumnRef::new(
                        table_ref.clone(),
                        Identifier::try_new("f").unwrap(),
                        ColumnType::BigInt,
                    ),
                    Scalar::from(45_u64),
                )),
                Box::new(EqualsExpr::new(
                    ColumnRef::new(
                        table_ref.clone(),
                        Identifier::try_new("c").unwrap(),
                        ColumnType::BigInt,
                    ),
                    -Scalar::from(2_u64),
                )),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                Scalar::from(3_u64),
            )),
        )))),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_min_i64_filter_value() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = -9223372036854775808")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![3])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            -Scalar::from(9223372036854775808u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_max_i64_filter_value() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 9223372036854775807")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![3])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(9223372036854775807_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_using_as_rename_keyword() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a as b_rename from sxt_tab where b = +4")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![]), ("b".to_string(), vec![])]),
        0_usize,
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("b_rename").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(4_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_a_nonexistent_column() {
    let table_ref: TableRef = "sxt.sxt_tab".parse().unwrap();
    let default_schema = table_ref.schema_id();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("b".to_string(), vec![3])]),
        0_usize,
    );

    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
        .is_err());
}

#[test]
fn we_can_convert_an_ast_with_a_schema() {
    let table_ref = "eth.sxt_tab".parse().unwrap();
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from eth.sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![3])]),
        0_usize,
    );

    let default_schema = Identifier::try_new("sxt").unwrap();

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, &default_schema)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(3_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_without_any_filter() {
    let table_ref = "eth.sxt_tab".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        &table_ref,
        &IndexMap::from([("a".to_string(), vec![3])]),
        0_usize,
    );

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(ConstBoolExpr::new(true)),
    );

    let default_schema = table_ref.schema_id();

    let queries = ["select * from eth.sxt_tab", "select a from eth.sxt_tab"];
    for query in queries {
        let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();

        let provable_ast = Converter::default()
            .visit_intermediate_ast(&intermediate_ast, &accessor, default_schema)
            .unwrap();

        assert_eq!(provable_ast, expected_provable_ast);
    }
}
