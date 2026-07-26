use crate::{
    Binding, BindingId, GraphId, LabelId, NullOrder, PlanError, RelationshipTypeId, ResultShape,
    RoleId, Scope, SortDirection, SourceTableId, TypedExpression,
};

/// A validated bound graph plan node with explicit visible scope and output.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    kind: PlanKind,
    scope: Scope,
    result_shape: ResultShape,
}

impl Plan {
    pub fn new(kind: PlanKind, scope: Scope, result_shape: ResultShape) -> Result<Self, PlanError> {
        result_shape.validate(&scope)?;
        Ok(Self {
            kind,
            scope,
            result_shape,
        })
    }

    pub fn kind(&self) -> &PlanKind {
        &self.kind
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn result_shape(&self) -> &ResultShape {
        &self.result_shape
    }
}

/// First read-only operator set shared by Cypher and compatibility adapters.
#[derive(Clone, Debug, PartialEq)]
pub enum PlanKind {
    Unit(Unit),
    NodeScan(NodeScan),
    RoleExpand(RoleExpand),
    GraphExpand(GraphExpand),
    Filter(Filter),
    Project(Project),
    Aggregate(Aggregate),
    Distinct(Distinct),
    Sort(Sort),
    Skip(Skip),
    Limit(Limit),
    LeftApply(LeftApply),
    Unwind(Unwind),
    ProcedureCall(ProcedureCall),
    Union(Union),
    Join(Join),
}

/// Cartesian product of two independent inputs; a later Filter applies any
/// cross-input predicates.
#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    pub left: Box<Plan>,
    pub right: Box<Plan>,
}

/// A single row with no bindings, used by source clauses such as `UNWIND`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Unit;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeScan {
    pub graph: GraphId,
    pub source: SourceTableId,
    pub binding: BindingId,
    pub labels: Vec<LabelId>,
}

/// A single relationship hop from an already-bound node.
#[derive(Clone, Debug, PartialEq)]
pub struct RoleExpand {
    pub input: Box<Plan>,
    pub from_node_source: SourceTableId,
    pub relationship_source: SourceTableId,
    pub target_node_source: SourceTableId,
    pub from: BindingId,
    pub relationship: Binding,
    pub to: Binding,
    /// Role the traversal leaves the source binding through.
    pub from_role: RoleId,
    /// Role the traversal enters the target binding through.
    pub to_role: RoleId,
    /// Also match the reversed pair. This is what an undirected pattern
    /// means when both endpoints share a node source; a plain ordered pair
    /// cannot say it.
    pub symmetric: bool,
    pub relationship_types: Vec<RelationshipTypeId>,
    /// When the target closes a cycle onto an already-bound node, the
    /// binding whose identity the target must equal. Lowering folds the
    /// equality into the relationship join (making composite endpoint
    /// indexes usable) instead of filtering after an extra node join.
    pub bound_target: Option<BindingId>,
}

impl RoleExpand {
    pub fn role_pair(&self) -> (RoleId, RoleId) {
        (self.from_role, self.to_role)
    }
}

/// A bounded variable-length expansion from an already-bound node.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphExpand {
    pub input: Box<Plan>,
    pub graph: GraphId,
    pub from_node_source: SourceTableId,
    pub relationship_source: SourceTableId,
    pub target_node_source: SourceTableId,
    pub from: BindingId,
    pub relationship: Binding,
    pub to: Binding,
    /// Role the traversal leaves the source binding through.
    pub from_role: RoleId,
    /// Role the traversal enters the target binding through.
    pub to_role: RoleId,
    /// Also match the reversed pair. This is what an undirected pattern
    /// means when both endpoints share a node source; a plain ordered pair
    /// cannot say it.
    pub symmetric: bool,
    pub relationship_types: Vec<RelationshipTypeId>,
    pub min_hops: u32,
    pub max_hops: u32,
    /// True when the source range had no upper bound (`[*]`, `[*min..]`):
    /// `max_hops` is then a resource cap, and execution must error instead
    /// of silently truncating when a longer admissible path exists.
    pub unbounded: bool,
    pub uniqueness: PathUniqueness,
    /// When set, the expansion also materializes each traversed path as a
    /// {nodes, relationships} value bound to this output.
    pub path_output: Option<Binding>,
    /// When set, the expansion also materializes the traversed relationship
    /// identities as a list value bound to this output (a named
    /// variable-length relationship).
    pub relationship_list_output: Option<Binding>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathUniqueness {
    Walk,
    Trail,
    Path,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub input: Box<Plan>,
    pub predicate: TypedExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
    pub output: Binding,
    pub expression: TypedExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub input: Box<Plan>,
    pub projections: Vec<Projection>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AggregateFunction {
    Count,
    Sum,
    Average,
    Minimum,
    Maximum,
    Collect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Grouping {
    pub output: Binding,
    pub expression: TypedExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Aggregation {
    pub output: Binding,
    pub function: AggregateFunction,
    pub expression: Option<TypedExpression>,
    pub distinct: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Aggregate {
    pub input: Box<Plan>,
    pub groupings: Vec<Grouping>,
    pub aggregations: Vec<Aggregation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Distinct {
    pub input: Box<Plan>,
    pub keys: Vec<TypedExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SortKey {
    pub expression: TypedExpression,
    pub direction: SortDirection,
    pub null_order: NullOrder,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sort {
    pub input: Box<Plan>,
    pub keys: Vec<SortKey>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Skip {
    pub input: Box<Plan>,
    pub count: TypedExpression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Limit {
    pub input: Box<Plan>,
    pub count: TypedExpression,
}

/// Correlated optional match. Bindings introduced by the right plan are
/// nullable in this node's output scope.
#[derive(Clone, Debug, PartialEq)]
pub struct LeftApply {
    pub left: Box<Plan>,
    pub right: Box<Plan>,
    pub correlated: Vec<BindingId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Unwind {
    pub input: Box<Plan>,
    pub list: TypedExpression,
    pub output: Binding,
}

/// Closed read-procedure implementations shared by graph frontends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcedureIdentity {
    DbLabels,
    DbRelationshipTypes,
    DbPropertyKeys,
}

/// One selected procedure column and its bound output.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureOutput {
    pub column: usize,
    pub output: Binding,
}

/// A typed read-only procedure composed with its relational input.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureCall {
    pub input: Box<Plan>,
    pub procedure: ProcedureIdentity,
    pub arguments: Vec<TypedExpression>,
    pub outputs: Vec<ProcedureOutput>,
}

/// Shape-compatible UNION inputs.
#[derive(Clone, Debug, PartialEq)]
pub struct Union {
    inputs: Vec<Plan>,
    all: bool,
}

impl Union {
    pub fn new(inputs: Vec<Plan>, all: bool) -> Result<Self, PlanError> {
        if inputs.len() < 2 {
            return Err(PlanError::UnionNeedsMultipleInputs);
        }
        let expected = inputs[0].result_shape().len();
        for (input, plan) in inputs.iter().enumerate().skip(1) {
            let actual = plan.result_shape().len();
            if actual != expected {
                return Err(PlanError::UnionShapeMismatch {
                    input,
                    expected,
                    actual,
                });
            }
        }
        Ok(Self { inputs, all })
    }

    pub fn inputs(&self) -> &[Plan] {
        &self.inputs
    }

    pub const fn is_all(&self) -> bool {
        self.all
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Nullability, ResultColumn, ValueType};

    fn scan(binding_id: u32, column_count: usize) -> Plan {
        let id = BindingId::new(binding_id).unwrap();
        let binding = Binding::new(
            id,
            format!("n{binding_id}"),
            ValueType::Node,
            Nullability::NonNull,
        )
        .unwrap();
        let scope = Scope::new(vec![binding]).unwrap();
        let columns = (0..column_count)
            .map(|index| ResultColumn::new(id, format!("column_{index}")).unwrap())
            .collect();
        let shape = ResultShape::new(columns, &scope).unwrap();
        Plan::new(
            PlanKind::NodeScan(NodeScan {
                graph: GraphId::new(1).unwrap(),
                source: SourceTableId::new(1).unwrap(),
                binding: id,
                labels: vec![],
            }),
            scope,
            shape,
        )
        .unwrap()
    }

    #[test]
    fn plan_revalidates_result_shape_against_its_scope() {
        let source = scan(1, 1);
        let other_scope = Scope::new(vec![Binding::new(
            BindingId::new(2).unwrap(),
            "m",
            ValueType::Node,
            Nullability::NonNull,
        )
        .unwrap()])
        .unwrap();

        assert_eq!(
            Plan::new(
                source.kind().clone(),
                other_scope,
                source.result_shape().clone()
            ),
            Err(PlanError::UnknownResultBinding(BindingId::new(1).unwrap()))
        );
    }

    #[test]
    fn union_requires_multiple_shape_compatible_inputs() {
        assert_eq!(
            Union::new(vec![scan(1, 1)], false),
            Err(PlanError::UnionNeedsMultipleInputs)
        );
        assert_eq!(
            Union::new(vec![scan(1, 1), scan(2, 2)], false),
            Err(PlanError::UnionShapeMismatch {
                input: 1,
                expected: 1,
                actual: 2,
            })
        );
        assert!(Union::new(vec![scan(1, 1), scan(2, 1)], true).is_ok());
    }

    fn sample_role_expand() -> RoleExpand {
        let from_binding = BindingId::new(1).unwrap();
        let relationship_binding = BindingId::new(2).unwrap();
        let to_binding = BindingId::new(3).unwrap();
        RoleExpand {
            input: Box::new(scan(1, 1)),
            from_node_source: SourceTableId::new(1).unwrap(),
            relationship_source: SourceTableId::new(2).unwrap(),
            target_node_source: SourceTableId::new(3).unwrap(),
            from: from_binding,
            relationship: Binding::new(
                relationship_binding,
                "r",
                ValueType::Relationship,
                Nullability::NonNull,
            )
            .unwrap(),
            to: Binding::new(to_binding, "b", ValueType::Node, Nullability::NonNull).unwrap(),
            from_role: RoleId::new(1).unwrap(),
            to_role: RoleId::new(2).unwrap(),
            symmetric: false,
            relationship_types: vec![],
            bound_target: None,
        }
    }

    #[test]
    fn a_role_expand_names_its_roles_and_no_direction() {
        // Direction is a parser spelling, not a plan concept. A plan that still
        // carried it would give two sources of truth for which way a traversal
        // runs.
        let expand = sample_role_expand();
        assert_eq!(
            expand.role_pair(),
            (RoleId::new(1).unwrap(), RoleId::new(2).unwrap())
        );
    }

    #[test]
    fn procedure_call_carries_closed_identity_arguments_and_selected_outputs() {
        let input = scan(1, 1);
        let output = Binding::new(
            BindingId::new(2).unwrap(),
            "propertyKey",
            ValueType::Text,
            Nullability::NonNull,
        )
        .unwrap();
        let call = ProcedureCall {
            input: Box::new(input.clone()),
            procedure: ProcedureIdentity::DbPropertyKeys,
            arguments: Vec::new(),
            outputs: vec![ProcedureOutput {
                column: 0,
                output: output.clone(),
            }],
        };
        assert_eq!(call.input.as_ref(), &input);
        assert_eq!(call.procedure, ProcedureIdentity::DbPropertyKeys);
        assert!(call.arguments.is_empty());
        assert_eq!(call.outputs[0].output, output);
    }
}
