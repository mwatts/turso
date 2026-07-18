use turso_core::vector::operations::text::vector_from_text;
use turso_core::vector::vector_types::VectorType as CoreVectorType;
use turso_graph_ir as ir;

#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentType {
    Any,
    Exact(ir::ValueType),
    Vector,
}

impl ArgumentType {
    /// Deliberately permissive: only a clear, unambiguous mismatch is
    /// rejected. `ir::ValueType::Any` (an unresolved/unknown type) always
    /// matches, so this never rejects a previously-valid query just because
    /// the binder couldn't pin down an argument's type.
    fn matches(&self, actual: &ir::ValueType) -> bool {
        match self {
            ArgumentType::Any => true,
            ArgumentType::Exact(expected) => {
                actual == expected || matches!(actual, ir::ValueType::Any)
            }
            // A BLOB column can hold vector bytes just as validly as a
            // value actually produced by vector32/etc. Custom scalars are
            // accepted regardless of their declared base (not just a BLOB
            // base) since this match is deliberately permissive — only
            // reject types that are clearly not vector-shaped (Text,
            // Integer, Boolean, Struct, ...).
            ArgumentType::Vector => matches!(
                actual,
                ir::ValueType::Vector(_, _)
                    | ir::ValueType::Bytes
                    | ir::ValueType::Any
                    | ir::ValueType::Custom { .. }
            ),
        }
    }
}

pub struct FunctionSignature {
    pub arguments: Vec<ArgumentType>,
    /// When true, `arguments` describes only the required minimum-arity
    /// prefix; a call may supply more arguments than `arguments.len()`,
    /// and anything past the prefix is not type-checked.
    pub variadic: bool,
    pub return_type: fn(&[ir::TypedExpression]) -> ir::ValueType,
}

/// Alias for the return-type callback, reused below to avoid a
/// clippy::type_complexity hit on the tuple type in `lookup`.
type ReturnTypeFn = fn(&[ir::TypedExpression]) -> ir::ValueType;

/// Validates a bound call's argument count and types against `signature`.
/// Returns a static description of the mismatch on failure, suitable for
/// `BindError::Unsupported`'s `&'static str` `feature` field.
pub fn validate_arguments(
    signature: &FunctionSignature,
    arguments: &[ir::TypedExpression],
) -> Result<(), &'static str> {
    if signature.variadic {
        if arguments.len() < signature.arguments.len() {
            return Err("function call with too few arguments");
        }
    } else if arguments.len() != signature.arguments.len() {
        return Err("function call with the wrong number of arguments");
    }
    for (expected, actual) in signature.arguments.iter().zip(arguments.iter()) {
        if !expected.matches(&actual.value_type) {
            return Err("function call with a mismatched argument type");
        }
    }
    Ok(())
}

/// Derives a vector-constructor call's dimension count by parsing its
/// single bound argument as vector text, when that argument is a Cypher
/// text literal (`vector32('[1.0, 2.0, 3.0]')`). Returns `None` for
/// anything else (a parameter, a column reference, a non-literal
/// expression, or text that fails to parse as a vector) — dims are a
/// best-effort static hint, never a hard requirement.
fn vector_literal_dims(argument: &ir::TypedExpression, vector_type: CoreVectorType) -> Option<u32> {
    let ir::Expression::Literal(ir::Literal::Text(text)) = &argument.expression else {
        return None;
    };
    vector_from_text(vector_type, text)
        .ok()
        .map(|vector| vector.dims as u32)
}

/// The one place a vector kind is paired with its core encoding, so a new
/// kind cannot pick up a mismatched dims parser.
const fn core_vector_type(kind: ir::VectorKind) -> CoreVectorType {
    match kind {
        ir::VectorKind::Float32Dense => CoreVectorType::Float32Dense,
        ir::VectorKind::Float64Dense => CoreVectorType::Float64Dense,
        ir::VectorKind::Float32Sparse => CoreVectorType::Float32Sparse,
        ir::VectorKind::Float1Bit => CoreVectorType::Float1Bit,
        ir::VectorKind::Float8 => CoreVectorType::Float8,
    }
}

fn vector_return_type(kind: ir::VectorKind, args: &[ir::TypedExpression]) -> ir::ValueType {
    ir::ValueType::Vector(
        kind,
        args.first()
            .and_then(|a| vector_literal_dims(a, core_vector_type(kind))),
    )
}

// `ReturnTypeFn` is a capture-free fn pointer, so each arm still names its
// kind once to close over it statically; all shared logic lives in
// `vector_return_type`.
fn vector_return(kind: ir::VectorKind) -> ReturnTypeFn {
    match kind {
        ir::VectorKind::Float32Dense => {
            |args| vector_return_type(ir::VectorKind::Float32Dense, args)
        }
        ir::VectorKind::Float64Dense => {
            |args| vector_return_type(ir::VectorKind::Float64Dense, args)
        }
        ir::VectorKind::Float32Sparse => {
            |args| vector_return_type(ir::VectorKind::Float32Sparse, args)
        }
        ir::VectorKind::Float1Bit => |args| vector_return_type(ir::VectorKind::Float1Bit, args),
        ir::VectorKind::Float8 => |args| vector_return_type(ir::VectorKind::Float8, args),
    }
}

fn return_real(_: &[ir::TypedExpression]) -> ir::ValueType {
    ir::ValueType::Real
}

fn return_bytes(_: &[ir::TypedExpression]) -> ir::ValueType {
    ir::ValueType::Bytes
}

fn return_boolean(_: &[ir::TypedExpression]) -> ir::ValueType {
    ir::ValueType::Boolean
}

fn return_text(_: &[ir::TypedExpression]) -> ir::ValueType {
    ir::ValueType::Text
}

/// Static typed function registry. Functions not listed here keep today's
/// untyped `Any` pass-through in `Binder::bind_expression` — this table is
/// additive, not a closed world.
pub fn lookup(name: &str) -> Option<FunctionSignature> {
    let (arguments, variadic, return_type): (Vec<ArgumentType>, bool, ReturnTypeFn) = match name {
        // Fixed-arity-1: one pre-encoded TEXT/BLOB literal, not
        // variadic-over-floats (core/vector/mod.rs enforces args.len() == 1
        // for all five constructors).
        "vector32" => (
            vec![ArgumentType::Any],
            false,
            vector_return(ir::VectorKind::Float32Dense),
        ),
        "vector32_sparse" => (
            vec![ArgumentType::Any],
            false,
            vector_return(ir::VectorKind::Float32Sparse),
        ),
        "vector64" => (
            vec![ArgumentType::Any],
            false,
            vector_return(ir::VectorKind::Float64Dense),
        ),
        "vector8" => (
            vec![ArgumentType::Any],
            false,
            vector_return(ir::VectorKind::Float8),
        ),
        "vector1bit" => (
            vec![ArgumentType::Any],
            false,
            vector_return(ir::VectorKind::Float1Bit),
        ),
        // Fixed-arity-1, Blob-only, returns the vector's text
        // representation — the inverse of construction, not an element
        // accessor (core/vector/mod.rs's vector_extract).
        "vector_extract" => (vec![ArgumentType::Vector], false, return_text),
        "vector_concat" => (
            vec![ArgumentType::Vector, ArgumentType::Vector],
            false,
            return_bytes,
        ),
        "vector_slice" => (
            vec![
                ArgumentType::Vector,
                ArgumentType::Exact(ir::ValueType::Integer),
                ArgumentType::Exact(ir::ValueType::Integer),
            ],
            false,
            return_bytes,
        ),
        "vector_distance_cos"
        | "vector_distance_l2"
        | "vector_distance_jaccard"
        | "vector_distance_dot" => (
            vec![ArgumentType::Vector, ArgumentType::Vector],
            false,
            return_real,
        ),
        // Variadic, no enforced minimum (translate_variadic_insn!).
        "struct_pack" => (vec![], true, return_bytes),
        "union_value" => (
            vec![ArgumentType::Exact(ir::ValueType::Text), ArgumentType::Any],
            false,
            return_bytes,
        ),
        "union_tag" => (vec![ArgumentType::Any], false, return_text),
        // Variadic, min 2: fts_match(col1, ..., query).
        "fts_match" => (
            vec![
                ArgumentType::Exact(ir::ValueType::Text),
                ArgumentType::Exact(ir::ValueType::Text),
            ],
            true,
            return_boolean,
        ),
        // Variadic, no enforced minimum.
        "fts_score" => (vec![], true, return_real),
        // Variadic, min 4: fts_highlight(col1, ..., before_tag, after_tag, query).
        "fts_highlight" => (
            vec![
                ArgumentType::Any,
                ArgumentType::Any,
                ArgumentType::Any,
                ArgumentType::Any,
            ],
            true,
            return_text,
        ),
        _ => return None,
    };
    Some(FunctionSignature {
        arguments,
        variadic,
        return_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_literal(text: &str) -> ir::TypedExpression {
        ir::TypedExpression {
            expression: ir::Expression::Literal(ir::Literal::Text(text.to_owned())),
            value_type: ir::ValueType::Text,
            nullability: ir::Nullability::NonNull,
        }
    }

    fn any_typed(value_type: ir::ValueType) -> ir::TypedExpression {
        ir::TypedExpression {
            expression: ir::Expression::Literal(ir::Literal::Null),
            value_type,
            nullability: ir::Nullability::Nullable,
        }
    }

    #[test]
    fn vector32_returns_dims_from_literal_text() {
        let signature = lookup("vector32").expect("vector32 registered");
        let arguments = vec![text_literal("[1.0, 2.0, 3.0]")];
        assert_eq!(
            (signature.return_type)(&arguments),
            ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(3))
        );
    }

    #[test]
    fn vector32_returns_none_dims_for_non_literal_argument() {
        let signature = lookup("vector32").expect("vector32 registered");
        let arguments = vec![any_typed(ir::ValueType::Text)];
        assert_eq!(
            (signature.return_type)(&arguments),
            ir::ValueType::Vector(ir::VectorKind::Float32Dense, None)
        );
    }

    #[test]
    fn vector_extract_returns_text() {
        let signature = lookup("vector_extract").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Text);
        assert_eq!(signature.arguments.len(), 1);
        assert!(!signature.variadic);
    }

    #[test]
    fn vector_distance_cos_returns_real() {
        let signature = lookup("vector_distance_cos").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Real);
    }

    #[test]
    fn struct_pack_returns_bytes() {
        let signature = lookup("struct_pack").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Bytes);
        assert!(signature.variadic);
    }

    #[test]
    fn fts_match_returns_boolean() {
        let signature = lookup("fts_match").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Boolean);
        assert!(signature.variadic);
        assert_eq!(signature.arguments.len(), 2);
    }

    #[test]
    fn unknown_function_returns_none() {
        assert!(lookup("not_a_real_function").is_none());
    }

    #[test]
    fn validate_arguments_rejects_wrong_fixed_arity() {
        let signature = lookup("vector32").expect("registered");
        let arguments = vec![
            any_typed(ir::ValueType::Real),
            any_typed(ir::ValueType::Real),
        ];
        assert_eq!(
            validate_arguments(&signature, &arguments),
            Err("function call with the wrong number of arguments")
        );
    }

    #[test]
    fn validate_arguments_rejects_too_few_variadic_arguments() {
        let signature = lookup("fts_match").expect("registered");
        let arguments = vec![any_typed(ir::ValueType::Text)];
        assert_eq!(
            validate_arguments(&signature, &arguments),
            Err("function call with too few arguments")
        );
    }

    #[test]
    fn validate_arguments_accepts_extra_variadic_arguments() {
        let signature = lookup("fts_match").expect("registered");
        let arguments = vec![
            any_typed(ir::ValueType::Text),
            any_typed(ir::ValueType::Text),
            any_typed(ir::ValueType::Text),
        ];
        assert_eq!(validate_arguments(&signature, &arguments), Ok(()));
    }

    #[test]
    fn validate_arguments_rejects_mismatched_type() {
        let signature = lookup("vector_distance_cos").expect("registered");
        let arguments = vec![
            any_typed(ir::ValueType::Text),
            any_typed(ir::ValueType::Text),
        ];
        assert_eq!(
            validate_arguments(&signature, &arguments),
            Err("function call with a mismatched argument type")
        );
    }

    #[test]
    fn validate_arguments_accepts_any_typed_argument() {
        let signature = lookup("vector_distance_cos").expect("registered");
        let arguments = vec![any_typed(ir::ValueType::Any), any_typed(ir::ValueType::Any)];
        assert_eq!(validate_arguments(&signature, &arguments), Ok(()));
    }
}
