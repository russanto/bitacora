use crate::state::entities::Entity;

#[derive(Debug)]
pub enum Error {
    FailedRelatingData(String, String),
    InconsistentRelatedData(String, String),
    NotFound(Entity),
    AlreadyExists,
    Generic
}