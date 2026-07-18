use turso_graph_ir as ir;

#[derive(Clone, Debug, PartialEq)]
pub enum ArgumentType {
    Any,
    Exact(ir::ValueType),
    Vector,
}

pub struct FunctionSignature {
    // Not yet consumed by `Binder::bind_expression` (only `return_type` is
    // wired in); reserved for future call-site argument-type validation.
    #[allow(dead_code)]
    pub arguments: Vec<ArgumentType>,
    pub return_type: fn(&[ir::ValueType]) -> ir::ValueType,
}

/// Alias for the return-type callback, reused below to avoid a
/// clippy::type_complexity hit on the tuple type in `lookup`.
type ReturnTypeFn = fn(&[ir::ValueType]) -> ir::ValueType;

fn vector_dims_return(kind: ir::VectorKind) -> fn(&[ir::ValueType]) -> ir::ValueType {
    match kind {
        ir::VectorKind::Float32Dense => {
            |args| ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(args.len() as u32))
        }
        ir::VectorKind::Float64Dense => {
            |args| ir::ValueType::Vector(ir::VectorKind::Float64Dense, Some(args.len() as u32))
        }
        ir::VectorKind::Float32Sparse => {
            |args| ir::ValueType::Vector(ir::VectorKind::Float32Sparse, Some(args.len() as u32))
        }
        ir::VectorKind::Float1Bit => {
            |args| ir::ValueType::Vector(ir::VectorKind::Float1Bit, Some(args.len() as u32))
        }
        ir::VectorKind::Float8 => {
            |args| ir::ValueType::Vector(ir::VectorKind::Float8, Some(args.len() as u32))
        }
    }
}

fn return_real(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Real
}

fn return_bytes(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Bytes
}

fn return_boolean(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Boolean
}

fn return_text(_: &[ir::ValueType]) -> ir::ValueType {
    ir::ValueType::Text
}

/// Static typed function registry. Functions not listed here keep today's
/// untyped `Any` pass-through in `Binder::bind_expression` — this table is
/// additive, not a closed world.
pub fn lookup(name: &str) -> Option<FunctionSignature> {
    let (arguments, return_type): (Vec<ArgumentType>, ReturnTypeFn) = match name {
        "vector32" => (
            vec![ArgumentType::Any],
            vector_dims_return(ir::VectorKind::Float32Dense),
        ),
        "vector32_sparse" => (
            vec![ArgumentType::Any],
            vector_dims_return(ir::VectorKind::Float32Sparse),
        ),
        "vector64" => (
            vec![ArgumentType::Any],
            vector_dims_return(ir::VectorKind::Float64Dense),
        ),
        "vector8" => (
            vec![ArgumentType::Any],
            vector_dims_return(ir::VectorKind::Float8),
        ),
        "vector1bit" => (
            vec![ArgumentType::Any],
            vector_dims_return(ir::VectorKind::Float1Bit),
        ),
        "vector_extract" => (
            vec![
                ArgumentType::Vector,
                ArgumentType::Exact(ir::ValueType::Integer),
            ],
            return_real,
        ),
        "vector_concat" => (
            vec![ArgumentType::Vector, ArgumentType::Vector],
            return_bytes,
        ),
        "vector_slice" => (
            vec![
                ArgumentType::Vector,
                ArgumentType::Exact(ir::ValueType::Integer),
                ArgumentType::Exact(ir::ValueType::Integer),
            ],
            return_bytes,
        ),
        "vector_distance_cos"
        | "vector_distance_l2"
        | "vector_distance_jaccard"
        | "vector_distance_dot" => (
            vec![ArgumentType::Vector, ArgumentType::Vector],
            return_real,
        ),
        "struct_pack" => (vec![ArgumentType::Any], return_bytes),
        "union_value" => (
            vec![ArgumentType::Exact(ir::ValueType::Text), ArgumentType::Any],
            return_bytes,
        ),
        "union_tag" => (vec![ArgumentType::Any], return_text),
        "fts_match" => (
            vec![
                ArgumentType::Exact(ir::ValueType::Text),
                ArgumentType::Exact(ir::ValueType::Text),
            ],
            return_boolean,
        ),
        "fts_score" => (vec![], return_real),
        "fts_highlight" => (vec![ArgumentType::Any], return_text),
        _ => return None,
    };
    Some(FunctionSignature {
        arguments,
        return_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector32_returns_dims_from_argument_count() {
        let signature = lookup("vector32").expect("vector32 registered");
        let arguments = vec![
            ir::ValueType::Real,
            ir::ValueType::Real,
            ir::ValueType::Real,
        ];
        assert_eq!(
            (signature.return_type)(&arguments),
            ir::ValueType::Vector(ir::VectorKind::Float32Dense, Some(3))
        );
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
    }

    #[test]
    fn fts_match_returns_boolean() {
        let signature = lookup("fts_match").expect("registered");
        assert_eq!((signature.return_type)(&[]), ir::ValueType::Boolean);
    }

    #[test]
    fn unknown_function_returns_none() {
        assert!(lookup("not_a_real_function").is_none());
    }
}
