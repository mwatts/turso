use std::fmt;
use std::num::{NonZeroU32, NonZeroU64};

use crate::InvalidId;

macro_rules! define_u64_id {
    ($name:ident, $kind:literal) => {
        #[doc = concat!("Stable non-zero ", $kind, " identity.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU64);

        impl $name {
            pub fn new(value: u64) -> Result<Self, InvalidId> {
                NonZeroU64::new(value)
                    .map(Self)
                    .ok_or(InvalidId { kind: $kind, value })
            }

            pub const fn get(self) -> u64 {
                self.0.get()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }
    };
}

macro_rules! define_u32_id {
    ($name:ident, $kind:literal) => {
        #[doc = concat!("Stable non-zero ", $kind, " identity.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            pub fn new(value: u32) -> Result<Self, InvalidId> {
                NonZeroU32::new(value).map(Self).ok_or(InvalidId {
                    kind: $kind,
                    value: u64::from(value),
                })
            }

            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.get().fmt(f)
            }
        }
    };
}

define_u64_id!(GraphId, "graph");
define_u64_id!(SourceTableId, "source table");
define_u64_id!(NodeId, "node");
define_u64_id!(RelationshipId, "relationship");
define_u32_id!(LabelId, "label");
define_u32_id!(RelationshipTypeId, "relationship type");
define_u32_id!(PropertyId, "property");
define_u32_id!(BindingId, "binding");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_public_identities_reject_zero() {
        assert!(GraphId::new(0).is_err());
        assert!(SourceTableId::new(0).is_err());
        assert!(NodeId::new(0).is_err());
        assert!(RelationshipId::new(0).is_err());
        assert!(LabelId::new(0).is_err());
        assert!(RelationshipTypeId::new(0).is_err());
        assert!(PropertyId::new(0).is_err());
        assert!(BindingId::new(0).is_err());
    }

    #[test]
    fn identity_round_trips_without_cross_type_conversion() {
        let graph = GraphId::new(7).unwrap();
        let node = NodeId::new(7).unwrap();
        assert_eq!(graph.get(), 7);
        assert_eq!(node.get(), 7);
        assert_eq!(graph.to_string(), "7");
    }
}
