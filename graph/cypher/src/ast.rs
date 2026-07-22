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
    pub unions: Vec<UnionBranch>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UnionBranch {
    pub all: bool,
    pub clauses: Vec<Spanned<Clause>>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Clause {
    Match(MatchClause),
    Create(CreateClause),
    Merge(MergeClause),
    Set(SetClause),
    Remove(RemoveClause),
    Delete(DeleteClause),
    Unwind(UnwindClause),
    With(ProjectionClause),
    Return(ProjectionClause),
    Foreach(ForeachClause),
    Call(CallClause),
    /// `CALL { ... }` scoped subquery whose RETURN feeds the outer scope.
    CallSubquery(Box<Query>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CallClause {
    pub name: Spanned<String>,
    pub arguments: Vec<Spanned<Expression>>,
    pub yields: Vec<Spanned<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForeachClause {
    pub variable: Spanned<String>,
    pub list: Spanned<Expression>,
    pub body: Vec<Spanned<Clause>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CreateClause {
    pub paths: Vec<PathPattern>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MergeClause {
    pub path: PathPattern,
    pub on_create: Vec<SetItem>,
    pub on_match: Vec<SetItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyTarget {
    pub variable: Spanned<String>,
    pub property: Spanned<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SetItem {
    /// `SET n.prop = value`
    Property {
        target: PropertyTarget,
        value: Spanned<Expression>,
    },
    /// `SET n = value` — replaces every property.
    ReplaceEntity {
        variable: Spanned<String>,
        value: Spanned<Expression>,
    },
    /// `SET n += value` — merges properties, keeping the rest.
    MergeEntity {
        variable: Spanned<String>,
        value: Spanned<Expression>,
    },
    /// `SET n:Label1:Label2`
    Labels {
        variable: Spanned<String>,
        labels: Vec<Spanned<String>>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct SetClause {
    pub items: Vec<SetItem>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoveClause {
    pub items: Vec<PropertyTarget>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeleteClause {
    pub detach: bool,
    pub variables: Vec<Spanned<String>>,
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
    /// True when a `{...}` map was written, even if it was empty; some
    /// binder rules distinguish `(n)` from `(n {})`.
    pub has_property_map: bool,
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
    Xor,
    And,
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    In,
    StartsWith,
    EndsWith,
    Contains,
    /// Apache AGE agtype concatenation (`||`).
    Concat,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    /// pgvector `<->` (L2 distance).
    VectorL2,
    /// pgvector `<=>` (cosine distance).
    VectorCosine,
    /// pgvector `<#>` (negative inner product).
    VectorInnerProduct,
    /// jsonb `->` (field/index access, JSON result).
    JsonGet,
    /// jsonb `->>` (field/index access, text result).
    JsonGetText,
    /// jsonb `#>` (path extraction).
    JsonPath,
    /// jsonb `#>>` (path extraction, text result).
    JsonPathText,
    /// jsonb `?` (key existence).
    JsonExists,
    /// jsonb `?|` (any key exists).
    JsonExistsAny,
    /// jsonb `?&` (all keys exist).
    JsonExistsAll,
    /// jsonb `@>` (contains).
    JsonContains,
    /// jsonb `<@` (contained by).
    JsonContainedBy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOperator {
    Not,
    IsNull,
    IsNotNull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuantifierKind {
    All,
    Any,
    None,
    Single,
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
    Unary {
        operator: UnaryOperator,
        operand: Box<Spanned<Expression>>,
    },
    Case {
        subject: Option<Box<Spanned<Expression>>>,
        branches: Vec<(Spanned<Expression>, Spanned<Expression>)>,
        default: Option<Box<Spanned<Expression>>>,
    },
    Quantifier {
        kind: QuantifierKind,
        variable: Spanned<String>,
        list: Box<Spanned<Expression>>,
        predicate: Box<Spanned<Expression>>,
    },
    /// `reduce(acc = init, x IN list | expr)` — a left fold over a list.
    Reduce {
        accumulator: Spanned<String>,
        initial: Box<Spanned<Expression>>,
        variable: Spanned<String>,
        list: Box<Spanned<Expression>>,
        expression: Box<Spanned<Expression>>,
    },
    PatternSubquery {
        count: bool,
        paths: Vec<PathPattern>,
        predicate: Option<Box<Spanned<Expression>>>,
    },
    PatternPredicate {
        path: Box<PathPattern>,
    },
    HasLabels {
        operand: Box<Spanned<Expression>>,
        labels: Vec<Spanned<String>>,
    },
    Index {
        base: Box<Spanned<Expression>>,
        index: Box<Spanned<Expression>>,
    },
    Slice {
        base: Box<Spanned<Expression>>,
        from: Option<Box<Spanned<Expression>>>,
        to: Option<Box<Spanned<Expression>>>,
    },
    Cast {
        operand: Box<Spanned<Expression>>,
        type_name: Spanned<String>,
    },
    ListComprehension {
        variable: Spanned<String>,
        list: Box<Spanned<Expression>>,
        predicate: Option<Box<Spanned<Expression>>>,
        map: Option<Box<Spanned<Expression>>>,
    },
    Function {
        name: Spanned<String>,
        arguments: Vec<Spanned<Expression>>,
        distinct: bool,
        /// True for a `*` argument as in `count(*)`.
        star: bool,
    },
    List(Vec<Spanned<Expression>>),
    Map(Vec<(Spanned<String>, Spanned<Expression>)>),
}
