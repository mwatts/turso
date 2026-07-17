//! Source-oriented Cypher syntax with byte spans.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub const fn new(value: T, span: Span) -> Self {
        Self { value, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub clauses: Vec<Spanned<Clause>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Clause {
    Match(MatchClause),
    Unwind(UnwindClause),
    With(ProjectionClause),
    Return(ProjectionClause),
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnwindClause {
    pub expression: Spanned<Expression>,
    pub alias: Spanned<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchClause {
    pub optional: bool,
    pub paths: Vec<PathPattern>,
    pub predicate: Option<Spanned<Expression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PathPattern {
    pub variable: Option<Spanned<String>>,
    pub start: NodePattern,
    pub steps: Vec<(RelationshipPattern, NodePattern)>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NodePattern {
    pub variable: Option<Spanned<String>>,
    pub labels: Vec<Spanned<String>>,
    pub properties: Vec<(Spanned<String>, Spanned<Expression>)>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationshipRange {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationshipPattern {
    pub variable: Option<Spanned<String>>,
    pub types: Vec<Spanned<String>>,
    pub direction: Direction,
    pub range: Option<Spanned<RelationshipRange>>,
    pub properties: Vec<(Spanned<String>, Spanned<Expression>)>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionClause {
    pub distinct: bool,
    pub items: Vec<ProjectionItem>,
    pub predicate: Option<Spanned<Expression>>,
    pub order_by: Vec<SortItem>,
    pub skip: Option<Spanned<Expression>>,
    pub limit: Option<Spanned<Expression>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SortItem {
    pub expression: Spanned<Expression>,
    pub descending: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProjectionItem {
    All(Span),
    Expression {
        expression: Spanned<Expression>,
        alias: Option<Spanned<String>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Text(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Or,
    And,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Parameter(String),
    Property {
        entity: Box<Spanned<Expression>>,
        name: Spanned<String>,
    },
    Binary {
        left: Box<Spanned<Expression>>,
        operator: BinaryOperator,
        right: Box<Spanned<Expression>>,
    },
    Function {
        name: Spanned<String>,
        arguments: Vec<Spanned<Expression>>,
        distinct: bool,
    },
    List(Vec<Spanned<Expression>>),
}
