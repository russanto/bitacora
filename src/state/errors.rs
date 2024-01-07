use crate::storage::errors::Error;
use crate::web3::traits::Web3Error;

use super::entities::Entity;

#[derive(Debug)]
pub enum BitacoraError {
    NotFound,
    AlreadyExists(Entity, String),
    StorageError(Error),
    Web3Error,
    BadIdFormat
}

impl From<Error> for BitacoraError {
    fn from(value: Error) -> Self {
        Self::StorageError(value)
    }
}

impl From<Web3Error> for BitacoraError {
    fn from(value: Web3Error) -> Self {
        Self::Web3Error
    }
}