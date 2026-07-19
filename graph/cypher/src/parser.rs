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
    BinaryOperator, Clause, CreateClause, DeleteClause, Direction, Expression, Literal,
    MatchClause, MergeClause, NodePattern, PathPattern, ProjectionClause, ProjectionItem,
    PropertyTarget, QuantifierKind, Query, RelationshipPattern, RelationshipRange, RemoveClause,
    SetClause, SetItem, SortItem, Span, Spanned, UnaryOperator, UnwindClause,
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
        Rule::create_clause => Clause::Create(walk_create(pair)?),
        Rule::merge_clause => Clause::Merge(walk_merge(pair)?),
        Rule::set_clause => Clause::Set(walk_set(pair)?),
        Rule::remove_clause => Clause::Remove(walk_remove(pair)?),
        Rule::delete_clause => Clause::Delete(walk_delete(pair)?),
        Rule::unwind_clause => Clause::Unwind(walk_unwind(pair)?),
        Rule::with_clause => Clause::With(walk_projection_clause(pair)?),
        Rule::return_clause => Clause::Return(walk_projection_clause(pair)?),
        rule => return Err(unexpected(&pair, "clause", rule)),
    };
    Ok(Spanned::new(clause, span))
}

fn walk_create(pair: Pair<'_, Rule>) -> Result<CreateClause, ParseError> {
    let span = pair_span(&pair);
    let pattern = pair
        .into_inner()
        .find(|item| item.as_rule() == Rule::pattern)
        .ok_or_else(|| ParseError::at(span, "CREATE has no pattern"))?;
    let paths = pattern
        .into_inner()
        .filter(|item| item.as_rule() == Rule::path_pattern)
        .map(walk_path)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CreateClause { paths })
}

fn walk_merge(pair: Pair<'_, Rule>) -> Result<MergeClause, ParseError> {
    let span = pair_span(&pair);
    let path = pair
        .into_inner()
        .find(|item| item.as_rule() == Rule::path_pattern)
        .ok_or_else(|| ParseError::at(span, "MERGE has no pattern"))?;
    Ok(MergeClause {
        path: walk_path(path)?,
    })
}

fn walk_property_target(pair: Pair<'_, Rule>) -> Result<PropertyTarget, ParseError> {
    let span = pair_span(&pair);
    let mut identifiers = pair.into_inner();
    let variable = identifiers
        .next()
        .ok_or_else(|| ParseError::at(span, "property target has no variable"))?;
    let property = identifiers
        .next()
        .ok_or_else(|| ParseError::at(span, "property target has no property"))?;
    Ok(PropertyTarget {
        variable: walk_identifier(variable),
        property: walk_identifier(property),
    })
}

fn walk_set(pair: Pair<'_, Rule>) -> Result<SetClause, ParseError> {
    let items = pair
        .into_inner()
        .filter(|item| item.as_rule() == Rule::set_item)
        .map(|item| {
            let span = pair_span(&item);
            let mut inner = item.into_inner();
            let target = inner
                .next()
                .ok_or_else(|| ParseError::at(span, "SET item has no target"))?;
            let value = inner
                .next()
                .ok_or_else(|| ParseError::at(span, "SET item has no value"))?;
            Ok(SetItem {
                target: walk_property_target(target)?,
                value: walk_expression(value)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SetClause { items })
}

fn walk_remove(pair: Pair<'_, Rule>) -> Result<RemoveClause, ParseError> {
    let items = pair
        .into_inner()
        .filter(|item| item.as_rule() == Rule::property_target)
        .map(walk_property_target)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RemoveClause { items })
}

fn walk_delete(pair: Pair<'_, Rule>) -> Result<DeleteClause, ParseError> {
    let detach = pair
        .clone()
        .into_inner()
        .any(|item| item.as_rule() == Rule::DETACH);
    let variables = pair
        .into_inner()
        .filter(|item| item.as_rule() == Rule::identifier)
        .map(walk_identifier)
        .collect();
    Ok(DeleteClause { detach, variables })
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
        | Rule::multiplicative_expression
        | Rule::power_expression => walk_binary(pair)?,
        Rule::not_expression => walk_not(pair)?,
        Rule::predicate_expression => walk_predicate(pair)?,
        Rule::postfix_expression => walk_postfix(pair)?,
        Rule::primary_expression => return walk_expression(only_child(pair)?),
        Rule::literal => return walk_expression(only_child(pair)?),
        Rule::integer => Expression::Literal(Literal::Integer(parse_i64(&pair)?)),
        Rule::hex_integer => Expression::Literal(Literal::Integer(parse_radix(&pair, 16)?)),
        Rule::octal_integer => Expression::Literal(Literal::Integer(parse_radix(&pair, 8)?)),
        Rule::real => Expression::Literal(Literal::Real(parse_f64(&pair)?)),
        Rule::string => Expression::Literal(Literal::Text(parse_string(&pair)?)),
        Rule::TRUE => Expression::Literal(Literal::Boolean(true)),
        Rule::FALSE => Expression::Literal(Literal::Boolean(false)),
        Rule::NULL => Expression::Literal(Literal::Null),
        Rule::identifier => Expression::Variable(identifier_text(pair.as_str())),
        Rule::parameter => Expression::Parameter(pair.as_str()[1..].to_owned()),
        Rule::function_call => walk_function(pair)?,
        Rule::case_expression => walk_case(pair)?,
        Rule::quantifier_expression => walk_quantifier(pair)?,
        Rule::list_comprehension => walk_list_comprehension(pair)?,
        Rule::list_literal => Expression::List(
            pair.into_inner()
                .filter(|item| item.as_rule() == Rule::expression_list)
                .flat_map(Pair::into_inner)
                .map(walk_expression)
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Rule::map_literal => Expression::Map(walk_map(pair)?),
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
        "xor" => Ok(BinaryOperator::Xor),
        "and" => Ok(BinaryOperator::And),
        "=" => Ok(BinaryOperator::Equal),
        "<>" | "!=" => Ok(BinaryOperator::NotEqual),
        "<" => Ok(BinaryOperator::Less),
        "<=" => Ok(BinaryOperator::LessOrEqual),
        ">" => Ok(BinaryOperator::Greater),
        ">=" => Ok(BinaryOperator::GreaterOrEqual),
        "in" => Ok(BinaryOperator::In),
        "+" => Ok(BinaryOperator::Add),
        "-" => Ok(BinaryOperator::Subtract),
        "*" => Ok(BinaryOperator::Multiply),
        "/" => Ok(BinaryOperator::Divide),
        "%" => Ok(BinaryOperator::Modulo),
        "^" => Ok(BinaryOperator::Power),
        _ => Err(ParseError::at(
            pair_span(pair),
            "unsupported binary operator",
        )),
    }
}

fn walk_not(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut negations = 0usize;
    let mut operand = None;
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::not_op => negations += 1,
            _ => operand = Some(walk_expression(child)?),
        }
    }
    let operand = operand.ok_or_else(|| ParseError::at(span, "NOT has no operand"))?;
    let mut expression = operand;
    for _ in 0..negations {
        expression = Spanned::new(
            Expression::Unary {
                operator: UnaryOperator::Not,
                operand: Box::new(expression),
            },
            span,
        );
    }
    Ok(expression.value)
}

fn walk_quantifier(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut kind = None;
    let mut variable = None;
    let mut expressions = Vec::new();
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::quantifier_kind => {
                kind = Some(match child.as_str().to_ascii_lowercase().as_str() {
                    "all" => QuantifierKind::All,
                    "any" => QuantifierKind::Any,
                    "none" => QuantifierKind::None,
                    _ => QuantifierKind::Single,
                });
            }
            Rule::identifier => variable = Some(walk_identifier(child)),
            Rule::expression => expressions.push(walk_expression(child)?),
            _ => {}
        }
    }
    let kind = kind.ok_or_else(|| ParseError::at(span, "quantifier has no kind"))?;
    let variable = variable.ok_or_else(|| ParseError::at(span, "quantifier has no variable"))?;
    let mut expressions = expressions.into_iter();
    let list = expressions
        .next()
        .ok_or_else(|| ParseError::at(span, "quantifier has no list"))?;
    let predicate = expressions
        .next()
        .ok_or_else(|| ParseError::at(span, "quantifier has no predicate"))?;
    Ok(Expression::Quantifier {
        kind,
        variable,
        list: Box::new(list),
        predicate: Box::new(predicate),
    })
}

fn walk_list_comprehension(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut variable = None;
    let mut list = None;
    let mut predicate = None;
    let mut map = None;
    let mut after_where = false;
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::identifier => variable = Some(walk_identifier(child)),
            Rule::WHERE => after_where = true,
            Rule::expression => {
                let expression = walk_expression(child)?;
                if list.is_none() {
                    list = Some(expression);
                } else if after_where && predicate.is_none() {
                    predicate = Some(Box::new(expression));
                } else {
                    map = Some(Box::new(expression));
                }
            }
            _ => {}
        }
    }
    let variable =
        variable.ok_or_else(|| ParseError::at(span, "list comprehension has no variable"))?;
    let list = list.ok_or_else(|| ParseError::at(span, "list comprehension has no list"))?;
    Ok(Expression::ListComprehension {
        variable,
        list: Box::new(list),
        predicate,
        map,
    })
}

fn walk_case(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut subject = None;
    let mut branches = Vec::new();
    let mut default = None;
    for child in pair.into_inner() {
        match child.as_rule() {
            Rule::case_subject => {
                let expression = child
                    .into_inner()
                    .find(|item| item.as_rule() == Rule::expression)
                    .ok_or_else(|| ParseError::at(span, "CASE subject has no expression"))?;
                subject = Some(Box::new(walk_expression(expression)?));
            }
            Rule::when_clause => {
                let clause_span = pair_span(&child);
                let mut expressions = child
                    .into_inner()
                    .filter(|item| item.as_rule() == Rule::expression);
                let condition = expressions
                    .next()
                    .ok_or_else(|| ParseError::at(clause_span, "WHEN has no condition"))?;
                let result = expressions
                    .next()
                    .ok_or_else(|| ParseError::at(clause_span, "WHEN has no THEN result"))?;
                branches.push((walk_expression(condition)?, walk_expression(result)?));
            }
            Rule::case_else => {
                let expression = child
                    .into_inner()
                    .find(|item| item.as_rule() == Rule::expression)
                    .ok_or_else(|| ParseError::at(span, "ELSE has no expression"))?;
                default = Some(Box::new(walk_expression(expression)?));
            }
            _ => {}
        }
    }
    Ok(Expression::Case {
        subject,
        branches,
        default,
    })
}

fn walk_predicate(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let base = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "predicate has no operand"))?;
    let mut left = walk_expression(base)?;
    for suffix in inner {
        let suffix_span = pair_span(&suffix);
        let combined = Span::new(left.span.start, suffix_span.end);
        let suffix = only_child(suffix)?;
        let value = match suffix.as_rule() {
            Rule::in_suffix | Rule::starts_suffix | Rule::ends_suffix | Rule::contains_suffix => {
                let rule = suffix.as_rule();
                let operator = match rule {
                    Rule::in_suffix => BinaryOperator::In,
                    Rule::starts_suffix => BinaryOperator::StartsWith,
                    Rule::ends_suffix => BinaryOperator::EndsWith,
                    _ => BinaryOperator::Contains,
                };
                let right = suffix
                    .into_inner()
                    .find(|item| item.as_rule() == Rule::additive_expression)
                    .ok_or_else(|| ParseError::at(suffix_span, "predicate has no right operand"))?;
                Expression::Binary {
                    left: Box::new(left),
                    operator,
                    right: Box::new(walk_expression(right)?),
                }
            }
            Rule::null_suffix => {
                let negated = suffix
                    .into_inner()
                    .any(|item| item.as_rule() == Rule::not_op);
                Expression::Unary {
                    operator: if negated {
                        UnaryOperator::IsNotNull
                    } else {
                        UnaryOperator::IsNull
                    },
                    operand: Box::new(left),
                }
            }
            rule => return Err(unexpected(&suffix, "predicate suffix", rule)),
        };
        left = Spanned::new(value, combined);
    }
    Ok(left.value)
}

fn walk_postfix(pair: Pair<'_, Rule>) -> Result<Expression, ParseError> {
    let span = pair_span(&pair);
    let mut inner = pair.into_inner();
    let primary = inner
        .next()
        .ok_or_else(|| ParseError::at(span, "expression has no value"))?;
    let mut expression = walk_expression(primary)?;
    for suffix in inner {
        let suffix_span = pair_span(&suffix);
        let combined = Span::new(expression.span.start, suffix_span.end);
        let suffix = only_child(suffix)?;
        let value = match suffix.as_rule() {
            Rule::property_suffix => {
                let name = suffix
                    .into_inner()
                    .next()
                    .ok_or_else(|| ParseError::at(span, "property access has no name"))?;
                Expression::Property {
                    entity: Box::new(expression),
                    name: walk_identifier(name),
                }
            }
            Rule::index_suffix => walk_index_suffix(suffix, expression)?,
            Rule::cast_suffix => {
                let name = suffix
                    .into_inner()
                    .find(|item| item.as_rule() == Rule::identifier)
                    .ok_or_else(|| ParseError::at(suffix_span, "cast has no type name"))?;
                Expression::Cast {
                    operand: Box::new(expression),
                    type_name: walk_identifier(name),
                }
            }
            Rule::call_suffix => walk_call_suffix(suffix, expression, combined)?,
            rule => return Err(unexpected(&suffix, "postfix suffix", rule)),
        };
        expression = Spanned::new(value, combined);
    }
    Ok(expression.value)
}

fn walk_index_suffix(
    suffix: Pair<'_, Rule>,
    base: Spanned<Expression>,
) -> Result<Expression, ParseError> {
    let mut leading_dotdot = false;
    let mut saw_dotdot = false;
    let mut first = None;
    let mut second = None;
    for item in suffix.into_inner() {
        match item.as_rule() {
            Rule::dotdot => {
                if first.is_none() {
                    leading_dotdot = true;
                }
                saw_dotdot = true;
            }
            Rule::expression => {
                let expression = walk_expression(item)?;
                if first.is_none() {
                    first = Some(Box::new(expression));
                } else {
                    second = Some(Box::new(expression));
                }
            }
            _ => {}
        }
    }
    Ok(if leading_dotdot {
        Expression::Slice {
            base: Box::new(base),
            from: None,
            to: first,
        }
    } else if saw_dotdot {
        Expression::Slice {
            base: Box::new(base),
            from: first,
            to: second,
        }
    } else {
        let index = first.expect("grammar guarantees an index expression");
        Expression::Index {
            base: Box::new(base),
            index,
        }
    })
}

fn walk_call_suffix(
    suffix: Pair<'_, Rule>,
    target: Spanned<Expression>,
    span: Span,
) -> Result<Expression, ParseError> {
    let name = qualified_call_name(&target).ok_or_else(|| {
        ParseError::at(
            span,
            "call target is not a plain or namespaced function name",
        )
    })?;
    let distinct = suffix
        .clone()
        .into_inner()
        .any(|item| item.as_rule() == Rule::DISTINCT);
    let arguments = suffix
        .into_inner()
        .filter(|item| item.as_rule() == Rule::expression_list)
        .flat_map(Pair::into_inner)
        .map(walk_expression)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Expression::Function {
        name: Spanned::new(name, target.span),
        arguments,
        distinct,
    })
}

fn qualified_call_name(target: &Spanned<Expression>) -> Option<String> {
    match &target.value {
        Expression::Variable(name) => Some(name.clone()),
        Expression::Property { entity, name } => {
            Some(format!("{}.{}", qualified_call_name(entity)?, name.value))
        }
        _ => None,
    }
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

fn parse_radix(pair: &Pair<'_, Rule>, radix: u32) -> Result<i64, ParseError> {
    let text = pair.as_str();
    let (negative, body) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let digits = &body[2..];
    i64::from_str_radix(
        &format!("{}{digits}", if negative { "-" } else { "" }),
        radix,
    )
    .map_err(|_| {
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

fn parse_string(pair: &Pair<'_, Rule>) -> Result<String, ParseError> {
    let text = pair.as_str();
    let body = text
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(text);
    let mut out = String::with_capacity(body.len());
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        match c {
            // The grammar only admits quotes as `''` pairs; fold to one.
            '\'' => {
                chars.next();
                out.push('\'');
            }
            '\\' => {
                let escape = chars.next().ok_or_else(|| {
                    ParseError::at(pair_span(pair), "unterminated escape in string literal")
                })?;
                match escape {
                    '\\' | '\'' | '"' => out.push(escape),
                    'b' => out.push('\u{0008}'),
                    'f' => out.push('\u{000C}'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'u' => {
                        let digits: String = chars.by_ref().take(4).collect();
                        let value = (digits.len() == 4)
                            .then(|| u32::from_str_radix(&digits, 16).ok())
                            .flatten()
                            .and_then(char::from_u32)
                            .ok_or_else(|| {
                                ParseError::at(
                                    pair_span(pair),
                                    format!(
                                        "invalid unicode escape `\\u{digits}` in string literal"
                                    ),
                                )
                            })?;
                        out.push(value);
                    }
                    other => {
                        return Err(ParseError::at(
                            pair_span(pair),
                            format!("unsupported escape `\\{other}` in string literal"),
                        ))
                    }
                }
            }
            _ => out.push(c),
        }
    }
    Ok(out)
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

    fn text_literal(literal: &str) -> String {
        let query = parse(&format!("RETURN {literal} AS v")).expect("query should parse");
        let Clause::Return(clause) = &query.clauses[0].value else {
            panic!("expected RETURN")
        };
        let ProjectionItem::Expression { expression, .. } = &clause.items[0] else {
            panic!("expected expression item")
        };
        let Expression::Literal(Literal::Text(text)) = &expression.value else {
            panic!("expected text literal, got {:?}", expression.value)
        };
        text.clone()
    }

    /// String literals must interpret every escape the grammar accepts;
    /// unescaping only quotes would leak raw backslash sequences into values.
    #[test]
    fn decodes_string_escape_sequences() {
        assert_eq!(text_literal(r"'line1\nline2'"), "line1\nline2");
        assert_eq!(text_literal(r"'tab\there'"), "tab\there");
        assert_eq!(text_literal(r"'a\\b'"), "a\\b");
        assert_eq!(text_literal(r"'quote\'inner'"), "quote'inner");
        assert_eq!(text_literal("'doubled''inner'"), "doubled'inner");
        assert_eq!(text_literal(r#"'say \"hi\"'"#), "say \"hi\"");
        assert_eq!(text_literal(r"'déjà'"), "déjà");
        assert_eq!(text_literal(r"'d\u00e9j\u00e0'"), "déjà");
    }

    #[test]
    fn rejects_invalid_string_escapes() {
        parse(r"RETURN 'bad\q' AS v").expect_err("unsupported escape must fail");
        parse(r"RETURN 'bad\uZZZZ' AS v").expect_err("invalid unicode escape must fail");
    }

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
    fn parses_map_literal_as_general_expression() {
        let query = parse("RETURN {x: 1, y: 2}").expect("parses");
        let Clause::Return(projection) = &query.clauses[0].value else {
            panic!("expected RETURN")
        };
        let ProjectionItem::Expression { expression, .. } = &projection.items[0] else {
            panic!("expected expression projection item")
        };
        assert!(matches!(
            expression.value,
            Expression::Map(ref entries) if entries.len() == 2
        ));
    }

    #[test]
    fn parses_create_and_merge_patterns() {
        let query = parse(
            "CREATE (a:Person {name: 'Ada'})-[:KNOWS]->(b:Person) MERGE (b)-[:KNOWS]->(c:Person {name: 'Grace'})",
        )
        .expect("mutation query");
        let Clause::Create(create) = &query.clauses[0].value else {
            panic!("expected CREATE")
        };
        assert_eq!(create.paths.len(), 1);
        assert_eq!(create.paths[0].steps.len(), 1);
        let Clause::Merge(merge) = &query.clauses[1].value else {
            panic!("expected MERGE")
        };
        assert_eq!(merge.path.steps.len(), 1);
    }

    #[test]
    fn parses_property_updates_and_deletes_with_source_spans() {
        let source = "MATCH (n:Person) SET n.name = 'Ada', n.age = 37 REMOVE n.old DETACH DELETE n";
        let query = parse(source).expect("mutation query");
        let Clause::Set(set) = &query.clauses[1].value else {
            panic!("expected SET")
        };
        assert_eq!(set.items.len(), 2);
        assert_eq!(set.items[0].target.variable.value, "n");
        assert_eq!(set.items[0].target.property.value, "name");
        assert_eq!(
            &source[set.items[0].target.property.span.start..set.items[0].target.property.span.end],
            "name"
        );
        let Clause::Remove(remove) = &query.clauses[2].value else {
            panic!("expected REMOVE")
        };
        assert_eq!(remove.items[0].property.value, "old");
        let Clause::Delete(delete) = &query.clauses[3].value else {
            panic!("expected DELETE")
        };
        assert!(delete.detach);
        assert_eq!(delete.variables[0].value, "n");
    }

    #[test]
    fn reserves_mutation_keywords_and_rejects_unsupported_set_forms() {
        assert!(parse("MATCH (set) RETURN set").is_err());
        assert!(parse("MATCH (n) SET n = {name: 'Ada'}").is_err());
        assert!(parse("MATCH (n) SET n += {name: 'Ada'}").is_err());
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

    /// openCypher list membership: `value IN list` must parse as a binary
    /// comparison so WHERE filters over list literals can reach the binder.
    #[test]
    fn parses_in_as_list_membership_comparison() {
        let query = parse("MATCH (x) WHERE x.name IN ['B', 'C'] RETURN x").expect("query");
        let Clause::Match(clause) = &query.clauses[0].value else {
            panic!("expected MATCH")
        };
        let predicate = clause.predicate.as_ref().expect("WHERE predicate");
        let Expression::Binary {
            operator, right, ..
        } = &predicate.value
        else {
            panic!("expected binary predicate, got {:?}", predicate.value)
        };
        assert_eq!(*operator, BinaryOperator::In);
        assert!(matches!(
            right.value,
            Expression::List(ref values) if values.len() == 2
        ));
    }

    /// IN must sit at comparison precedence: additive operands group under
    /// it, and boolean connectives group above it, matching openCypher.
    #[test]
    fn in_binds_tighter_than_boolean_and_looser_than_additive() {
        let query = parse("RETURN 1 + 2 IN [3] AS r, true AND 1 IN [1] AS s").expect("query");
        let Clause::Return(projection) = &query.clauses[0].value else {
            panic!("expected RETURN")
        };
        let ProjectionItem::Expression { expression, .. } = &projection.items[0] else {
            panic!("expected expression item")
        };
        let Expression::Binary { left, operator, .. } = &expression.value else {
            panic!("expected binary expression, got {:?}", expression.value)
        };
        assert_eq!(*operator, BinaryOperator::In);
        assert!(matches!(
            left.value,
            Expression::Binary {
                operator: BinaryOperator::Add,
                ..
            }
        ));
        let ProjectionItem::Expression { expression, .. } = &projection.items[1] else {
            panic!("expected expression item")
        };
        let Expression::Binary {
            operator, right, ..
        } = &expression.value
        else {
            panic!("expected binary expression, got {:?}", expression.value)
        };
        assert_eq!(*operator, BinaryOperator::And);
        assert!(matches!(
            right.value,
            Expression::Binary {
                operator: BinaryOperator::In,
                ..
            }
        ));
    }

    /// `IN` must only match as a whole word so identifiers with an `in`
    /// prefix keep parsing as plain variables.
    #[test]
    fn keeps_identifiers_with_in_prefix_intact() {
        let query = parse("MATCH (inner) RETURN inner").expect("query");
        let Clause::Return(projection) = &query.clauses[1].value else {
            panic!("expected RETURN")
        };
        let ProjectionItem::Expression { expression, .. } = &projection.items[0] else {
            panic!("expected expression item")
        };
        assert!(matches!(
            expression.value,
            Expression::Variable(ref name) if name == "inner"
        ));
    }
}
