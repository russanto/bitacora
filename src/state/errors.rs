use crate::storage::errors::Error as StorageError;
use crate::web3::traits::Web3Error;

#[derive(Debug)]
pub enum IdError {
    Length(usize, usize),
    Format(&'static str, &'static str),
    Unknown,
}

#[derive(Debug)]
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
