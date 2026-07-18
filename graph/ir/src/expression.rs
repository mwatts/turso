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
    /// A `CREATE TYPE name BASE <primitive> ENCODE ... DECODE ...` scalar.
    /// `base` is the underlying storage primitive (e.g. `Bytes` for a BLOB
    /// custom type).
    Custom {
        name: String,
        base: Box<ValueType>,
    },
    /// A `CREATE TYPE name AS STRUCT(...)` composite, fields in declared order.
    Struct(Vec<(String, ValueType)>),
    /// A `CREATE TYPE name AS UNION(...)` tagged union, variants in declared order.
    Union(Vec<(String, ValueType)>),
    /// The result of a typed vector function call (`vector32`, `vector64`, ...).
    /// Dims are known only when statically determinable from the call site —
    /// never from a column declaration, since no schema-level VECTOR column
    /// type exists (see `docs/plans/2026-07-17-graph-type-system-design.md`).
    Vector(VectorKind, Option<u32>),
}

/// Mirrors `core::vector::vector_types::VectorType`'s 5 encodings. Kept local
/// (not reused from `core`) because `graph/ir` has zero `turso_core`
/// dependency; this enum carries no behavior, only the closed set of vector
/// encodings the typed function registry (Task 11) can produce.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorKind {
    Float32Dense,
    Float64Dense,
    Float32Sparse,
    Float1Bit,
    Float8,
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

#[cfg(test)]
mod value_type_tests {
    use super::*;

    #[test]
    fn struct_and_union_value_types_compare_by_field_shape() {
        let a = ValueType::Struct(vec![
            ("x".to_owned(), ValueType::Integer),
            ("y".to_owned(), ValueType::Integer),
        ]);
        let b = ValueType::Struct(vec![
            ("x".to_owned(), ValueType::Integer),
            ("y".to_owned(), ValueType::Integer),
        ]);
        assert_eq!(a, b);

        let point = ValueType::Custom {
            name: "point".to_owned(),
            base: Box::new(ValueType::Bytes),
        };
        assert_eq!(point.clone(), point);

        let tagged = ValueType::Union(vec![("ok".to_owned(), ValueType::Text)]);
        assert_ne!(a, tagged);
    }

    #[test]
    fn vector_value_type_carries_kind_and_optional_dims() {
        let dense = ValueType::Vector(VectorKind::Float32Dense, Some(3));
        let unknown_dims = ValueType::Vector(VectorKind::Float32Dense, None);
        assert_ne!(dense, unknown_dims);
        assert_eq!(VectorKind::Float32Dense, VectorKind::Float32Dense);
    }
}
