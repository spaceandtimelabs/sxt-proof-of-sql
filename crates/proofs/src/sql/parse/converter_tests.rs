use crate::base::database::TestAccessor;
use crate::sql::ast::{
    AndExpr, ColumnRef, EqualsExpr, FilterExpr, FilterResultExpr, NotExpr, OrExpr, TableExpr,
};
use crate::sql::parse::Converter;
use curve25519_dalek::scalar::Scalar;
use proofs_sql::sql::SelectStatementParser;
use std::collections::HashMap;

#[test]
fn we_can_convert_an_ast_with_one_column() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table("SXT_TAB", &HashMap::from([("A".to_string(), vec![3])]));

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "A".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            },
            Scalar::from(3_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_two_columns() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a,  b from sxt_tab where c = 123")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([
            ("A".to_string(), vec![]),
            ("B".to_string(), vec![]),
            ("C".to_string(), vec![]),
        ]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![
            FilterResultExpr::new(ColumnRef {
                column_name: "A".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            }),
            FilterResultExpr::new(ColumnRef {
                column_name: "B".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            }),
        ],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "C".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            },
            Scalar::from(123_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

// Filter
#[test]
fn we_can_convert_an_ast_with_one_positive_cond() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where b = +4")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([("A".to_string(), vec![]), ("B".to_string(), vec![])]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "B".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            },
            Scalar::from(4_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

// Filter
#[test]
fn we_can_convert_an_ast_with_one_negative_cond() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where b = -4")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([("A".to_string(), vec![]), ("B".to_string(), vec![])]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "B".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            },
            -Scalar::from(4_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_and() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) and (c = -2)")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([
            ("A".to_string(), vec![]),
            ("B".to_string(), vec![]),
            ("C".to_string(), vec![]),
        ]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(AndExpr::new(
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "B".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                Scalar::from(3_u64),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "C".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                -Scalar::from(2_u64),
            )),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_or() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) or (c = -2)")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([
            ("A".to_string(), vec![]),
            ("B".to_string(), vec![]),
            ("C".to_string(), vec![]),
        ]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(OrExpr::new(
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "B".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                Scalar::from(3_u64),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "C".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                -Scalar::from(2_u64),
            )),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_or_not() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where (b = 3) or (not (c = -2))")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([
            ("A".to_string(), vec![]),
            ("B".to_string(), vec![]),
            ("C".to_string(), vec![]),
        ]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(OrExpr::new(
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "B".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                Scalar::from(3_u64),
            )),
            Box::new(NotExpr::new(Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "C".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                -Scalar::from(2_u64),
            )))),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_not_and_or() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where not (((f = 45) or (c = -2)) and (b = 3))")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "SXT_TAB",
        &HashMap::from([
            ("A".to_string(), vec![]),
            ("B".to_string(), vec![]),
            ("C".to_string(), vec![]),
            ("F".to_string(), vec![]),
        ]),
    );

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(NotExpr::new(Box::new(AndExpr::new(
            Box::new(OrExpr::new(
                Box::new(EqualsExpr::new(
                    ColumnRef {
                        column_name: "F".to_string(),
                        table_name: "SXT_TAB".to_string(),
                        namespace: None,
                    },
                    Scalar::from(45_u64),
                )),
                Box::new(EqualsExpr::new(
                    ColumnRef {
                        column_name: "C".to_string(),
                        table_name: "SXT_TAB".to_string(),
                        namespace: None,
                    },
                    -Scalar::from(2_u64),
                )),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef {
                    column_name: "B".to_string(),
                    table_name: "SXT_TAB".to_string(),
                    namespace: None,
                },
                Scalar::from(3_u64),
            )),
        )))),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_min_i64_filter_value() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = -9223372036854775808")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table("SXT_TAB", &HashMap::from([("A".to_string(), vec![3])]));

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "A".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            },
            -Scalar::from(9223372036854775808u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_max_i64_filter_value() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 9223372036854775807")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table("SXT_TAB", &HashMap::from([("A".to_string(), vec![3])]));

    let provable_ast = Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .unwrap();

    let expected_provable_ast = FilterExpr::new(
        vec![FilterResultExpr::new(ColumnRef {
            column_name: "A".to_string(),
            table_name: "SXT_TAB".to_string(),
            namespace: None,
        })],
        TableExpr {
            name: "SXT_TAB".to_string(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef {
                column_name: "A".to_string(),
                table_name: "SXT_TAB".to_string(),
                namespace: None,
            },
            Scalar::from(9223372036854775807_u64),
        )),
    );

    assert_eq!(expected_provable_ast, provable_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_a_nonexistent_column() {
    let intermediate_ast = SelectStatementParser::new()
        .parse("select a from sxt_tab where a = 3")
        .unwrap();

    let mut accessor = TestAccessor::new();
    accessor.add_table("SXT_TAB", &HashMap::from([("B".to_string(), vec![3])]));

    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor)
        .is_err());
}

#[test]
fn we_cannot_convert_an_ast_with_a_namespaced_table_yet() {
    assert!(SelectStatementParser::new()
        .parse("select a from eth.sxt_tab where a = -3")
        .is_err());
}
