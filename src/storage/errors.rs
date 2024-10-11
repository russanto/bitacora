use std::fmt::Display;

use crate::state::entities::Entity;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    FailedRelatingData(String, String),
    MalformedData(String),
    InconsistentRelatedData(String, String),
    NotFound(Entity),
    AlreadyExists,
    NoOp,
    Generic,
}

impl From<Error> for String {
    fn from(value: Error) -> Self {
        match value {
            Error::FailedRelatingData(e1, e2) => format!("Failed to relate {} with {}", e1, e2),
            Error::MalformedData(field) => format!("Malformed data in field: {}", field),
            Error::InconsistentRelatedData(e1, e2) => {
                format!("Inconsistent data between {} and {}", e1, e2)
            }
            Error::NotFound(entity) => format!("{} not found", entity),
            Error::AlreadyExists => "Entity already exists".into(),
            Error::NoOp => "No operation performed".into(),
            Error::Generic => "Generic error".into(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}
