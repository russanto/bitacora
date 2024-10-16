use crate::storage::errors::Error;

use super::entities::Entity;

#[derive(Debug)]
pub enum BitacoraError {
    NotFound,
    AlreadyExists(Entity, String),
    StorageError(Error),
    Web3Error,
    BadIdFormat
}