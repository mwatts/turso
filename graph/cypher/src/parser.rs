//! Parse the supported Cypher read-query slice into source-oriented syntax.
//!
//! Source: <https://github.com/rustic-ai/uni-db>
//! Revision: 0812a496c62769b67cf688930750ae384e3de68d
//! Path: crates/uni-cypher/src/grammar/walker.rs
//! License: Apache-2.0
//! Adaptation: structural-adaptation
//! Changes: Replaced Uni AST and error types, reduced the supported syntax,
//! and retained only a source-AST walker with byte-span diagnostics.

use pest::{error::InputLocation, iterators::Pair, Parser};
use pest_derive::Parser;
use thiserror::Error;

use crate::{
    BinaryOperator, Clause, Direction, Expression, Literal, MatchClause, NodePattern, PathPattern,
    ProjectionClause, ProjectionItem, Query, RelationshipPattern, RelationshipRange, SortItem,
    Span, Spanned, UnwindClause,
};

#[derive(Parser)]
#[grammar = "cypher.pest"]
struct CypherParser;

type ParsedProperties = Vec<(Spanned<String>, Spanned<Expression>)>;

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("{message} at byte {span_start}..{span_end}")]
pub struct ParseError {
    pub message: String,
    pub span_start: usize,
    pub span_end: usize,
}

impl ParseError {
    fn at(span: Span, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span_start: span.start,
            span_end: span.end,
        }
    }

    pub const fn span(&self) -> Span {
        Span::new(self.span_start, self.span_end)
    }
}

pub fn parse(source: &str) -> Result<Query, ParseError> {
    let mut pairs = CypherParser::parse(Rule::query, source).map_err(parse_error)?;
    let query = pairs
        .next()
        .ok_or_else(|| ParseError::at(Span::new(0, 0), "empty query"))?;
    walk_query(query)
}

fn parse_error(error: pest::error::Error<Rule>) -> ParseError {
    let span = match error.location {
        InputLocation::Pos(position) => Span::new(position, position),
        InputLocation::Span((start, end)) => Span::new(start, end),
    };
    ParseError::at(span, error.variant.message())
}

fn walk_query(pair: Pair<'_, Rule>) -> Result<Query, ParseError> {
    let span = pair_span(&pair);
    let clauses = pair
        .into_inner()
        .filter(|pair| pair.as_rule() == Rule::clause)
        .map(walk_clause)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Query { clauses, span })
}

fn walk_clause(pair: Pair<'_, Rule>) -> Result<Spanned<Clause>, ParseError> {
    let pair = only_child(pair)?;
    let span = pair_span(&pair);
    let clause = match pair.as_rule() {
        Rule::match_clause => Clause::Match(walk_match(pair)?),
        Rule::unwind_clause => Clause::Unwind(walk_unwind(pair)?),
        Rule::with_clause => Clause::With(walk_projection_clause(pair)?),
        Rule::return_clause => Clause::Return(walk_projection_clause(pair)?),
        rule => return Err(unexpected(&pair, "clause", rule)),
    };
    Ok(Spanned::new(clause, span))
}

fn walk_unwind(pair: Pair<'_, Rule>) -> Result<UnwindClause, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let expression = inner
        .find(|item| item.as_rule() == Rule::expression)
        .ok_or_else(|| ParseError::at(span, "UNWIND has no expression"))?;
    let expression = walk_expression(expression)?;
    let alias = inner
        .find(|item| item.as_rule() == Rule::identifier)
        .ok_or_else(|| ParseError::at(span, "UNWIND has no alias"))?;
    Ok(UnwindClause {
        expression,
        alias: walk_identifier(alias),
    })
}

fn walk_match(pair: Pair<'_, Rule>) -> Result<MatchClause, ParseError> {
    let optional = pair
        .clone()
        .into_inner()
        .any(|child| child.as_rule() == Rule::OPTIONAL);
    let mut paths = Vec::new();
    let mut predicate = None;
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::pattern => {
                paths = child
                    .into_inner()
                    .filter(|item| item.as_rule() == Rule::path_pattern)
                    .map(walk_path)
                    .collect::<Result<Vec<_>, _>>()?;
            }
            Rule::where_clause => predicate = Some(walk_where(child)?),
            _ => {}
        }
    }
    Ok(MatchClause {
        optional,
        paths,
        predicate,
    })
}

fn walk_path(pair: Pair<'_, Rule>) -> Result<PathPattern, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner().peekable();
    let variable = if inner
        .peek()
        .is_some_and(|pair| pair.as_rule() == Rule::identifier)
    {
        Some(walk_identifier(inner.next().expect("peeked identifier")))
    } else {
        None
    };
    let start_pair = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "path has no start node"))?;
    let start = walk_node(start_pair)?;
    let mut steps = Vec::new();
    while let Some(relationship) = inner.next() {
        let relationship = walk_relationship(relationship)?;
        let node = inner
            .next()
            .ok_or_else(|| ParseError::at(span, "relationship has no target node"))?;
        steps.push((relationship, walk_node(node)?));
    }
    Ok(PathPattern {
        variable,
        start,
        steps,
        span,
    })
}

fn walk_node(pair: Pair<'_, Rule>) -> Result<NodePattern, ParseError> {
    let span = pair_span(&pair);
    let mut variable = None;
    let mut labels = Vec::new();
    let mut properties = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => variable = Some(walk_identifier(child)),
            Rule::node_labels => {
                labels.extend(child.into_inner().map(walk_identifier));
            }
            Rule::map_literal => properties = walk_map(child)?,
            rule => return Err(unexpected(&child, "node component", rule)),
        }
    }
    Ok(NodePattern {
        variable,
        labels,
        properties,
        span,
    })
}

fn walk_relationship(pair: Pair<'_, Rule>) -> Result<RelationshipPattern, ParseError> {
    let span = pair_span(&pair);
    let child = only_child(pair)?;
    let direction = match child.as_rule() {
        Rule::incoming_relationship => Direction::Incoming,
        Rule::outgoing_relationship => Direction::Outgoing,
        Rule::undirected_relationship => Direction::Both,
        rule => return Err(unexpected(&child, "relationship", rule)),
    };
    let mut variable = None;
    let mut types = Vec::new();
    let mut range = None;
    let mut properties = Vec::new();
    for body in child.into_inner() {
        if body.as_rule() != Rule::relationship_body {
            continue;
        }
        for item in body.into_inner() {
            match item.as_rule() {
                Rule::identifier => variable = Some(walk_identifier(item)),
                Rule::relationship_types => {
                    types.extend(item.into_inner().map(walk_identifier));
                }
                Rule::range_literal => {
                    let item_span = pair_span(&item);
                    range = Some(Spanned::new(
                        parse_range(item.as_str(), item_span)?,
                        item_span,
                    ));
                }
                Rule::map_literal => properties = walk_map(item)?,
                rule => return Err(unexpected(&item, "relationship component", rule)),
            }
        }
    }
    Ok(RelationshipPattern {
        variable,
        types,
        direction,
        range,
        properties,
        span,
    })
}

fn parse_range(text: &str, span: Span) -> Result<RelationshipRange, ParseError> {
    let body = text
        .strip_prefix('*')
        .ok_or_else(|| ParseError::at(span, "invalid range"))?;
    if let Some((minimum, maximum)) = body.split_once("..") {
        return Ok(RelationshipRange {
            min: parse_optional_u32(minimum, span)?,
            max: parse_optional_u32(maximum, span)?,
        });
    }
    if body.is_empty() {
        return Ok(RelationshipRange {
            min: None,
            max: None,
        });
    }
    let exact = parse_u32(body, span)?;
    Ok(RelationshipRange {
        min: Some(exact),
        max: Some(exact),
    })
}

fn parse_optional_u32(text: &str, span: Span) -> Result<Option<u32>, ParseError> {
    if text.is_empty() {
        Ok(None)
    } else {
        parse_u32(text, span).map(Some)
    }
}

fn parse_u32(text: &str, span: Span) -> Result<u32, ParseError> {
    text.replace('_', "").parse().map_err(|_| {
        ParseError::at(
            span,
            "relationship range is outside the supported u32 range",
        )
    })
}

fn walk_projection_clause(pair: Pair<'_, Rule>) -> Result<ProjectionClause, ParseError> {
    let mut distinct = false;
    let mut items = Vec::new();
    let mut predicate = None;
    let mut order_by = Vec::new();
    let mut skip = None;
    let mut limit = None;
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::DISTINCT => distinct = true,
            Rule::projection_items => items = walk_projection_items(child)?,
            Rule::where_clause => predicate = Some(walk_where(child)?),
            Rule::projection_tail => {
                for tail in child.into_inner() {
                    match tail.as_rule() {
                        Rule::order_by_clause => {
                            order_by = tail
                                .into_inner()
                                .filter(|item| item.as_rule() == Rule::sort_item)
                                .map(walk_sort_item)
                                .collect::<Result<Vec<_>, _>>()?;
                        }
                        Rule::skip_clause => skip = Some(walk_tail_expression(tail)?),
                        Rule::limit_clause => limit = Some(walk_tail_expression(tail)?),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    Ok(ProjectionClause {
        distinct,
        items,
        predicate,
        order_by,
        skip,
        limit,
    })
}

fn walk_sort_item(pair: Pair<'_, Rule>) -> Result<SortItem, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let expression = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "ORDER BY item has no expression"))?;
    let expression = walk_expression(expression)?;
    let descending = inner.any(|item| item.as_rule() == Rule::DESC);
    Ok(SortItem {
        expression,
        descending,
    })
}

fn walk_tail_expression(pair: Pair<'_, Rule>) -> Result<Spanned<Expression>, ParseError> {
    let span = pair_span(&pair);
    let expression = pair
        .into_inner()
        .find(|item| item.as_rule() == Rule::expression)
        .ok_or_else(|| ParseError::at(span, "clause has no expression"))?;
    walk_expression(expression)
}

fn walk_projection_items(pair: Pair<'_, Rule>) -> Result<Vec<ProjectionItem>, ParseError> {
    pair.into_inner()
        .map(|item| match item.as_rule() {
            Rule::projection_all => Ok(ProjectionItem::All(pair_span(&item))),
            Rule::projection_item => walk_projection_item(item),
            rule => Err(unexpected(&item, "projection", rule)),
        })
        .collect()
}

fn walk_projection_item(pair: Pair<'_, Rule>) -> Result<ProjectionItem, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let expression = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "projection has no expression"))?;
    let expression = walk_expression(expression)?;
    let alias = inner
        .find(|item| item.as_rule() == Rule::identifier)
        .map(walk_identifier);
    Ok(ProjectionItem::Expression { expression, alias })
}

fn walk_where(pair: Pair<'_, Rule>) -> Result<Spanned<Expression>, ParseError> {
    let span = pair_span(&pair);
    let expression = pair
        .into_inner()
        .find(|item| item.as_rule() == Rule::expression)
        .ok_or_else(|| ParseError::at(span, "WHERE has no expression"))?;
    walk_expression(expression)
}

fn walk_expression(pair: Pair<'_, Rule>) -> Result<Spanned<Expression>, ParseError> {
    let span = pair_span(&pair);
    let value = match pair.as_rule() {
        Rule::expression
        | Rule::or_expression
        | Rule::and_expression
        | Rule::comparison_expression
        | Rule::additive_expression
        | Rule::multiplicative_expression => walk_binary(pair)?,
        Rule::postfix_expression => walk_postfix(pair)?,
        Rule::primary_expression => return walk_expression(only_child(pair)?),
        Rule::literal => return walk_expression(only_child(pair)?),
        Rule::integer => Expression::Literal(Literal::Integer(parse_i64(&pair)?)),
        Rule::real => Expression::Literal(Literal::Real(parse_f64(&pair)?)),
        Rule::string => Expression::Literal(Literal::Text(parse_string(pair.as_str()))),
        Rule::TRUE => Expression::Literal(Literal::Boolean(true)),
        Rule::FALSE => Expression::Literal(Literal::Boolean(false)),
        Rule::NULL => Expression::Literal(Literal::Null),
        Rule::identifier => Expression::Variable(identifier_text(pair.as_str())),
        Rule::parameter => Expression::Parameter(pair.as_str()[1..].to_owned()),
        Rule::function_call => walk_function(pair)?,
        Rule::list_literal => Expression::List(
            pair.into_inner()
                .filter(|item| item.as_rule() == Rule::expression_list)
                .flat_map(Pair::into_inner)
                .map(walk_expression)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        rule => return Err(unexpected(&pair, "expression", rule)),
    };
    Ok(Spanned::new(value, span))
}

fn walk_binary(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "expression has no operand"))?;
    let mut left = walk_expression(first)?;
    while let Some(operator) = inner.next() {
        let right = inner
            .next()
            .ok_or_else(|| ParseError::at(span, "operator has no right operand"))?;
        let right = walk_expression(right)?;
        let combined_span = Span::new(left.span.start, right.span.end);
        left = Spanned::new(
            Expression::Binary {
                left: Box::new(left),
                operator: binary_operator(&operator)?,
                right: Box::new(right),
            },
            combined_span,
        );
    }
    Ok(left.value)
}

fn binary_operator(pair: &Pair<'_, Rule>) -> Result<BinaryOperator, ParseError> {
    match pair.as_str().to_ascii_lowercase().as_str() {
        "or" => Ok(BinaryOperator::Or),
        "and" => Ok(BinaryOperator::And),
        "=" => Ok(BinaryOperator::Equal),
        "<>" | "!=" => Ok(BinaryOperator::NotEqual),
        "<" => Ok(BinaryOperator::Less),
        "<=" => Ok(BinaryOperator::LessOrEqual),
        ">" => Ok(BinaryOperator::Greater),
        ">=" => Ok(BinaryOperator::GreaterOrEqual),
        "+" => Ok(BinaryOperator::Add),
        "-" => Ok(BinaryOperator::Subtract),
        "*" => Ok(BinaryOperator::Multiply),
        "/" => Ok(BinaryOperator::Divide),
        _ => Err(ParseError::at(
            pair_span(pair),
            "unsupported binary operator",
        )),
    }
}

fn walk_postfix(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let primary = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "expression has no value"))?;
    let mut expression = walk_expression(primary)?;
    for suffix in inner {
        let name = suffix
            .into_inner()
            .next()
            .ok_or_else(|| ParseError::at(span, "property access has no name"))?;
        let name = walk_identifier(name);
        let property_span = Span::new(expression.span.start, name.span.end);
        expression = Spanned::new(
            Expression::Property {
                entity: Box::new(expression),
                name,
            },
            property_span,
        );
    }
    Ok(expression.value)
}

fn walk_function(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let name = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "function has no name"))?;
    let name = walk_identifier(name);
    let mut distinct = false;
    let mut arguments = Vec::new();
    for item in inner {
        match item.as_rule() {
            Rule::DISTINCT => distinct = true,
            Rule::expression_list => {
                arguments = item
                    .into_inner()
                    .map(walk_expression)
                    .collect::<Result<Vec<_>, _>>()?;
            }
            _ => {}
        }
    }
    Ok(Expression::Function {
        name,
        arguments,
        distinct,
    })
}

fn walk_map(pair: Pair<'_, Rule>) -> Result<ParsedProperties, ParseError> {
    pair.into_inner()
        .map(|entry| {
            let span = pair_span(&entry);
            let mut inner = entry.into_inner();
            let name = inner
                .next()
                .ok_or_else(|| ParseError::at(span, "map entry has no name"))?;
            let value = inner
                .next()
                .ok_or_else(|| ParseError::at(span, "map entry has no value"))?;
            Ok((walk_identifier(name), walk_expression(value)?))
        })
        .collect()
}

fn walk_identifier(pair: Pair<'_, Rule>) -> Spanned<String> {
    let span = pair_span(&pair);
    Spanned::new(identifier_text(pair.as_str()), span)
}

fn identifier_text(text: &str) -> String {
    text.strip_prefix('`')
        .and_then(|value| value.strip_suffix('`'))
        .unwrap_or(text)
        .replace("``", "`")
}

fn parse_i64(pair: &Pair<'_, Rule>) -> Result<i64, ParseError> {
    pair.as_str().replace('_', "").parse().map_err(|_| {
        ParseError::at(
            pair_span(pair),
            "integer literal is outside the supported i64 range",
        )
    })
}

fn parse_f64(pair: &Pair<'_, Rule>) -> Result<f64, ParseError> {
    pair.as_str()
        .parse()
        .map_err(|_| ParseError::at(pair_span(pair), "invalid real literal"))
}

fn parse_string(text: &str) -> String {
    let body = text
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(text);
    body.replace("''", "'").replace("\\'", "'")
}

fn only_child(pair: Pair<'_, Rule>) -> Result<Pair<'_, Rule>, ParseError> {
    let span = pair_span(&pair);
    pair.into_inner()
        .next()
        .ok_or_else(|| ParseError::at(span, "syntax node has no value"))
}

fn pair_span(pair: &Pair<'_, Rule>) -> Span {
    let span = pair.as_span();
    Span::new(span.start(), span.end())
}

fn unexpected(pair: &Pair<'_, Rule>, expected: &str, actual: Rule) -> ParseError {
    ParseError::at(
        pair_span(pair),
        format!("expected {expected}, found {actual:?}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relationship(query: &str) -> RelationshipPattern {
        let query = parse(query).expect("query should parse");
        let Clause::Match(clause) = &query.clauses[0].value else {
            panic!("expected MATCH")
        };
        clause.paths[0].steps[0].0.clone()
    }

    #[test]
    fn preserves_fixed_directions_and_bounded_range() {
        let relationship = relationship("MATCH (a:Person)-[r:KNOWS*2..5]->(b) RETURN a");
        assert_eq!(relationship.direction, Direction::Outgoing);
        assert_eq!(
            relationship.range.expect("range").value,
            RelationshipRange {
                min: Some(2),
                max: Some(5)
            }
        );
    }

    #[test]
    fn distinguishes_exact_and_open_ranges() {
        let exact = relationship("MATCH (a)-[:KNOWS*3]->(b) RETURN b")
            .range
            .expect("range")
            .value;
        let upper = relationship("MATCH (a)-[:KNOWS*..5]->(b) RETURN b")
            .range
            .expect("range")
            .value;
        let lower = relationship("MATCH (a)-[:KNOWS*2..]->(b) RETURN b")
            .range
            .expect("range")
            .value;
        assert_eq!(
            exact,
            RelationshipRange {
                min: Some(3),
                max: Some(3)
            }
        );
        assert_eq!(
            upper,
            RelationshipRange {
                min: None,
                max: Some(5)
            }
        );
        assert_eq!(
            lower,
            RelationshipRange {
                min: Some(2),
                max: None
            }
        );
    }

    #[test]
    fn parses_with_scope_property_parameter_and_predicate() {
        let query = parse(
            "MATCH (p:Person {id: $id}) WITH p AS person WHERE person.age >= 18 RETURN person.name",
        )
        .expect("query");
        assert_eq!(query.clauses.len(), 3);
        assert!(matches!(query.clauses[1].value, Clause::With(_)));
        assert!(matches!(query.clauses[2].value, Clause::Return(_)));
    }

    #[test]
    fn parses_unwind_list_ordering_and_pagination() {
        let query =
            parse("UNWIND [1, 2, 3] AS x RETURN x ORDER BY x DESC SKIP 1 LIMIT 2").expect("query");
        let Clause::Unwind(unwind) = &query.clauses[0].value else {
            panic!("expected UNWIND")
        };
        assert_eq!(unwind.alias.value, "x");
        assert!(matches!(
            unwind.expression.value,
            Expression::List(ref values) if values.len() == 3
        ));
        let Clause::Return(projection) = &query.clauses[1].value else {
            panic!("expected RETURN")
        };
        assert_eq!(projection.order_by.len(), 1);
        assert!(projection.order_by[0].descending);
        assert!(projection.skip.is_some());
        assert!(projection.limit.is_some());
    }

    #[test]
    fn reports_invalid_and_overflow_ranges_without_panicking() {
        assert!(parse("MATCH (a)-[:R*-1]->(b) RETURN b").is_err());
        let error =
            parse("MATCH (a)-[:R*999999999999999999999]->(b) RETURN b").expect_err("overflow");
        assert!(error.message.contains("u32"));
    }

    #[test]
    fn reports_malformed_input_with_a_byte_span() {
        let error = parse("MATCH (a)-[r:R]-> RETURN a").expect_err("invalid query");
        assert!(error.span_start <= error.span_end);
    }
}
