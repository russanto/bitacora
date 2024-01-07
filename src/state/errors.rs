use crate::storage::errors::Error as StorageError;
use crate::web3::traits::Web3Error;

use super::entities::Entity;

#[derive(Debug)]
pub enum IdError {
    Length(usize, usize),
    Format(&'static str, &'static str),
    Unknown
}

#[derive(Debug)]
pub enum BitacoraError {
    NotFound,
    AlreadyExists(Entity, String),
    StorageError(StorageError),
    Web3Error,
    BadId(IdError)
}

impl From<StorageError> for BitacoraError {
    fn from(value: StorageError) -> Self {
        Self::StorageError(value)
    }
}

impl From<Web3Error> for BitacoraError {
    fn from(value: Web3Error) -> Self {
        Self::Web3Error
    }
}