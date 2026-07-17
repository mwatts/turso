use std::collections::HashSet;

use crate::{BindingId, PlanError, ValueType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Nullability {
    NonNull,
    Nullable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Outgoing,
    Incoming,
    Both,
}

/// A named value visible at a point in a bound graph plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    id: BindingId,
    name: String,
    value_type: ValueType,
    nullability: Nullability,
}

impl Binding {
    pub fn new(
        id: BindingId,
        name: impl Into<String>,
        value_type: ValueType,
        nullability: Nullability,
    ) -> Result<Self, PlanError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(PlanError::EmptyBindingName);
        }
        Ok(Self {
            id,
            name,
            value_type,
            nullability,
        })
    }

    pub const fn id(&self) -> BindingId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub const fn nullability(&self) -> Nullability {
        self.nullability
    }
}

/// Ordered bindings visible after an operator.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Scope {
    bindings: Vec<Binding>,
}

impl Scope {
    pub fn new(bindings: Vec<Binding>) -> Result<Self, PlanError> {
        let mut ids = HashSet::with_capacity(bindings.len());
        let mut names = HashSet::with_capacity(bindings.len());
        for binding in &bindings {
            if !ids.insert(binding.id()) {
                return Err(PlanError::DuplicateBindingId(binding.id()));
            }
            if !names.insert(binding.name()) {
                return Err(PlanError::DuplicateBindingName(binding.name().to_string()));
            }
        }
        Ok(Self { bindings })
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Binding> {
        self.bindings.iter()
    }

    pub fn get(&self, id: BindingId) -> Option<&Binding> {
        self.bindings.iter().find(|binding| binding.id() == id)
    }

    pub fn resolve(&self, name: &str) -> Option<&Binding> {
        self.bindings.iter().find(|binding| binding.name() == name)
    }
}

/// A projected output column tied to a bound value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResultColumn {
    binding: BindingId,
    name: String,
}

impl ResultColumn {
    pub fn new(binding: BindingId, name: impl Into<String>) -> Result<Self, PlanError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(PlanError::EmptyResultColumnName);
        }
        Ok(Self { binding, name })
    }

    pub const fn binding(&self) -> BindingId {
        self.binding
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Ordered shape returned by a plan node.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResultShape {
    columns: Vec<ResultColumn>,
}

impl ResultShape {
    pub fn new(columns: Vec<ResultColumn>, scope: &Scope) -> Result<Self, PlanError> {
        for column in &columns {
            if scope.get(column.binding()).is_none() {
                return Err(PlanError::UnknownResultBinding(column.binding()));
            }
        }
        Ok(Self { columns })
    }

    pub(crate) fn validate(&self, scope: &Scope) -> Result<(), PlanError> {
        for column in &self.columns {
            if scope.get(column.binding()).is_none() {
                return Err(PlanError::UnknownResultBinding(column.binding()));
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &ResultColumn> {
        self.columns.iter()
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(id: u32, name: &str) -> Binding {
        Binding::new(
            BindingId::new(id).unwrap(),
            name,
            ValueType::Node,
            Nullability::NonNull,
        )
        .unwrap()
    }

    #[test]
    fn scope_rejects_duplicate_names_and_ids() {
        assert_eq!(
            Scope::new(vec![binding(1, "n"), binding(2, "n")]),
            Err(PlanError::DuplicateBindingName("n".to_string()))
        );
        assert_eq!(
            Scope::new(vec![binding(1, "n"), binding(1, "m")]),
            Err(PlanError::DuplicateBindingId(BindingId::new(1).unwrap()))
        );
    }

    #[test]
    fn result_shape_must_reference_visible_bindings() {
        let scope = Scope::new(vec![binding(1, "n")]).unwrap();
        let column = ResultColumn::new(BindingId::new(2).unwrap(), "missing").unwrap();
        assert_eq!(
            ResultShape::new(vec![column], &scope),
            Err(PlanError::UnknownResultBinding(BindingId::new(2).unwrap()))
        );
    }
}
