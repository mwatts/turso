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
    /// A general key/value map with no declared field types.
    Map,
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
pub enum QuantifierKind {
    All,
    Any,
    None,
    Single,
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
    Modulo,
    Power,
    And,
    Or,
    Xor,
    In,
    StartsWith,
    EndsWith,
    Contains,
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
        fields: Vec<String>,
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
    Case {
        subject: Option<Box<TypedExpression>>,
        branches: Vec<(TypedExpression, TypedExpression)>,
        default: Option<Box<TypedExpression>>,
    },
    /// The loop variable of the list scope at the recorded depth (1-based).
    /// Only the innermost scope is addressable; the binder enforces this.
    ListElement(usize),
    /// A fixed-length path value: the ordered node and relationship
    /// bindings the path traverses.
    PathValue {
        nodes: Vec<BindingId>,
        relationships: Vec<BindingId>,
    },
    /// EXISTS/COUNT over a correlated graph-pattern subquery. Correlations
    /// pair an outer-scope binding with the subquery binding carrying the
    /// same user variable; lowering equates their identity columns.
    PatternSubquery {
        count: bool,
        plan: Box<crate::Plan>,
        correlations: Vec<(BindingId, BindingId)>,
    },
    Index {
        base: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },
    Slice {
        base: Box<TypedExpression>,
        from: Option<Box<TypedExpression>>,
        to: Option<Box<TypedExpression>>,
    },
    Cast {
        expression: Box<TypedExpression>,
        target: ValueType,
    },
    Quantifier {
        kind: QuantifierKind,
        /// Scope depth of this quantifier's loop variable (1-based).
        depth: usize,
        list: Box<TypedExpression>,
        predicate: Box<TypedExpression>,
    },
    ListComprehension {
        /// Scope depth of this comprehension's loop variable (1-based).
        depth: usize,
        list: Box<TypedExpression>,
        predicate: Option<Box<TypedExpression>>,
        map: Option<Box<TypedExpression>>,
    },
    Function {
        function: FunctionName,
        arguments: Vec<TypedExpression>,
    },
    List(Vec<TypedExpression>),
    /// A `{field: value}` literal bound against a resolved STRUCT/UNION
    /// property target. Entries preserve the literal's source order as
    /// written by the user, not the target's declared field/variant order —
    /// consumers that need declared order (e.g. lowering) must look up
    /// entries by name rather than by position.
    Map(Vec<(String, TypedExpression)>),
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

    #[test]
    fn map_expression_holds_ordered_field_bindings() {
        let map = Expression::Map(vec![(
            "x".to_owned(),
            TypedExpression {
                expression: Expression::Literal(Literal::Integer(1)),
                value_type: ValueType::Integer,
                nullability: Nullability::NonNull,
            },
        )]);
        match map {
            Expression::Map(entries) => assert_eq!(entries.len(), 1),
            _ => panic!("expected Map"),
        }
    }

    #[test]
    fn property_expression_carries_nested_field_chain() {
        let property = Expression::Property {
            entity: BindingId::new(1).unwrap(),
            property: PropertyId::new(1).unwrap(),
            fields: vec!["address".to_owned(), "city".to_owned()],
        };
        match property {
            Expression::Property { fields, .. } => assert_eq!(fields.len(), 2),
            _ => panic!("expected Property"),
        }
    }
}
