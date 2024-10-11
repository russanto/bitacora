use std::fmt::Display;

use crate::storage::errors::Error as StorageError;
use crate::web3::traits::Web3Error;

#[derive(Clone, Debug)]
pub enum IdError {
    Length(usize, usize),
    Format(&'static str, &'static str),
    Unknown,
}

#[derive(Clone, Debug)]
pub enum BitacoraError {
    StorageError(StorageError),
    Web3Error,
    BadId(IdError),
    CompletedWithError(Box<BitacoraError>),
}

impl BitacoraError {
    pub fn wrap_with_completed(err: BitacoraError) -> BitacoraError {
        BitacoraError::CompletedWithError(Box::new(err))
    }
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

impl From<BitacoraError> for String {
    fn from(value: BitacoraError) -> Self {
        match value {
            BitacoraError::StorageError(error) => error.into(),
            BitacoraError::BadId(_) => "Unexpected Id format or semantic".into(),
            BitacoraError::CompletedWithError(error) => format!(
                "Operation partially completed with the following error: {}",
                error
            ),
            BitacoraError::Web3Error => "Error with the Web3 interaction".into(),
        }
    }
}

impl Display for BitacoraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}
