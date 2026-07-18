use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TestId(String);

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TestIdError {
    #[error("test identity must contain at least one namespace separator")]
    MissingNamespace,
    #[error("test identity must start and end with an ASCII lowercase letter or digit")]
    InvalidBoundary,
    #[error("test identity contains invalid character `{0}`")]
    InvalidCharacter(char),
}

impl TestId {
    pub fn parse(value: impl Into<String>) -> Result<Self, TestIdError> {
        let value = value.into();
        if !value.contains('.') {
            return Err(TestIdError::MissingNamespace);
        }
        if !value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
            || !value
                .bytes()
                .next_back()
                .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            return Err(TestIdError::InvalidBoundary);
        }
        if let Some(character) = value.chars().find(|character| {
            !character.is_ascii_lowercase()
                && !character.is_ascii_digit()
                && !matches!(character, '.' | '-')
        }) {
            return Err(TestIdError::InvalidCharacter(character));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for TestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identities_are_stable_machine_readable_names() {
        assert!(TestId::parse("tck.with.with1.scenario-1").is_ok());
        assert!(TestId::parse("scenario-1").is_err());
        assert!(TestId::parse("TCK.with.1").is_err());
        assert!(TestId::parse("tck.with.1_").is_err());
    }
}
