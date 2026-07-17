use crate::{BindingId, Nullability, PropertyId};

/// Frontend-neutral value categories used during binding and planning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueType {
    Any,
    Boolean,
    Integer,
    Real,
    Text,
    Bytes,
    Node,
    Relationship,
    Path,
    List(Box<ValueType>),
}

/// Literal data owned by graph IR rather than a storage-engine value type.
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
    IsNull,
    IsNotNull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
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
    And,
    Or,
    In,
}

/// Bound function identity. Resolution happens before this reaches execution.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FunctionName(String);

impl FunctionName {
    pub fn new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        (!name.trim().is_empty()).then_some(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Expression whose names and properties have already been resolved.
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Literal(Literal),
    Binding(BindingId),
    Property {
        entity: BindingId,
        property: PropertyId,
    },
    Parameter(String),
    Unary {
        op: UnaryOp,
        expression: Box<TypedExpression>,
    },
    Binary {
        left: Box<TypedExpression>,
        op: BinaryOp,
        right: Box<TypedExpression>,
    },
    Function {
        function: FunctionName,
        arguments: Vec<TypedExpression>,
    },
    List(Vec<TypedExpression>),
}

/// Expression plus the type/nullability established by the binder.
#[derive(Clone, Debug, PartialEq)]
pub struct TypedExpression {
    pub expression: Expression,
    pub value_type: ValueType,
    pub nullability: Nullability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NullOrder {
    First,
    Last,
}
