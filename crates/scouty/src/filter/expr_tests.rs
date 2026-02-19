#[cfg(test)]
mod tests {
    use crate::filter::expr::{parse, Expr, Op};

    #[test]
    fn simple_comparison() {
        let expr = parse(r#"level = "Error""#).unwrap();
        assert_eq!(
            expr,
            Expr::Comparison {
                field: "level".into(),
                op: Op::Eq,
                value: "Error".into(),
            }
        );
    }

    #[test]
    fn and_expression() {
        let expr = parse(r#"level = "Error" AND component = "auth""#).unwrap();
        match expr {
            Expr::And(left, right) => {
                assert_eq!(*left, Expr::Comparison { field: "level".into(), op: Op::Eq, value: "Error".into() });
                assert_eq!(*right, Expr::Comparison { field: "component".into(), op: Op::Eq, value: "auth".into() });
            }
            _ => panic!("Expected And expression"),
        }
    }

    #[test]
    fn or_expression() {
        let expr = parse(r#"level = "Error" OR level = "Fatal""#).unwrap();
        match expr {
            Expr::Or(left, right) => {
                assert_eq!(*left, Expr::Comparison { field: "level".into(), op: Op::Eq, value: "Error".into() });
                assert_eq!(*right, Expr::Comparison { field: "level".into(), op: Op::Eq, value: "Fatal".into() });
            }
            _ => panic!("Expected Or expression"),
        }
    }

    #[test]
    fn parenthesized_expression() {
        let expr = parse(r#"(level = "Error" OR level = "Fatal") AND component = "auth""#).unwrap();
        match expr {
            Expr::And(left, right) => {
                assert!(matches!(*left, Expr::Or(_, _)));
                assert_eq!(*right, Expr::Comparison { field: "component".into(), op: Op::Eq, value: "auth".into() });
            }
            _ => panic!("Expected And with nested Or"),
        }
    }

    #[test]
    fn not_expression() {
        let expr = parse(r#"NOT level = "Debug""#).unwrap();
        match expr {
            Expr::Not(inner) => {
                assert_eq!(*inner, Expr::Comparison { field: "level".into(), op: Op::Eq, value: "Debug".into() });
            }
            _ => panic!("Expected Not expression"),
        }
    }

    #[test]
    fn nested_not() {
        let expr = parse(r#"NOT NOT level = "Info""#).unwrap();
        match expr {
            Expr::Not(inner) => assert!(matches!(*inner, Expr::Not(_))),
            _ => panic!("Expected nested Not"),
        }
    }

    #[test]
    fn all_operators() {
        for (op_str, expected_op) in &[
            ("=", Op::Eq), ("!=", Op::Ne),
            (">", Op::Gt), (">=", Op::Ge),
            ("<", Op::Lt), ("<=", Op::Le),
            ("contains", Op::Contains),
            ("starts_with", Op::StartsWith),
            ("ends_with", Op::EndsWith),
            ("regex", Op::Regex),
        ] {
            let input = format!(r#"field {} "val""#, op_str);
            let expr = parse(&input).unwrap();
            assert_eq!(
                expr,
                Expr::Comparison { field: "field".into(), op: expected_op.clone(), value: "val".into() }
            );
        }
    }

    #[test]
    fn and_has_higher_precedence_than_or() {
        // "a OR b AND c" should parse as "a OR (b AND c)"
        let expr = parse(r#"level = "A" OR level = "B" AND level = "C""#).unwrap();
        match expr {
            Expr::Or(_, right) => assert!(matches!(*right, Expr::And(_, _))),
            _ => panic!("Expected OR at top level"),
        }
    }

    #[test]
    fn unquoted_value() {
        let expr = parse("id >= 100").unwrap();
        assert_eq!(
            expr,
            Expr::Comparison { field: "id".into(), op: Op::Ge, value: "100".into() }
        );
    }

    #[test]
    fn metadata_dot_field() {
        let expr = parse(r#"metadata.env = "prod""#).unwrap();
        assert_eq!(
            expr,
            Expr::Comparison { field: "metadata.env".into(), op: Op::Eq, value: "prod".into() }
        );
    }

    #[test]
    fn complex_nested() {
        let expr = parse(r#"(level = "Error" OR level = "Fatal") AND (component = "auth" OR component = "db") AND NOT source = "test""#).unwrap();
        // Should parse without error
        assert!(matches!(expr, Expr::And(_, _)));
    }

    #[test]
    fn empty_expression_error() {
        assert!(parse("").is_err());
    }

    #[test]
    fn missing_value_error() {
        assert!(parse("level =").is_err());
    }

    #[test]
    fn unclosed_paren_error() {
        assert!(parse(r#"(level = "Error""#).is_err());
    }

    #[test]
    fn single_quotes() {
        let expr = parse("level = 'Error'").unwrap();
        assert_eq!(
            expr,
            Expr::Comparison { field: "level".into(), op: Op::Eq, value: "Error".into() }
        );
    }
}
