use std::{fmt::Display, hash::Hash, str::FromStr};

use serde::{Deserialize, Serialize, de::Visitor};
use thiserror::Error;
use ts_rs::TS;
use uuid::Uuid;

pub trait Entity<T: PartialEq> {
    fn id(&self) -> T;
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

/// ID - a unique identifier for an entity (UUID)
#[derive(Debug, Clone, Eq, TS)]
#[ts(export)]
pub struct ID(Uuid);

impl ID {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

impl AsMut<Uuid> for ID {
    fn as_mut(&mut self) -> &mut Uuid {
        &mut self.0
    }
}

impl AsRef<Uuid> for ID {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl From<ID> for Uuid {
    fn from(e: ID) -> Self {
        e.0
    }
}

impl From<Uuid> for ID {
    fn from(e: Uuid) -> Self {
        Self(e)
    }
}

impl Default for ID {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Hash for ID {
    fn hash<H: std::hash::Hasher>(&self, content: &mut H) {
        self.0.hash(content);
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
pub enum InvalidIDError {
    #[error("ID: {0} is malformed")]
    Malformed(String),
}

impl FromStr for ID {
    type Err = InvalidIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<Uuid>()
            .map(Self)
            .map_err(|_| InvalidIDError::Malformed(s.to_string()))
    }
}

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Serialize for ID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IDVisitor;

        impl Visitor<'_> for IDVisitor {
            type Value = ID;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A valid string id representation")
            }

            fn visit_str<E>(self, value: &str) -> Result<ID, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<ID>()
                    .map_err(|_| E::custom(format!("Malformed id: {}", value)))
            }
        }

        deserializer.deserialize_str(IDVisitor)
    }
}
