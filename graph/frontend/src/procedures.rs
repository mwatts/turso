use turso_graph_ir as ir;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProcedureAccess {
    ReadOnly,
    Mutating,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProcedureArgument {
    pub name: &'static str,
    pub value_type: ir::ValueType,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProcedureYield {
    pub name: &'static str,
    pub value_type: ir::ValueType,
    pub nullability: ir::Nullability,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProcedureDescriptor {
    pub name: &'static str,
    pub identity: ir::ProcedureIdentity,
    pub arguments: &'static [ProcedureArgument],
    pub yields: &'static [ProcedureYield],
    pub access: ProcedureAccess,
}

impl ProcedureDescriptor {
    pub fn accepts_arity(&self, actual: usize) -> bool {
        let required = self
            .arguments
            .iter()
            .filter(|argument| argument.required)
            .count();
        (required..=self.arguments.len()).contains(&actual)
    }

    pub fn yield_index(&self, name: &str) -> Option<usize> {
        self.yields
            .iter()
            .position(|column| column.name.eq_ignore_ascii_case(name))
    }
}

const NO_ARGUMENTS: &[ProcedureArgument] = &[];
const LABEL_YIELD: &[ProcedureYield] = &[ProcedureYield {
    name: "label",
    value_type: ir::ValueType::Text,
    nullability: ir::Nullability::NonNull,
}];
const RELATIONSHIP_TYPE_YIELD: &[ProcedureYield] = &[ProcedureYield {
    name: "relationshipType",
    value_type: ir::ValueType::Text,
    nullability: ir::Nullability::NonNull,
}];
const PROPERTY_KEY_YIELD: &[ProcedureYield] = &[ProcedureYield {
    name: "propertyKey",
    value_type: ir::ValueType::Text,
    nullability: ir::Nullability::NonNull,
}];

const PROCEDURES: &[ProcedureDescriptor] = &[
    ProcedureDescriptor {
        name: "db.labels",
        identity: ir::ProcedureIdentity::DbLabels,
        arguments: NO_ARGUMENTS,
        yields: LABEL_YIELD,
        access: ProcedureAccess::ReadOnly,
    },
    ProcedureDescriptor {
        name: "db.relationshipTypes",
        identity: ir::ProcedureIdentity::DbRelationshipTypes,
        arguments: NO_ARGUMENTS,
        yields: RELATIONSHIP_TYPE_YIELD,
        access: ProcedureAccess::ReadOnly,
    },
    ProcedureDescriptor {
        name: "db.propertyKeys",
        identity: ir::ProcedureIdentity::DbPropertyKeys,
        arguments: NO_ARGUMENTS,
        yields: PROPERTY_KEY_YIELD,
        access: ProcedureAccess::ReadOnly,
    },
];

pub(crate) fn lookup(name: &str) -> Option<&'static ProcedureDescriptor> {
    PROCEDURES
        .iter()
        .find(|procedure| procedure.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_resolves_canonical_seed_signatures_case_insensitively() {
        let cases = [
            (
                "DB.LABELS",
                ir::ProcedureIdentity::DbLabels,
                "db.labels",
                "label",
            ),
            (
                "db.relationshiptypes",
                ir::ProcedureIdentity::DbRelationshipTypes,
                "db.relationshipTypes",
                "relationshipType",
            ),
            (
                "Db.PropertyKeys",
                ir::ProcedureIdentity::DbPropertyKeys,
                "db.propertyKeys",
                "propertyKey",
            ),
        ];
        for (lookup_name, identity, canonical_name, yield_name) in cases {
            let descriptor = lookup(lookup_name).expect("seed procedure");
            assert_eq!(descriptor.identity, identity);
            assert_eq!(descriptor.name, canonical_name);
            assert_eq!(descriptor.access, ProcedureAccess::ReadOnly);
            assert!(descriptor.arguments.is_empty());
            assert_eq!(descriptor.yields.len(), 1);
            assert_eq!(descriptor.yields[0].name, yield_name);
            assert_eq!(descriptor.yields[0].value_type, ir::ValueType::Text);
            assert_eq!(descriptor.yields[0].nullability, ir::Nullability::NonNull);
            assert!(descriptor.accepts_arity(0));
            assert!(!descriptor.accepts_arity(1));
            assert_eq!(descriptor.yield_index(yield_name), Some(0));
        }
        assert!(lookup("db.unknown").is_none());
    }

    #[test]
    fn descriptor_arity_supports_required_and_optional_arguments() {
        const ARGUMENTS: &[ProcedureArgument] = &[
            ProcedureArgument {
                name: "required",
                value_type: ir::ValueType::Text,
                required: true,
            },
            ProcedureArgument {
                name: "optional",
                value_type: ir::ValueType::Integer,
                required: false,
            },
        ];
        let descriptor = ProcedureDescriptor {
            name: "test.signature",
            identity: ir::ProcedureIdentity::DbLabels,
            arguments: ARGUMENTS,
            yields: LABEL_YIELD,
            access: ProcedureAccess::Mutating,
        };
        assert!(!descriptor.accepts_arity(0));
        assert!(descriptor.accepts_arity(1));
        assert!(descriptor.accepts_arity(2));
        assert!(!descriptor.accepts_arity(3));
        assert_eq!(descriptor.arguments[0].name, "required");
        assert_eq!(descriptor.arguments[0].value_type, ir::ValueType::Text);
        assert_eq!(descriptor.access, ProcedureAccess::Mutating);
    }
}
