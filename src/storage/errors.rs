use crate::state::entities::Entity;

#[derive(Debug, PartialEq)]
pub enum Error {
    FailedRelatingData(String, String),
    MalformedData(String),
    InconsistentRelatedData(String, String),
    NotFound(Entity),
    AlreadyExists,
    NoOp,
    Generic
}