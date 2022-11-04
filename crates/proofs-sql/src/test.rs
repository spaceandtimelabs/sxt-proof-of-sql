use crate::intermediate_ast::*;
use crate::sql;
use crate::symbols::Name;

// Projection
#[test]
fn select_one_col() {
    let actual = sql::SelectStatementParser::new().parse("select a from namespace.sxt_tab");
    let tab = Name::from("sxt_tab");
    let namespace = Name::from("namespace");
    let a = Name::from("a");
    let rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(rc)]),
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: None,
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn select_two_cols() {
    let actual = sql::SelectStatementParser::new().parse("select a, b from tab");
    let tab = Name::from("TAB");
    let a = Name::from("a");
    let b = Name::from("b");
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![b])),
        rename: None,
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc), Box::new(b_rc)]),
        from: vec![Box::new(TableExpression::Named { name: vec![tab] })],
        where_expr: None,
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn select_star() {
    let actual = sql::SelectStatementParser::new().parse("select * from namespace.tab");
    let tab = Name::from("TAB");
    let namespace = Name::from("namespace");
    let query = SetExpression::Query {
        columns: ResultColumns::All,
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: None,
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn select_tablename_star() {
    let actual = sql::SelectStatementParser::new().parse("select tab.* from namespace.tab");
    let tab = Name::from("tab");
    let namespace = Name::from("namespace");
    let rc = ResultColumn::AllFrom(tab.clone());
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(rc)]),
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: None,
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

// Filter
#[test]
fn filter_one_cond() {
    let actual =
        sql::SelectStatementParser::new().parse("select a from namespace.tab where b = +4");
    let tab = Name::from("TAB");
    let namespace = Name::from("namespace");
    let a = Name::from("a");
    let b = Name::from("b");
    let four_expr = Expression::Literal(Literal::NumericLiteral(4));
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let comp = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(four_expr),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: Some(Box::new(comp)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn filter_one_cond_with_add_sub() {
    let actual = sql::SelectStatementParser::new().parse("select a from tab where b = 4 + 5 - 7");
    let tab = Name::from("TAB");
    let a = Name::from("a");
    let b = Name::from("b");
    let four_expr = Expression::Literal(Literal::NumericLiteral(4));
    let five_expr = Expression::Literal(Literal::NumericLiteral(5));
    let seven_expr = Expression::Literal(Literal::NumericLiteral(7));
    let sum = Expression::Binary {
        op: BinaryOperator::Add,
        left: Box::new(four_expr),
        right: Box::new(five_expr),
    };
    let diff = Expression::Binary {
        op: BinaryOperator::Subtract,
        left: Box::new(sum),
        right: Box::new(seven_expr),
    };
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let comp = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(diff),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named { name: vec![tab] })],
        where_expr: Some(Box::new(comp)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn filter_one_cond_with_num_ops() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a from another_namespace.tab where b = -(4 + 5) * 6 / 3");
    let tab = Name::from("TAB");
    let namespace = Name::from("another_namespace");
    let a = Name::from("a");
    let b = Name::from("b");
    let four_expr = Expression::Literal(Literal::NumericLiteral(4));
    let five_expr = Expression::Literal(Literal::NumericLiteral(5));
    let six_expr = Expression::Literal(Literal::NumericLiteral(6));
    let three_expr = Expression::Literal(Literal::NumericLiteral(3));
    let sum = Expression::Binary {
        op: BinaryOperator::Add,
        left: Box::new(four_expr),
        right: Box::new(five_expr),
    };
    let neg = Expression::Unary {
        op: UnaryOperator::Negate,
        expr: Box::new(sum),
    };
    let prod = Expression::Binary {
        op: BinaryOperator::Multiply,
        left: Box::new(neg),
        right: Box::new(six_expr),
    };
    let num = Expression::Binary {
        op: BinaryOperator::Divide,
        left: Box::new(prod),
        right: Box::new(three_expr),
    };
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let comp = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(num),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: Some(Box::new(comp)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn filter_two_cond_and() {
    let actual =
        sql::SelectStatementParser::new().parse("select a from tab where b = 3 and c != -2");
    let tab = Name::from("TAB");
    let a = Name::from("a");
    let b = Name::from("b");
    let c = Name::from("c");
    let three_expr = Expression::Literal(Literal::NumericLiteral(3));
    let minus_two_expr = Expression::Literal(Literal::NumericLiteral(-2));
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let c_expr = Expression::QualifiedIdentifier(vec![c]);
    let comp_0 = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(three_expr),
    };
    let comp_1 = Expression::Comparison {
        op: ComparisonOperator::NotEqual,
        left: Box::new(c_expr),
        right: Box::new(minus_two_expr),
    };
    let and = Expression::Binary {
        op: BinaryOperator::And,
        left: Box::new(comp_0),
        right: Box::new(comp_1),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named { name: vec![tab] })],
        where_expr: Some(Box::new(and)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn filter_two_cond_and_not() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a from public.tab where b = 3 and not c != -2");
    let tab = Name::from("TAB");
    let namespace = Name::from("public");
    let a = Name::from("a");
    let b = Name::from("b");
    let c = Name::from("c");
    let three_expr = Expression::Literal(Literal::NumericLiteral(3));
    let minus_two_expr = Expression::Literal(Literal::NumericLiteral(-2));
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let c_expr = Expression::QualifiedIdentifier(vec![c]);
    let comp_0 = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(three_expr),
    };
    let comp_1 = Expression::Comparison {
        op: ComparisonOperator::NotEqual,
        left: Box::new(c_expr),
        right: Box::new(minus_two_expr),
    };
    let neg = Expression::Unary {
        op: UnaryOperator::Not,
        expr: Box::new(comp_1),
    };
    let and = Expression::Binary {
        op: BinaryOperator::And,
        left: Box::new(comp_0),
        right: Box::new(neg),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: Some(Box::new(and)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

// Test whether operator precedence is working
#[test]
fn filter_mult_cond_and_or() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a from tab where b = 3 and c != -2 or d <> 4 and e == 5");
    let tab = Name::from("TAB");
    let a = Name::from("a");
    let b = Name::from("b");
    let c = Name::from("c");
    let d = Name::from("d");
    let e = Name::from("e");
    let three_expr = Expression::Literal(Literal::NumericLiteral(3));
    let minus_two_expr = Expression::Literal(Literal::NumericLiteral(-2));
    let four_expr = Expression::Literal(Literal::NumericLiteral(4));
    let five_expr = Expression::Literal(Literal::NumericLiteral(5));
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let c_expr = Expression::QualifiedIdentifier(vec![c]);
    let d_expr = Expression::QualifiedIdentifier(vec![d]);
    let e_expr = Expression::QualifiedIdentifier(vec![e]);
    let comp_00 = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(three_expr),
    };
    let comp_01 = Expression::Comparison {
        op: ComparisonOperator::NotEqual,
        left: Box::new(c_expr),
        right: Box::new(minus_two_expr),
    };
    let and_0 = Expression::Binary {
        op: BinaryOperator::And,
        left: Box::new(comp_00),
        right: Box::new(comp_01),
    };
    let comp_10 = Expression::Comparison {
        op: ComparisonOperator::NotEqual,
        left: Box::new(d_expr),
        right: Box::new(four_expr),
    };
    let comp_11 = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(e_expr),
        right: Box::new(five_expr),
    };
    let and_1 = Expression::Binary {
        op: BinaryOperator::And,
        left: Box::new(comp_10),
        right: Box::new(comp_11),
    };
    let or = Expression::Binary {
        op: BinaryOperator::Or,
        left: Box::new(and_0),
        right: Box::new(and_1),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named { name: vec![tab] })],
        where_expr: Some(Box::new(or)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

// Check min and max values of i64

#[test]
fn filter_bigint_min_value() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a from some_namespace.tab where b = -9223372036854775808");
    let tab = Name::from("TAB");
    let namespace = Name::from("some_namespace");
    let a = Name::from("a");
    let b = Name::from("b");
    let i64min_expr = Expression::Literal(Literal::NumericLiteral(-9223372036854775808));
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let comp = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(i64min_expr),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named {
            name: vec![namespace, tab],
        })],
        where_expr: Some(Box::new(comp)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn filter_bigint_max_value() {
    let actual =
        sql::SelectStatementParser::new().parse("select a from tab where b = 9223372036854775807");
    let tab = Name::from("TAB");
    let a = Name::from("a");
    let b = Name::from("b");
    let i64max_expr = Expression::Literal(Literal::NumericLiteral(9223372036854775807));
    let a_rc = ResultColumn::Expr {
        expr: Box::new(Expression::QualifiedIdentifier(vec![a])),
        rename: None,
    };
    let b_expr = Expression::QualifiedIdentifier(vec![b]);
    let comp = Expression::Comparison {
        op: ComparisonOperator::Equal,
        left: Box::new(b_expr),
        right: Box::new(i64max_expr),
    };
    let query = SetExpression::Query {
        columns: ResultColumns::List(vec![Box::new(a_rc)]),
        from: vec![Box::new(TableExpression::Named { name: vec![tab] })],
        where_expr: Some(Box::new(comp)),
    };
    let expected = SelectStatement {
        expr: Box::new(query),
    };
    assert_eq!(expected, actual.unwrap());
}

#[test]
#[should_panic(expected = "Integer out of range")]
fn filter_min_overflow() {
    let actual =
        sql::SelectStatementParser::new().parse("select a from tab where b = -9223372036854775809");
    actual.unwrap();
}

#[test]
#[should_panic(expected = "Integer out of range")]
fn filter_max_overflow() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a from namespace.tab where b = 9223372036854775808");
    actual.unwrap();
}

// Unparsables
// Unparsables consist of the following categories
// 1. Queries we don't support yet but plan to support in the future.
// 2. Valid queries that are out of scope.
// 3. Invalid queries.

// Not supported yet
// The following are valid queries that will be gradually enabled as our PoSQL engine is built.
// We ignore the exact LALRPOP error type since it changes as LARPOP is upgraded
// and is outside our control.

// Select constant
#[test]
#[should_panic]
fn select_constant_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select 2");
    actual.unwrap();
}

// Aliasing
#[test]
#[should_panic]
fn aliasing_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select a as b from tab");
    actual.unwrap();
}

// Select from subquery
#[test]
#[should_panic]
fn subquery_not_supported() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a from (select a from namespace.tab where b > 4)");
    actual.unwrap();
}

// Semicolon at the end of a query
#[test]
#[should_panic]
fn semicolon_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select a from tab;");
    actual.unwrap();
}

// Limit
#[test]
#[should_panic]
fn limit_not_supported() {
    let actual =
        sql::SelectStatementParser::new().parse("select a from name.tab where b = 4 limit 3");
    actual.unwrap();
}

// Inequality
#[test]
#[should_panic]
fn filter_gt_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select a from tab where b > 4");
    actual.unwrap();
}

#[test]
#[should_panic]
fn filter_le_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select a from tab where b <= 4");
    actual.unwrap();
}

// Aggregation
#[test]
#[should_panic]
fn sum_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select sum(a) from some_namespace.tab");
    actual.unwrap();
}

// Group By
#[test]
#[should_panic]
fn groupby_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("select b, sum(a) from tab group by b");
    actual.unwrap();
}

// Join
#[test]
#[should_panic]
fn inner_join_not_supported() {
    let actual = sql::SelectStatementParser::new()
        .parse("select tab1.a from tab1 join tab2 on tab1.c = tab2.c where tab2.b > 4");
    actual.unwrap();
}

// Case when
#[test]
#[should_panic]
fn casewhen_not_supported() {
    let actual = sql::SelectStatementParser::new()
        .parse("select case when a == 2 then 3 else 5 from tab where b <= 4");
    actual.unwrap();
}

// SQL out of scope
// The following queries are valid but we don't plan to support them.

// Insert
#[test]
#[should_panic]
fn insert_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("insert into tab values (1, 2)");
    actual.unwrap();
}

// Update
#[test]
#[should_panic]
fn update_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("update tab set a = 1 where b = 2");
    actual.unwrap();
}

// Delete
#[test]
#[should_panic]
fn delete_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("delete from namespace.tab where b = 2");
    actual.unwrap();
}

// Create table
#[test]
#[should_panic]
fn create_table_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("create table tab (a bigint)");
    actual.unwrap();
}

// Truncate
#[test]
#[should_panic]
fn truncate_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("truncate table tab");
    actual.unwrap();
}

// Drop
#[test]
#[should_panic]
fn drop_not_supported() {
    let actual = sql::SelectStatementParser::new().parse("drop table tab");
    actual.unwrap();
}

// Invalid SQL
#[test]
#[should_panic]
fn binaryop_with_one_operand_is_invalid() {
    let actual = sql::SelectStatementParser::new()
        .parse("select a + from a_namespace.tab where b = 4 limit 3");
    actual.unwrap();
}

#[test]
#[should_panic]
fn unparseable_char_is_invalid() {
    let actual = sql::SelectStatementParser::new().parse("select '' from where b = 4 limit 3!");
    actual.unwrap();
}

#[test]
#[should_panic]
fn having_from_directly_after_select_is_invalid() {
    let actual = sql::SelectStatementParser::new().parse("select from where");
    actual.unwrap();
}

#[test]
#[should_panic]
fn having_nothing_after_where_is_invalid() {
    let actual = sql::SelectStatementParser::new().parse("select a from tab where");
    actual.unwrap();
}

#[test]
#[should_panic]
fn select_col_without_table_is_invalid() {
    let actual = sql::SelectStatementParser::new().parse("select a where b = 4");
    actual.unwrap();
}

#[test]
#[should_panic]
fn not_having_select_is_invalid() {
    let actual = sql::SelectStatementParser::new().parse("where b = 4");
    actual.unwrap();
}
